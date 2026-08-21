# frozen_string_literal: true

require "openssl"

require_relative "errors"

# Verifying that a webhook came from Hook0, and that nothing in it changed on the way.
module Hook0
  # A signature header, read into the pieces a verification needs.
  #
  # A signature names the moment it was signed and one or two message authentication codes over the
  # body. The `v1` scheme also covers a list of request headers, so a receiver can tell apart two
  # deliveries that carry the same body but not the same context; `v0` covers the body alone and is
  # what an older sender still produces. When both are offered, `v1` is the one verified: accepting
  # the weaker of two schemes on the strength of the sender offering it is how a downgrade works.
  #
  # Two things are refused before any code is computed. A header the signature says it covers but
  # the request did not carry is refused outright, because signing over an absent value would let a
  # sender drop a header and keep the signature valid. And a signature whose codes are not whole
  # hexadecimal is refused rather than decoded as far as it goes: a decoder that stops at the first
  # bad character compares a prefix, and a prefix of the right code is not the right code.
  class Signature
    # Longest signature header read. The header is written by whoever reached the endpoint, so its
    # size is bounded before any of it is split, decoded or compared.
    MAX_SIGNATURE_BYTES = 8 * 1024

    # Most `key=value` parts one signature header is split into.
    MAX_SIGNATURE_PARTS = 32

    # Most header names one signature covers.
    MAX_COVERED_HEADERS = 64

    # Furthest from the epoch, in either direction, a signature's moment may sit. Ruby's integers
    # grow without bound, so a header carrying thousands of digits would otherwise reach the
    # arithmetic that holds it against the current time and cost more than reading it did.
    MAX_TIMESTAMP = 10**12

    # What separates one part of the signature header from the next.
    PART_SEPARATOR = ","

    # What separates the name of a part from its value. Only the first one counts: a value may hold
    # further ones, and splitting on all of them would silently drop everything past the second.
    PART_ASSIGNATOR = "="

    # What separates two header names inside the `h` part, and what they are joined back with.
    HEADER_NAME_SEPARATOR = " "

    # What separates the pieces of the message a code is computed over.
    MESSAGE_SEPARATOR = "."

    # Part naming the moment the delivery was signed, in whole seconds since the Unix epoch.
    TIMESTAMP_PART = "t"

    # Part carrying the code covering the body alone.
    BODY_SCHEME_PART = "v0"

    # Part carrying the code covering the covered headers and the body.
    HEADERS_SCHEME_PART = "v1"

    # Part listing the headers the `v1` code covers, in the order it covers them.
    COVERED_HEADERS_PART = "h"

    # What a whole number of seconds reads as. `Integer()` would accept `1_0` as ten, which is a
    # spelling no sender produces and no receiver should invent a meaning for.
    WHOLE_SECONDS = /\A-?\d+\z/

    # What a code reads as: whole pairs of hexadecimal digits, and nothing else.
    WHOLE_HEXADECIMAL = /\A(?:\h\h)+\z/

    # What a header name is written with, as RFC 9110 spells a token.
    HEADER_NAME = /\A[A-Za-z0-9!\#$%&'*+\-.^_`|~]+\z/

    # What the codes are computed with.
    DIGEST = "SHA256"

    # @return [Integer] the moment the delivery was signed, in whole seconds since the epoch
    attr_reader :timestamp

    # @return [Array<String>] the headers the stronger scheme covers, lowercased and in order
    attr_reader :covered_headers

    # @return [String, nil] the `v0` code, decoded
    attr_reader :body_code

    # @return [String, nil] the `v1` code, decoded
    attr_reader :headers_code

    # @param timestamp [Integer]
    # @param covered_headers [Array<String>]
    # @param body_code [String, nil]
    # @param headers_code [String, nil]
    def initialize(timestamp, covered_headers, body_code, headers_code)
      @timestamp = timestamp
      @covered_headers = covered_headers
      @body_code = body_code
      @headers_code = headers_code
      freeze
    end

    # Reads a signature header, refusing anything it cannot read whole.
    #
    # @param signature [String] the value of the `X-Hook0-Signature` header
    # @return [Signature]
    # @raise [ClientError] for every way a header can fail to be one
    def self.parse(signature)
      raise ClientError, "the signature is #{signature.class}, not a header value" unless signature.is_a?(String)

      if signature.length > MAX_SIGNATURE_BYTES
        raise ClientError,
              "the signature is #{signature.length} characters long, above the #{MAX_SIGNATURE_BYTES} accepted"
      end

      read = parts_of(signature)
      raise ClientError, "the signature carries neither a timestamp nor a code" if read.size < 2

      body_code = code_of(read, BODY_SCHEME_PART)
      headers_code = code_of(read, HEADERS_SCHEME_PART)
      if body_code.nil? && headers_code.nil?
        raise ClientError, "the signature carries neither a `#{BODY_SCHEME_PART}` nor a `#{HEADERS_SCHEME_PART}` code"
      end

      new(timestamp_of(read), covered_headers_of(read), body_code, headers_code)
    end

    # The `key=value` parts of a header, split on the first assignator of each and trimmed.
    def self.parts_of(signature)
      parts = signature.split(PART_SEPARATOR, -1)
      if parts.size > MAX_SIGNATURE_PARTS
        raise ClientError, "the signature carries more than the #{MAX_SIGNATURE_PARTS} parts accepted"
      end

      parts.each_with_object({}) do |part, read|
        name, assigned, value = part.partition(PART_ASSIGNATOR)
        read[name.strip] = value.strip unless assigned.empty?
      end
    end
    private_class_method :parts_of

    # The moment the signature names, which it is not a signature without.
    def self.timestamp_of(read)
      written = read[TIMESTAMP_PART]
      raise ClientError, "the signature carries no `#{TIMESTAMP_PART}` part" if written.nil?
      raise ClientError, "`#{written}` is not a number of seconds" unless WHOLE_SECONDS.match?(written)

      seconds = Integer(written, 10)
      if seconds.abs > MAX_TIMESTAMP
        raise ClientError, "the signature's moment is further than #{MAX_TIMESTAMP} seconds from the epoch"
      end

      seconds
    end
    private_class_method :timestamp_of

    # One of the codes a signature offers, decoded whole or not at all.
    def self.code_of(read, part)
      written = read[part]
      return nil if written.nil?
      raise ClientError, "the `#{part}` code is not hexadecimal" unless WHOLE_HEXADECIMAL.match?(written)

      [written].pack("H*")
    end
    private_class_method :code_of

    # The headers the stronger scheme covers, in the order it covers them.
    def self.covered_headers_of(read)
      written = read[COVERED_HEADERS_PART]
      return [] if written.nil? || written.empty?

      names = written.split(HEADER_NAME_SEPARATOR, -1)
      if names.size > MAX_COVERED_HEADERS
        raise ClientError, "the signature covers more than the #{MAX_COVERED_HEADERS} headers accepted"
      end

      names.map do |name|
        raise ClientError, "`#{name}` is not a header name" unless HEADER_NAME.match?(name)

        name.downcase
      end
    end
    private_class_method :covered_headers_of

    # Whether the code this signature carries is the one the secret produces.
    #
    # The stronger scheme wins when both are offered, and the comparison is made in constant time:
    # one that gave up at the first differing byte would say, by how long it took, how much of a
    # guess was right.
    #
    # @param payload [String] the raw body of the webhook request
    # @param covered_values [Array<String>] the values of the covered headers, in order
    # @param subscription_secret [String]
    # @return [Boolean]
    def matches?(payload, covered_values, subscription_secret)
      code = OpenSSL::HMAC.new(subscription_secret.to_s, DIGEST)
      code << @timestamp.to_s
      code << MESSAGE_SEPARATOR

      unless @headers_code.nil?
        code << @covered_headers.join(HEADER_NAME_SEPARATOR)
        code << MESSAGE_SEPARATOR
        code << covered_values.join(MESSAGE_SEPARATOR)
        code << MESSAGE_SEPARATOR
        code << payload
        return Signature.same_code?(code.digest, @headers_code)
      end

      # A signature carrying neither code is refused while it is being read, so what is left here
      # is the body-only scheme.
      code << payload
      Signature.same_code?(code.digest, @body_code)
    end

    # Whether two codes are the same, without saying by how long it took how much of one was right.
    #
    # @param left [String]
    # @param right [String]
    # @return [Boolean]
    def self.same_code?(left, right)
      left.bytesize == right.bytesize && OpenSSL.fixed_length_secure_compare(left, right)
    end
  end

  # Verifies a webhook against a moment the caller names.
  #
  # The clock window is bilateral. A moment too far in the future is refused exactly like one too
  # far in the past, so the window a given delivery is accepted in stays the width the caller asked
  # for, whichever way a clock drifted.
  #
  # @param signature [String] the value of the `X-Hook0-Signature` header
  # @param payload [String] the raw body of the webhook request
  # @param headers [Hash, Array<Array>] the headers of the webhook request
  # @param subscription_secret [String] the signing secret of the subscription it was delivered for
  # @param tolerance [Numeric] how far, in seconds and in either direction, the moment the signature
  #   names may sit from `current_time`. Five minutes is a reasonable trade-off between tolerating
  #   clock drift and bounding how long a captured delivery can be replayed.
  # @param current_time [Time] what to hold the signature's moment against
  # @return [void]
  # @raise [ClientError] for every reason a webhook may be refused
  def self.verify_webhook_signature_with_current_time(
    signature, payload, headers, subscription_secret, tolerance, current_time
  )
    parsed = Signature.parse(signature)

    delivered = delivered_headers(headers)
    covered_values = parsed.covered_headers.map do |name|
      raise ClientError, "the `#{name}` header the signature covers was not delivered" unless delivered.key?(name)

      delivered[name]
    end

    unless parsed.matches?(payload.to_s, covered_values, subscription_secret)
      raise ClientError, "the signature does not match what the subscription secret produces"
    end

    drift = current_time.to_f - parsed.timestamp
    if drift.abs > tolerance
      raise ClientError,
            "the signature was made #{format("%.0f", drift)} seconds from now, outside the #{tolerance} accepted"
    end

    nil
  end

  # Verifies a webhook against the current moment.
  #
  # See {verify_webhook_signature_with_current_time} for what each argument is.
  #
  # @return [void]
  # @raise [ClientError]
  def self.verify_webhook_signature(signature, payload, headers, subscription_secret, tolerance)
    verify_webhook_signature_with_current_time(
      signature, payload, headers, subscription_secret, tolerance, Time.now
    )
  end

  # The headers of the request, under the names a signature refers to them by.
  #
  # A later value wins over an earlier one under the same name, which is what a hash built by the
  # caller would have done.
  def self.delivered_headers(headers)
    headers.to_a.each_with_object({}) do |(name, value), delivered|
      delivered[header_text(name).downcase] = header_text(value)
    end
  end
  private_class_method :delivered_headers

  # A header name or value as text, whichever way the caller holds it.
  def self.header_text(value)
    raise ClientError, "a header is #{value.class}, not a header value" unless value.is_a?(String)
    raise ClientError, "a header is not UTF-8" unless value.dup.force_encoding(Encoding::UTF_8).valid_encoding?

    value
  end
  private_class_method :header_text
end
