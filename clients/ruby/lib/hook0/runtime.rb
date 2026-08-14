# frozen_string_literal: true

require "date"
require "json"
require "time"

module Hook0
  # What the generated half of this gem reads and writes values through.
  #
  # Everything here is hand-written and never regenerated. It is the one seam between what the API
  # declares — the classes, the problems and the methods the generator writes under `generated/` —
  # and what it does not: how a JSON document is turned into a value, and what happens to a document
  # that does not say what it was declared to say.
  #
  # Reading is deliberately strict. A member the document declares as a string and the API answered
  # as a number stops the read with the name of that member, rather than yielding an object whose
  # documentation lies about what it holds. Every failure of that kind is a {DecodeError}, so a
  # caller has one thing to rescue whatever the shape of the answer was.
  #
  # A reader is anything answering to `call`. The scalar ones are constants, since there is exactly
  # one of each; the ones built around another reader are methods, since there is one per shape.
  module Runtime
    # What the API answered is not what it declares it answers.
    class DecodeError < StandardError; end

    # Longest fragment of a response body an error message carries. Bodies are answered by a server
    # this gem does not control, so they are cut at a fixed budget rather than echoed whole into
    # whatever the caller logs.
    MAX_PREVIEW_BYTES = 256

    # Largest JSON document read out of a response body, in bytes. The transport caps what it reads
    # off a socket; this caps what is handed to the parser whichever way the bytes arrived.
    MAX_PAYLOAD_BYTES = 8 * 1024 * 1024

    # Deepest a JSON document may nest before the parser gives up, which is what keeps a document
    # that is nothing but brackets from growing the stack.
    MAX_PAYLOAD_NESTING = 64

    # The characters a path segment carries as themselves; everything else travels percent-encoded.
    UNRESERVED = /[^A-Za-z0-9\-._~]/

    # The shape a UUID is written in, whichever version it carries.
    UUID_PATTERN = /\A\h{8}-\h{4}-\h{4}-\h{4}-\h{12}\z/

    # A string, refusing what merely spells like one.
    TEXT = lambda { |value|
      raise DecodeError, "expected a string, got #{value.class}" unless value.is_a?(String)

      value
    }

    # A UUID, as the document spells one. It travels as the text the API answered, since that text
    # is what has to go back out unchanged.
    UUID = lambda { |value|
      text = TEXT.call(value)
      raise DecodeError, "expected a UUID, got `#{text}`" unless UUID_PATTERN.match?(text)

      text
    }

    # A whole number. `true` is not one, here or on the wire.
    INTEGER = lambda { |value|
      raise DecodeError, "expected a whole number, got #{value.class}" unless value.is_a?(Integer)

      value
    }

    # A number, whether the document wrote it with a fractional part or not.
    FLOAT = lambda { |value|
      raise DecodeError, "expected a number, got #{value.class}" unless value.is_a?(Integer) || value.is_a?(Float)

      value.to_f
    }

    # A boolean, refusing the numbers that stand in for one elsewhere.
    BOOLEAN = lambda { |value|
      raise DecodeError, "expected a boolean, got #{value.class}" unless [true, false].include?(value)

      value
    }

    # A moment, as RFC 3339 spells one.
    DATE_TIME = lambda { |value|
      text = TEXT.call(value)
      begin
        Time.iso8601(text)
      rescue ArgumentError => e
        raise DecodeError, "expected a date and a time, got `#{text}`: #{e.message}"
      end
    }

    # A day, as ISO 8601 spells one.
    DATE = lambda { |value|
      text = TEXT.call(value)
      begin
        Date.iso8601(text)
      rescue ArgumentError => e
        # `Date::Error` is an `ArgumentError`, so rescuing the one covers the other.
        raise DecodeError, "expected a date, got `#{text}`: #{e.message}"
      end
    }

    # A value the document does not describe, which is therefore kept as it arrived.
    JSON_VALUE = ->(value) { value }

    # As much of a response body as a message may carry.
    #
    # @param payload [String]
    # @return [String]
    def self.preview(payload)
      bytes = payload.to_s.b
      kept = bytes.byteslice(0, MAX_PREVIEW_BYTES).force_encoding(Encoding::UTF_8)
      rendered = kept.scrub("�")
      bytes.bytesize > MAX_PREVIEW_BYTES ? "#{rendered}…" : rendered
    end

    # What to say about an answer the API document does not describe.
    #
    # @param status [Integer]
    # @param payload [String]
    # @return [String]
    def self.unreadable(status, payload)
      "the API answered #{status} with a body this client cannot read: #{preview(payload)}"
    end

    # What to say about a problem the API reported.
    #
    # @param status [Integer]
    # @param problem [#to_h] the problem document the API answered
    # @return [String]
    def self.reported(status, problem)
      "the API answered #{status}: #{problem.to_h}"
    end

    # The JSON document a response body carries.
    #
    # @param payload [String]
    # @return [Object]
    # @raise [DecodeError] when the body is larger than this gem reads, or is not JSON
    def self.decode_payload(payload)
      bytes = payload.to_s
      if bytes.bytesize > MAX_PAYLOAD_BYTES
        raise DecodeError, "the response is #{bytes.bytesize} bytes, above the #{MAX_PAYLOAD_BYTES} accepted"
      end

      begin
        JSON.parse(bytes, max_nesting: MAX_PAYLOAD_NESTING)
      rescue JSON::ParserError, EncodingError => e
        raise DecodeError, "the response is not JSON: #{preview(bytes)} (#{e.message})"
      end
    end

    # The members of an object the document declares, under the name it declares it with.
    #
    # @param value [Object]
    # @param owner [String] what the document calls the object being read
    # @return [Hash]
    # @raise [DecodeError] when the API answered something that is not an object
    def self.as_fields(value, owner)
      raise DecodeError, "#{owner} is not a JSON object" unless value.is_a?(Hash)

      value
    end

    # A member the document requires, which is therefore missing when it is absent.
    #
    # @param fields [Hash]
    # @param key [String] the name the member travels under
    # @param reader [#call]
    # @return [Object]
    # @raise [DecodeError]
    def self.read(fields, key, reader)
      raise DecodeError, "`#{key}` is required and was not answered" unless fields.key?(key)

      named(key) { reader.call(fields[key]) }
    end

    # A member the document does not require, absent as readily as answered as null.
    #
    # @param fields [Hash]
    # @param key [String] the name the member travels under
    # @param reader [#call]
    # @return [Object, nil]
    # @raise [DecodeError]
    def self.maybe(fields, key, reader)
      return nil if fields[key].nil?

      named(key) { reader.call(fields[key]) }
    end

    # Every item of an array, each one read the same way.
    #
    # @param reader [#call]
    # @return [Proc]
    def self.list(reader)
      lambda { |value|
        raise DecodeError, "expected an array, got #{value.class}" unless value.is_a?(Array)

        value.map { |item| reader.call(item) }
      }
    end

    # Every value of an object whose keys the document leaves open.
    #
    # @param reader [#call]
    # @return [Proc]
    def self.map(reader)
      lambda { |value|
        raise DecodeError, "expected an object, got #{value.class}" unless value.is_a?(Hash)

        value.to_h { |key, item| [TEXT.call(key), reader.call(item)] }
      }
    end

    # One of the values a closed list declares, refusing anything the list does not carry.
    #
    # @param declared [#member?] the module the generator wrote for that list
    # @return [Proc]
    def self.member_of(declared)
      lambda { |value|
        text = TEXT.call(value)
        raise DecodeError, "`#{text}` is not one of the values #{declared} declares" unless declared.member?(text)

        text
      }
    end

    # A moment, written the way the API reads one.
    #
    # A moment carrying no fraction of a second is written without one, and one that does keeps
    # every digit it has, so that what was read comes back out unchanged either way.
    #
    # @param moment [Time]
    # @return [String]
    def self.moment(moment)
      moment.nsec.zero? ? moment.iso8601 : moment.iso8601(9)
    end

    # A day, written the way the API reads one.
    #
    # @param day [Date]
    # @return [String]
    def self.day(day)
      day.iso8601
    end

    # Where a request lands, with each placeholder of the template filled in.
    #
    # @param template [String] the path as the document writes it, placeholders included
    # @param filled [Hash{String => Object}] the value each placeholder carries
    # @return [String]
    def self.path(template, filled = {})
      filled.reduce(template) do |written, (name, value)|
        written.gsub("{#{name}}", path_segment(value))
      end
    end

    # A value as one segment of a path, with nothing left in it that could name another one.
    #
    # @param value [Object]
    # @return [String]
    def self.path_segment(value)
      written(value).b.gsub(UNRESERVED) { |byte| format("%%%02X", byte.ord) }
    end

    # What travels in the query string: everything the document requires, and everything it does not
    # that the caller actually passed.
    #
    # @param required [Array<Array>] name and value pairs the operation always sends
    # @param optional [Array<Array>] name and value pairs it sends only when they carry something
    # @return [Array<Array<String>>]
    def self.query(required, optional = [])
      asked = required.map { |name, value| [name, written(value)] }
      asked + optional.filter_map { |name, value| [name, written(value)] unless value.nil? }
    end

    # How a value travels in a request line, which is not always how Ruby prints it.
    #
    # @param value [Object]
    # @return [String]
    def self.written(value)
      case value
      when true then "true"
      when false then "false"
      when Time then moment(value)
      when Date then day(value)
      else value.to_s
      end
    end

    # Reads a member, saying which member it was that could not be read.
    def self.named(key)
      yield
    rescue DecodeError => e
      raise DecodeError, "`#{key}`: #{e.message}"
    end
    private_class_method :named
  end
end
