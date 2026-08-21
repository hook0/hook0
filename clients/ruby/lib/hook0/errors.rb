# frozen_string_literal: true

module Hook0
  # The one failure this client reports.
  #
  # Sending an event, upserting an event type and verifying a webhook all raise this, so a caller
  # has one thing to rescue whatever it asked the client to do. The failures the *API* reports are a
  # different matter: those are the problems it names in its own error contract, and the generated
  # half of this gem raises one exception per problem.
  #
  # The constructors below are what the client raises through; each one exists so that the same
  # situation always reads the same way, whichever call it came out of.
  class ClientError < StandardError
    # A send the API refused for a reason repeating it would not change.
    #
    # @param event_id [String]
    # @param detail [String]
    # @return [ClientError]
    def self.event_sending(event_id, detail)
      new("Sending event #{event_id} failed: #{detail}")
    end

    # A send that ran out of attempts, or out of the delay budget its attempts share.
    #
    # A send that gave up and a single refused request are otherwise indistinguishable to a caller,
    # which is the difference between a transient outage and a request that will never be accepted.
    #
    # @param event_id [String]
    # @param attempts [Integer]
    # @param waited [Float] seconds spent waiting between the attempts
    # @param detail [String]
    # @return [ClientError]
    def self.retries_exhausted(event_id, attempts, waited, detail)
      new(
        "Sending event #{event_id} failed: gave up after #{attempts} attempts " \
        "spread over #{format("%.3f", waited)}s of retry delay; last failure: #{detail}"
      )
    end

    # A payload above what the client agrees to send, refused before a socket is opened.
    #
    # @param event_id [String]
    # @param size [Integer]
    # @param maximum [Integer]
    # @return [ClientError]
    def self.payload_too_large(event_id, size, maximum)
      new(
        "Sending event #{event_id} failed: event payload is #{size} bytes, which is more than " \
        "the #{maximum} bytes this client sends at most; nothing was sent"
      )
    end

    # An event type that does not read as `service.resource_type.verb`.
    #
    # @param event_type [String]
    # @return [ClientError]
    def self.invalid_event_type(event_type)
      new("Provided event type '#{event_type}' does not have a valid syntax (service.resource_type.verb)")
    end

    # The list of event types the application already declares could not be read.
    #
    # @param detail [String]
    # @return [ClientError]
    def self.available_event_types(detail)
      new("Getting available event types failed: #{detail}")
    end

    # An event type that could not be created.
    #
    # @param event_type [String]
    # @param detail [String]
    # @return [ClientError]
    def self.creating_event_type(event_type, detail)
      new("Creating event type '#{event_type}' failed: #{detail}")
    end
  end
end
