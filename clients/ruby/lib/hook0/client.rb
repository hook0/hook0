# frozen_string_literal: true

require "json"
require "securerandom"
require "time"

require_relative "errors"
require_relative "transport"

# Sending events to Hook0, idempotently and under bounds the caller sets.
module Hook0
  # How a client spaces out the attempts of a single send.
  #
  # The delay before a retry doubles from {#initial_backoff} and is capped by {#max_backoff}; the
  # delay actually waited is then drawn anywhere between zero and that ceiling, so that emitters
  # which failed at the same moment do not come back at the same moment. Retrying stops as soon as
  # the delays of the send would add up to more than {#max_total_delay}.
  #
  # The defaults are four attempts spread over at most five seconds: three retries absorb the blips
  # a webhook emitter meets in production — a connection reset, a rolling deployment answering 503 —
  # without holding the caller for long, and the five-second budget bounds what the worst send costs
  # whatever the individual delays turn out to be.
  class RetryPolicy
    # Most attempts a policy can ever make, whatever {#max_attempts} says.
    #
    # A policy is configuration, and configuration can be wrong; this cap keeps a mistyped
    # `max_attempts` from turning one send into an unbounded series of requests.
    MAX_ATTEMPTS_CAP = 16

    # Beyond this many doublings any backoff has long since reached its ceiling.
    MAX_BACKOFF_DOUBLINGS = 30

    # @return [Integer] attempts a single send makes at most, the first one included
    attr_reader :max_attempts

    # @return [Float] ceiling of the delay before the first retry, in seconds
    attr_reader :initial_backoff

    # @return [Float] ceiling no single delay ever exceeds, in seconds
    attr_reader :max_backoff

    # @return [Float] budget all the delays of one send share, in seconds
    attr_reader :max_total_delay

    # What each duration of a policy is where a caller named none, in seconds.
    #
    # Declared here rather than written into the signature below, because they are also what a
    # duration falls back to when a caller names one no schedule could be built on: a fallback
    # spelled out a second time is one that will disagree with the default the first time either
    # moves.
    DEFAULT_INITIAL_BACKOFF = 0.1
    DEFAULT_MAX_BACKOFF = 2.0
    DEFAULT_MAX_TOTAL_DELAY = 5.0

    # @param max_attempts [Integer] `1` disables retrying
    # @param initial_backoff [Float]
    # @param max_backoff [Float]
    # @param max_total_delay [Float]
    def initialize(max_attempts: 4, initial_backoff: DEFAULT_INITIAL_BACKOFF,
                   max_backoff: DEFAULT_MAX_BACKOFF, max_total_delay: DEFAULT_MAX_TOTAL_DELAY)
      @max_attempts = max_attempts
      @initial_backoff = initial_backoff
      @max_backoff = max_backoff
      @max_total_delay = max_total_delay
      freeze
    end

    # A policy that never retries: one attempt, and the caller hears what it answered.
    #
    # @return [RetryPolicy]
    def self.disabled
      new(max_attempts: 1, initial_backoff: 0.0, max_backoff: 0.0, max_total_delay: 0.0)
    end

    # Attempts this policy actually makes: {#max_attempts}, brought inside `1..MAX_ATTEMPTS_CAP`.
    #
    # @return [Integer]
    def attempts
      @max_attempts.to_i.clamp(1, MAX_ATTEMPTS_CAP)
    end

    # Ceiling of the delay before retry number `retry_number`, where `1` is the first retry.
    #
    # It doubles from {#initial_backoff} and never exceeds {#max_backoff}, so the ceilings of
    # successive retries never decrease.
    #
    # @param retry_number [Integer]
    # @return [Float]
    def backoff_ceiling(retry_number)
      doublings = (retry_number - 1).clamp(0, MAX_BACKOFF_DOUBLINGS)
      ceiling = max_backoff_in_force
      (initial_backoff_in_force * (2**doublings)).clamp(0.0, ceiling)
    end

    # The delay before the first retry this policy is in force with, in seconds.
    #
    # What a send waits and what a request states are both read from here, so the two cannot come to
    # describe different policies.
    #
    # @return [Float]
    def initial_backoff_in_force
      self.class.in_force(@initial_backoff, DEFAULT_INITIAL_BACKOFF)
    end

    # The ceiling no single delay of this policy exceeds, in seconds.
    #
    # @return [Float]
    def max_backoff_in_force
      self.class.in_force(@max_backoff, DEFAULT_MAX_BACKOFF)
    end

    # The budget all the delays of one send share, in seconds.
    #
    # @return [Float]
    def max_total_delay_in_force
      self.class.in_force(@max_total_delay, DEFAULT_MAX_TOTAL_DELAY)
    end

    # A number of seconds a caller set, brought back to something a schedule can be built on.
    #
    # A value that is not a finite number names no duration at all, and it is read as the one an
    # unconfigured policy holds. Nothing is the tempting reading and the wrong one: a policy whose
    # delays collapse to zero fires its whole schedule back to back, which is the burst a client
    # states its policy so that an instance could recognise — it would manufacture the very traffic
    # the header exists to explain. Unbounded is worse: a send that never comes back. The default is
    # bounded, is what every client falls back to, and leaves the client behaving the way an
    # unconfigured one does, which is what an unusable value should buy.
    #
    # A negative number is a real duration somebody wrote rather than an unusable one, and keeps
    # being read as nothing. `Float::NAN` never reaches an ordering here, which is what used to
    # raise: it answers false to every comparison, so `clamp` and `max` refuse it.
    #
    # @param seconds [Numeric]
    # @param fallback [Float]
    # @return [Float]
    def self.in_force(seconds, fallback)
      number = seconds.to_f
      return fallback unless number.finite?

      [number, 0.0].max
    end

    # The delays this policy waits between the attempts of one send, one per retry.
    #
    # Each delay lands between zero and the ceiling of its retry, and the schedule is cut short as
    # soon as the next delay would spend more than {#max_total_delay}. There are therefore at most
    # `attempts - 1` delays, and they add up to at most `max_total_delay`.
    #
    # A draw that is missing or is not a finite number is read as `1`, which asks for the whole
    # ceiling: an unusable source of randomness makes the client wait longer, never less.
    #
    # @param draws [Array<Float>] one draw in `[0, 1)` per retry
    # @return [Array<Float>]
    def delays(draws)
      budget = max_total_delay_in_force
      waits = []
      spent = 0.0

      1.upto(attempts - 1) do |retry_number|
        delay = backoff_ceiling(retry_number) * self.class.draw(draws, retry_number - 1)
        break if spent + delay > budget

        spent += delay
        waits << delay
      end

      waits
    end

    # The draw for one retry, brought back inside `[0, 1]` whatever the randomness gave.
    #
    # @param draws [Array<Float>]
    # @param index [Integer]
    # @return [Float]
    def self.draw(draws, index)
      drawn = draws[index]
      return 1.0 unless drawn.is_a?(Numeric)
      return 1.0 unless drawn.to_f.finite?

      drawn.to_f.clamp(0.0, 1.0)
    end
  end

  # Every bound a client applies to one send.
  class Options
    # Largest event payload the client agrees to send, in bytes.
    #
    # Hook0's API refuses request bodies above 2 MiB, so a payload above 1 MiB is already at risk of
    # being refused once the JSON envelope around it — metadata, labels, identifiers — is counted.
    # The client rules such an event out rather than spending a round trip, and every retry after
    # it, on a request that cannot be accepted.
    DEFAULT_MAX_PAYLOAD_BYTES = 1024 * 1024

    # @return [RetryPolicy] how the attempts of one send are spaced out
    attr_reader :retry_policy

    # @return [Float] how long one attempt is given, in seconds
    attr_reader :request_timeout

    # @return [Integer] the largest payload sent, refused before a socket is opened
    attr_reader :max_payload_bytes

    # @return [Integer] the largest answer read off a socket
    attr_reader :max_response_bytes

    # @return [Integer] how many header lines an answer may carry
    attr_reader :max_response_headers

    # @return [Integer] the longest one header line may be
    attr_reader :max_header_bytes

    # @return [Integer] the largest whole head, every line counted together
    attr_reader :max_head_bytes

    # @param retry_policy [RetryPolicy]
    # @param request_timeout [Float]
    # @param max_payload_bytes [Integer]
    # @param max_response_bytes [Integer]
    # @param max_response_headers [Integer]
    # @param max_header_bytes [Integer]
    # @param max_head_bytes [Integer]
    def initialize(
      retry_policy: RetryPolicy.new,
      request_timeout: Transport::DEFAULT_REQUEST_TIMEOUT,
      max_payload_bytes: DEFAULT_MAX_PAYLOAD_BYTES,
      max_response_bytes: Transport::DEFAULT_MAX_RESPONSE_BYTES,
      max_response_headers: Transport::DEFAULT_MAX_RESPONSE_HEADERS,
      max_header_bytes: Transport::DEFAULT_MAX_HEADER_BYTES,
      max_head_bytes: Transport::DEFAULT_MAX_HEAD_BYTES
    )
      @retry_policy = retry_policy
      @request_timeout = request_timeout
      @max_payload_bytes = max_payload_bytes
      @max_response_bytes = max_response_bytes
      @max_response_headers = max_response_headers
      @max_header_bytes = max_header_bytes
      @max_head_bytes = max_head_bytes
      freeze
    end
  end

  # An event to send to Hook0.
  #
  # `event_id` is the caller's to set when it already has one to key the event on. Left unset, the
  # client generates a UUIDv7, sends it and answers it — which is what lets it repeat a request
  # without risking a second copy of the event being ingested and delivered to every subscriber.
  class Event
    # @return [String] the type of the event, as the application declares it
    attr_reader :event_type

    # @return [String] what the event carries
    attr_reader :payload

    # @return [String] how to read the payload
    attr_reader :payload_content_type

    # @return [Hash{String => String}] what Hook0 routes the event by
    attr_reader :labels

    # @return [Hash{String => String}, nil] anything else worth carrying
    attr_reader :metadata

    # @return [Time, nil] when the event happened; the current moment when unset
    attr_reader :occurred_at

    # @return [String, nil] what to key the event on; the client chooses when unset
    attr_reader :event_id

    # @param event_type [String]
    # @param payload [String]
    # @param payload_content_type [String]
    # @param labels [Hash{String => String}]
    # @param metadata [Hash{String => String}, nil]
    # @param occurred_at [Time, nil]
    # @param event_id [String, nil]
    def initialize(
      event_type:,
      payload:,
      payload_content_type:,
      labels: {},
      metadata: nil,
      occurred_at: nil,
      event_id: nil
    )
      @event_type = event_type
      @payload = payload
      @payload_content_type = payload_content_type
      @labels = labels
      @metadata = metadata
      @occurred_at = occurred_at
      @event_id = event_id
      freeze
    end
  end

  # An event type, read out of the `service.resource_type.verb` it is written as.
  class EventType
    # What an event type reads as.
    PATTERN = /\A([A-Za-z0-9_]+)\.([A-Za-z0-9_]+)\.([A-Za-z0-9_]+)\z/

    # @return [String] the leading segment
    attr_reader :service

    # @return [String] the middle segment
    attr_reader :resource_type

    # @return [String] the trailing segment
    attr_reader :verb

    # @param service [String]
    # @param resource_type [String]
    # @param verb [String]
    def initialize(service, resource_type, verb)
      @service = service
      @resource_type = resource_type
      @verb = verb
      freeze
    end

    # Reads an event type, refusing one that does not name all three of its parts.
    #
    # @param written [String]
    # @return [EventType]
    # @raise [ClientError]
    def self.parse(written)
      read = PATTERN.match(written.to_s)
      raise ClientError.invalid_event_type(written) if read.nil?

      new(read[1], read[2], read[3])
    end

    # @return [String] the event type as the API reads one
    def to_s
      "#{@service}.#{@resource_type}.#{@verb}"
    end
  end

  # A UUIDv7, the shape of identifier Hook0 mints when it is the one choosing.
  #
  # Its leading 48 bits are the current time in milliseconds, so identifiers generated in sequence
  # are ordered, which is what keeps the index they end up in from being written all over. Written
  # here rather than taken from the standard library, which has had `SecureRandom.uuid_v7` only
  # since Ruby 3.3 and this gem supports older.
  #
  # @return [String]
  def self.generate_event_id
    drawn = SecureRandom.random_bytes(16).unpack("C*")

    milliseconds = (Time.now.to_f * 1000).floor
    6.times { |index| drawn[index] = (milliseconds >> (8 * (5 - index))) & 0xFF }
    drawn[6] = (drawn[6] & 0x0F) | 0x70
    drawn[8] = (drawn[8] & 0x3F) | 0x80

    written = drawn.pack("C*").unpack1("H*")
    [written[0, 8], written[8, 4], written[12, 4], written[16, 4], written[20, 12]].join("-")
  end

  # The Hook0 client, built once and shared wherever an application sends events.
  #
  # Every event is sent under an identifier this client knows: the one set on the {Event}, or a
  # UUIDv7 it generates when the event carries none. Passing none does not mean the identifier comes
  # from Hook0 — the value comes from here, travels with the request, and is what {#send_event}
  # answers.
  #
  # That is what makes retrying safe. Hook0 keys events on that identifier, so a request repeated
  # after a network failure or a server error ingests the event once rather than twice; without a
  # client-chosen identifier, a repeated request would create a second event and deliver it to every
  # subscriber. It also gives the answer to a retry its meaning: `EventAlreadyIngested` in reply to
  # a *repeated* request says an earlier attempt of that same send reached the API, so the send
  # succeeded. The same answer to a *first* attempt is a genuine conflict and is reported as one.
  #
  # Only what could end differently is retried: a request that got no answer, a server error, and an
  # instance saying it is being reached faster than it accepts. What the API refuses outright — a
  # quota that is spent, a payload it will not read — is reported as is, since repeating it would
  # only spend the same round trip again. The verdict for every problem the API can report is
  # written down in the conformance corpus committed beside this gem, which the suite here reads.
  #
  # A send is bounded on five axes, each of them the caller's to set: the size of the payload, which
  # is refused before a socket is opened; how long one attempt is given; how many attempts are made;
  # how long a single wait between them may be; and how long every wait of one send may add up to.
  class Client
    # The identifier Hook0 gives the problem it answers when an event identifier is already taken.
    ALREADY_INGESTED = "EventAlreadyIngested"

    # The identifier Hook0 gives the problem it answers when requests are reaching the instance
    # faster than it accepts them.
    #
    # It shares its status with the quota problems, and is the only one of them worth repeating: a
    # quota clears when a plan changes or a day turns, neither of which happens inside the seconds a
    # send is given, while pacing clears on its own and the answer says when.
    RATE_LIMITED = "RateLimited"

    # What Hook0 answers when the event identifier a request carries is already taken.
    CONFLICT = 409

    # What Hook0 answers both when a quota is spent and when requests are coming in faster than the
    # instance accepts them. Which of the two it is only the problem the body names can say, which
    # is why this status alone decides nothing.
    PACED = 429

    # First status saying the failure is on Hook0's side, and so could clear on its own.
    LOWEST_SERVER_ERROR = 500

    # What the API names the delay before the request becomes servable in, in whole seconds.
    DELAY_HEADER = "retry-after"

    # Longest value of that header read, and the largest delay it may name. A header written by the
    # other end is bounded before it is turned into a number, and a delay above this is one nobody
    # meant.
    MAX_DELAY_HEADER_BYTES = 32
    MAX_NAMED_DELAY_SECONDS = (2**31) - 1

    # What a whole number of seconds reads as, which is the one form of that header this client
    # honours.
    WHOLE_SECONDS = /\A\d+\z/

    # Where an event is ingested, under the API URL.
    EVENT_PATH = "event"

    # Where event types are read and created, under the API URL.
    EVENT_TYPES_PATH = "event_types"

    # @return [String] the base API URL this client reaches
    attr_reader :api_url

    # @return [String] the application events are sent to
    attr_reader :application_id

    # @return [Options] the bounds one send is held to
    attr_reader :options

    # @return [Transport] what this client issues its requests through, which is also what a
    #   generated operation group is built on
    attr_reader :transport

    # @param api_url [String] base API URL of a Hook0 instance, such as https://app.hook0.com/api/v1
    # @param application_id [String] identifier of the Hook0 application events are sent to
    # @param token [String] an authentication token valid for that application
    # @param options [Options] the bounds one send is held to
    def initialize(api_url, application_id, token, options = Options.new)
      @api_url = api_url
      @application_id = application_id
      @options = options
      @transport = Transport.new(
        api_url,
        token,
        timeout: options.request_timeout,
        max_response_bytes: options.max_response_bytes,
        max_response_headers: options.max_response_headers,
        max_header_bytes: options.max_header_bytes,
        max_head_bytes: options.max_head_bytes,
        retry_policy: options.retry_policy
      )
    end

    # Sends an event, and answers the identifier it was sent under.
    #
    # @param event [Event]
    # @return [String]
    # @raise [ClientError] when the event was not ingested
    def send_event(event)
      event_id = identifier_of(event)
      refuse_oversized(event, event_id)

      body = full_event(event, event_id)
      policy = @options.retry_policy
      delays = policy.delays(jitter_draws(policy.attempts - 1))

      issued = 0
      waited = 0.0
      loop do
        issued += 1
        outcome = attempt(body)

        return outcome.ingested unless outcome.ingested.nil?
        return event_id if outcome.already_ingested && issued > 1
        raise ClientError.event_sending(event_id, outcome.detail) if outcome.already_ingested

        scheduled = outcome.retryable ? delays[issued - 1] : nil
        raise given_up(event_id, issued, waited, outcome.detail) if scheduled.nil?

        waiting = wait_for(outcome, scheduled, policy.max_total_delay_in_force - waited)
        sleep(waiting)
        waited += waiting
      end
    end

    # Creates the event types the application does not declare yet, and answers those.
    #
    # @param event_types [Array<String>]
    # @return [Array<String>]
    # @raise [ClientError]
    def upsert_event_types(event_types)
      wanted = event_types.map { |written| EventType.parse(written) }
      return [] if wanted.empty?

      declared = declared_event_types
      wanted.reject { |event_type| declared.include?(event_type.to_s) }.map do |event_type|
        create_event_type(event_type)
        event_type.to_s
      end
    end

    # What one attempt at sending an event ended with, `retry_after` being how long the answer said
    # to wait before repeating the request, in seconds, when it said.
    Attempt = Struct.new(:ingested, :already_ingested, :detail, :retryable, :retry_after)
    private_constant :Attempt

    private

    # The identifier an event is sent under: the one it carries, or one generated for it.
    def identifier_of(event)
      carried = event.event_id
      return carried if carried.is_a?(String) && !carried.empty?

      Hook0.generate_event_id
    end

    # Rules an oversized payload out, before a socket is opened for it.
    def refuse_oversized(event, event_id)
      size = event.payload.to_s.bytesize
      return if size <= @options.max_payload_bytes

      raise ClientError.payload_too_large(event_id, size, @options.max_payload_bytes)
    end

    # An event as the API reads one.
    def full_event(event, event_id)
      occurred_at = event.occurred_at.nil? ? Time.now.utc : event.occurred_at
      body = {
        "event_id" => event_id,
        "application_id" => @application_id,
        "event_type" => event.event_type,
        "payload" => event.payload,
        "payload_content_type" => event.payload_content_type,
        "occurred_at" => occurred_at.iso8601,
        "labels" => event.labels.to_h
      }
      body["metadata"] = event.metadata.to_h unless event.metadata.nil?
      body
    end

    # One attempt at sending an already-bounded event.
    def attempt(body)
      status, headers, payload = @transport.deliver("POST", EVENT_PATH, [], body)
      read_attempt(status, headers, payload)
    rescue TransportError => e
      Attempt.new(nil, false, e.message, e.retryable?, nil)
    end

    # What the API answered one attempt, and whether repeating it could end differently.
    def read_attempt(status, headers, payload)
      body = payload.to_s

      if status >= 200 && status < 300
        ingested = ingested_id(body)
        # The API accepted the event but answered something this client cannot read; repeating the
        # request would meet the same answer.
        return Attempt.new(nil, false, "Hook0 answered #{status} without an event id", false, nil) if ingested.nil?

        return Attempt.new(ingested, false, "", false, nil)
      end

      problem = problem_id(body)
      return Attempt.new(nil, true, body, false, nil) if status == CONFLICT && problem == ALREADY_INGESTED

      Attempt.new(nil, false, body, retryable?(status, problem), named_delay(headers))
    end

    # Whether repeating a request the API answered that way could end differently.
    #
    # The status decides on its own everywhere but under the one it answers both a spent quota and a
    # paced instance with: a quota clears when a plan changes or a day turns, and neither is
    # something a send spending seconds can wait for. Only the problem the body names tells the two
    # apart, and a body naming a problem this client has never heard of falls back to what the
    # status says.
    def retryable?(status, problem)
      return problem == RATE_LIMITED if status == PACED

      status >= LOWEST_SERVER_ERROR
    end

    # The delay the API named before the request becomes servable, in seconds.
    #
    # Only a whole number of seconds is read. The header may also carry a date, which is a clock
    # this client would be comparing against its own, and anything else is a header nobody meant:
    # both leave the client's own schedule in place rather than being guessed at.
    def named_delay(headers)
      written = headers.to_h.fetch(DELAY_HEADER, "").to_s.strip
      return nil if written.empty? || written.bytesize > MAX_DELAY_HEADER_BYTES
      return nil unless WHOLE_SECONDS.match?(written)

      seconds = Integer(written, 10)
      seconds > MAX_NAMED_DELAY_SECONDS ? nil : seconds.to_f
    end

    # How long to wait before the next attempt.
    #
    # It is what the API asked for when it asked for anything, and the client's own schedule
    # otherwise. Either way it is cut down to what is left of the budget every delay of one send
    # shares, so a delay written by the other end cannot stretch a send past what the caller allowed
    # for it.
    def wait_for(outcome, scheduled, remaining)
      wanted = outcome.retry_after.nil? ? scheduled : outcome.retry_after
      wanted.clamp(0.0, [remaining, 0.0].max)
    end

    # The identifier the API says it ingested the event under.
    def ingested_id(body)
      answered = parsed(body)
      return nil unless answered.is_a?(Hash)

      ingested = answered["event_id"]
      ingested.is_a?(String) ? ingested : nil
    end

    # The problem a refusal names, unset when the body names none this client can read.
    def problem_id(body)
      problem = parsed(body)
      return nil unless problem.is_a?(Hash)

      named = problem["id"]
      named.is_a?(String) ? named : nil
    end

    def parsed(body)
      JSON.parse(body, max_nesting: Runtime::MAX_PAYLOAD_NESTING)
    rescue JSON::ParserError, EncodingError
      nil
    end

    # What to raise when a send is being given up on.
    def given_up(event_id, attempts, waited, detail)
      return ClientError.event_sending(event_id, detail) if attempts <= 1

      ClientError.retries_exhausted(event_id, attempts, waited, detail)
    end

    # The randomness used to jitter the delays of one send.
    #
    # Jitter only has to keep emitters that failed together from coming back together; it does not
    # have to be unpredictable, so the platform's own generator is enough.
    def jitter_draws(count)
      Array.new([count, 0].max) { Random.rand }
    end

    # The event types an application already declares, out of what the API answered.
    def declared_event_types
      begin
        status, payload = @transport.request("GET", EVENT_TYPES_PATH, [["application_id", @application_id]])
      rescue TransportError => e
        raise ClientError.available_event_types(e.message)
      end
      raise ClientError.available_event_types(payload.to_s) unless status >= 200 && status < 300

      answered = parsed(payload.to_s)
      unless answered.is_a?(Array)
        raise ClientError.available_event_types("the API did not answer a list of event types")
      end

      answered.filter_map { |entry| declared_name(entry) }
    end

    # The name one entry of the list the API answered declares, when it declares one.
    def declared_name(entry)
      return nil unless entry.is_a?(Hash)

      name = entry["event_type_name"]
      name.is_a?(String) ? name : nil
    end

    # Declares one event type on the application.
    def create_event_type(event_type)
      body = {
        "application_id" => @application_id,
        "service" => event_type.service,
        "resource_type" => event_type.resource_type,
        "verb" => event_type.verb
      }

      begin
        status, payload = @transport.request("POST", EVENT_TYPES_PATH, [], body)
      rescue TransportError => e
        raise ClientError.creating_event_type(event_type.to_s, e.message)
      end
      return if status >= 200 && status < 300

      raise ClientError.creating_event_type(event_type.to_s, payload.to_s)
    end
  end
end
