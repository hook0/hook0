# frozen_string_literal: true

require "json"
require "net/http"
require "uri"

module Hook0
  # A request the API never answered, and what caused that.
  #
  # The three causes are told apart because only one of them could end differently. A request that
  # got no answer — a connection refused or reset, an attempt out of time, a body that stopped
  # mid-way — says nothing about whether the API acted on it, which is exactly why a send carries an
  # identifier the client chose itself, and why repeating it is safe and worth doing. An answer that
  # crossed a ceiling this client set for itself draws the same answer the second time, and reading
  # it again four times over costs the caller four times as much for the same failure. A URL nothing
  # can be sent to was never sent at all, and a repetition builds the same unusable request, turning
  # a misconfiguration into a message that accuses the network.
  #
  # The names are the ones the shared conformance corpus gives them, so the verdict a client applies
  # and the verdict that corpus writes down are the same words.
  class TransportError < StandardError
    # @return [String] which of the corpus's causes this is
    attr_reader :cause_name

    # @param detail [String] what went wrong, in the words a caller is given
    # @param cause_name [String] which of the corpus's causes this is
    # @param retryable [Boolean] whether repeating the request could end differently
    def initialize(detail, cause_name, retryable)
      super(detail)
      @cause_name = cause_name
      @retryable = retryable
    end

    # Whether repeating the request that met this could end differently.
    #
    # @return [Boolean]
    def retryable?
      @retryable
    end

    # The API was reached for and answered nothing this client could read to its end.
    #
    # @param detail [String]
    # @return [TransportError]
    def self.no_answer(detail)
      new(detail, "no_answer", true)
    end

    # The API answered, and what it answered crossed a ceiling this client set for itself.
    #
    # @param detail [String]
    # @return [TransportError]
    def self.answer_above_a_bound(detail)
      new(detail, "answer_above_a_bound", false)
    end

    # There is nowhere to send the request, so nothing was sent.
    #
    # @param detail [String]
    # @return [TransportError]
    def self.unusable_api_url(detail)
      new(detail, "unusable_api_url", false)
    end
  end

  # How a request reaches the API, and what a server on the other end is not allowed to cost.
  #
  # The transport answers the status and the bytes and knows nothing of what the API declares:
  # reading those bytes is the generated half's job, and deciding whether to send them again is the
  # client's. That is what lets one HTTP implementation serve both the hand-written event path and
  # every generated method — a generated group calls whatever object it is handed, and this is the
  # one this gem ships.
  #
  # Nothing here reaches for a third-party HTTP library. Everything a server controls is bounded:
  # how long one exchange may take, and how many bytes of body are read off the socket.
  class Transport
    # Longest one attempt at reaching the API is given before it is abandoned, in seconds.
    #
    # Ten seconds is far above what ingesting an event takes when the API is healthy, and short
    # enough that a stuck connection does not hold a caller for a noticeable time.
    DEFAULT_REQUEST_TIMEOUT = 10.0

    # Largest response body read off a socket, in bytes.
    DEFAULT_MAX_RESPONSE_BYTES = 8 * 1024 * 1024

    # How many header lines an answer may carry before it is refused.
    #
    # `Net::HTTP` bounds neither how many header lines it accepts nor how long one may be: it holds
    # fifty thousand of them, and a single value of eight megabytes, without complaint. So the head
    # of an answer is a server-controlled way to spend a caller's memory, and the ceiling has to be
    # this client's own. Sixty-four is well above what the API sends.
    DEFAULT_MAX_RESPONSE_HEADERS = 64

    # Longest one header line may be, name and value together, in bytes.
    DEFAULT_MAX_HEADER_BYTES = 64 * 1024

    # What a request body says it carries, and what an answer is asked for in.
    JSON_MEDIA_TYPE = "application/json"

    # The schemes this transport reaches.
    SCHEMES = %w[http https].freeze

    # What the standard library reports when the API was not reached at all.
    UNREACHABLE = [
      IOError,
      SocketError,
      SystemCallError,
      Timeout::Error,
      Net::HTTPBadResponse,
      Net::HTTPHeaderSyntaxError,
      Net::ProtocolError,
      OpenSSL::SSL::SSLError
    ].freeze

    # @param base_url [String] where the API lives, such as https://app.hook0.com/api/v1
    # @param token [String] an authentication token valid for that API
    # @param timeout [Float] how long one attempt is given, in seconds
    # @param max_response_bytes [Integer] the largest answer read off a socket
    # @param max_response_headers [Integer] how many header lines an answer may carry
    # @param max_header_bytes [Integer] the longest one header line may be
    def initialize(
      base_url,
      token,
      timeout: DEFAULT_REQUEST_TIMEOUT,
      max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
      max_response_headers: DEFAULT_MAX_RESPONSE_HEADERS,
      max_header_bytes: DEFAULT_MAX_HEADER_BYTES
    )
      @base_url = base_url
      @token = token
      @timeout = timeout
      @max_response_bytes = max_response_bytes
      @max_response_headers = max_response_headers
      @max_header_bytes = max_header_bytes
    end

    # What the API answered, whether or not it answered a success.
    #
    # This is the shape the generated half of this gem reads, which is the status and the bytes. A
    # caller that also needs what the answer carried beside its body — the delay a paced instance
    # names is one — asks {#deliver} for it.
    #
    # @param method [String] the HTTP method the operation is issued under
    # @param path [String] where the request lands, absolute or under the base URL
    # @param query [Array<Array<String>>] the name and value pairs of the query string
    # @param body [Object, nil] what to send as a JSON document, or nothing at all
    # @return [Array(Integer, String)] the status and the body
    # @raise [TransportError] when the API answered nothing at all
    def request(method, path, query = [], body = nil)
      status, _, payload = deliver(method, path, query, body)
      [status, payload]
    end

    # What the API answered, headers included, whether or not it answered a success.
    #
    # Header names are lowercased and a later value wins over an earlier one under the same name, so
    # a caller reads a header without knowing which case the server wrote it in.
    #
    # @param method [String] the HTTP method the operation is issued under
    # @param path [String] where the request lands, absolute or under the base URL
    # @param query [Array<Array<String>>] the name and value pairs of the query string
    # @param body [Object, nil] what to send as a JSON document, or nothing at all
    # @return [Array(Integer, Hash{String => String}, String)] the status, the headers and the body
    # @raise [TransportError] when the API answered nothing at all
    def deliver(method, path, query = [], body = nil)
      target = resolved(path, query)
      exchange(target, built(method, target, body))
    end

    private

    # Where a request lands: a path of its own replaces the base's, a relative one extends it.
    def resolved(path, query)
      begin
        target = URI.join("#{@base_url.to_s.chomp("/")}/", path.to_s)
      rescue URI::Error => e
        raise TransportError.unusable_api_url(e.message)
      end
      unless reachable?(target)
        raise TransportError.unusable_api_url("`#{target}` is not somewhere this transport can send a request")
      end
      return target if query.nil? || query.empty?

      separator = target.query.nil? || target.query.empty? ? "" : "#{target.query}&"
      target.query = "#{separator}#{URI.encode_www_form(query)}"
      target
    end

    def reachable?(target)
      SCHEMES.include?(target.scheme) && !target.host.nil? && !target.host.empty?
    end

    # One request, as Net::HTTP carries it.
    def built(method, target, body)
      request = Net::HTTPGenericRequest.new(method.to_s.upcase, !body.nil?, true, target)
      request["Authorization"] = "Bearer #{@token}"
      request["Accept"] = JSON_MEDIA_TYPE
      unless body.nil?
        request["Content-Type"] = JSON_MEDIA_TYPE
        request.body = JSON.generate(body)
      end
      request
    end

    # One exchange, bounded on every axis a server controls.
    def exchange(target, request)
      http = Net::HTTP.new(target.host, target.port)
      http.use_ssl = target.scheme == "https"
      http.open_timeout = @timeout
      http.read_timeout = @timeout
      http.write_timeout = @timeout

      # `Net::HTTP#request` answers the response rather than what the block it was given answered,
      # so what the block worked out is kept here instead of returned through it.
      answered = nil
      http.start do |session|
        session.request(request) do |answer|
          answered = [answer.code.to_i, carried(answer), bounded(answer)]
        end
      end
      answered
    rescue TransportError
      raise
    rescue *UNREACHABLE => e
      raise TransportError.no_answer(e.message)
    end

    # What an answer carried beside its body, under the names a caller looks them up by.
    #
    # Refused before the body is read, so an abusive head costs one pass over what `Net::HTTP` has
    # already buffered rather than that plus a megabyte-scale body on top.
    def carried(answer)
      held = 0
      answer.each_header.to_h do |name, value|
        held += 1
        if held > @max_response_headers
          raise TransportError.answer_above_a_bound(
            "the API answered more than the #{@max_response_headers} header lines read at most"
          )
        end

        line = name.to_s.bytesize + value.to_s.bytesize
        if line > @max_header_bytes
          raise TransportError.answer_above_a_bound(
            "the API answered a `#{name.to_s.downcase}` header above the #{@max_header_bytes} bytes read at most"
          )
        end

        [name.downcase, value.to_s.strip]
      end
    end

    # The body of an answer, up to what this transport agrees to hold.
    def bounded(answer)
      payload = +""
      answer.read_body do |chunk|
        payload << chunk
        if payload.bytesize > @max_response_bytes
          raise TransportError.answer_above_a_bound(
            "the API answered more than the #{@max_response_bytes} bytes read at most"
          )
        end
      end
      payload
    end
  end
end
