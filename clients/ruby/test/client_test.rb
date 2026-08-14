# frozen_string_literal: true

# What a send does over a real socket.
#
# What is asserted is what the API saw — how many requests arrived, and what each of them carried —
# rather than what the client was asked to do, so a client that reports success without having sent
# anything fails here.

require_relative "test_helper"

module Hook0Test
  class ClientTest < ApiCase
    INGESTED_ID = "01961234-5678-7abc-8def-0123456789ab"

    def test_a_send_that_succeeds_issues_one_request
      @api.will_answer(ingested(INGESTED_ID))

      event_id = client.send_event(an_event)

      assert_equal INGESTED_ID, event_id
      assert_equal 1, @api.received.size
    end

    def test_an_event_carrying_no_id_is_sent_under_one_the_client_generated
      @api.will_answer(ingested(INGESTED_ID))

      client.send_event(an_event)

      # Without an identifier of its own, a repeated request makes Hook0 mint a second one, and the
      # event is ingested and delivered twice.
      assert_match UUID_PATTERN, @api.event_id_of(0)
    end

    def test_an_event_carrying_an_id_is_sent_under_it
      chosen = "00000000-0000-0000-0000-000000000000"
      @api.will_answer(ingested(chosen))

      event_id = client.send_event(an_event(event_id: chosen))

      assert_equal chosen, event_id
      assert_equal chosen, @api.event_id_of(0)
    end

    def test_an_attempt_that_ran_out_of_time_is_repeated_under_the_same_id
      @api.will_answer(
        ScriptedResponse.new(201, ingested(INGESTED_ID).body, 1.0),
        ingested(INGESTED_ID)
      )

      event_id = client(options(max_attempts: 3, request_timeout: 0.2)).send_event(an_event)

      assert_equal INGESTED_ID, event_id
      assert_equal 2, @api.received.size
      # The retry has to repeat the identifier of the attempt it repeats, or Hook0 ingests twice.
      assert_equal @api.event_id_of(0), @api.event_id_of(1)
      assert_match UUID_PATTERN, @api.event_id_of(0)
    end

    def test_repeated_server_errors_stop_at_the_configured_number_of_attempts
      @api.will_answer(server_error, server_error, server_error, server_error)

      refused = assert_raises(Hook0::ClientError) do
        client(options(max_attempts: 3)).send_event(an_event)
      end

      assert_includes refused.message, "gave up after 3 attempts"
      assert_equal 3, @api.received.size
    end

    def test_an_answer_the_api_would_repeat_is_not_retried
      @api.will_answer(ScriptedResponse.new(429, { "id" => "TooManyEventsToday", "status" => 429 }))

      # A quota that is spent for the day cannot clear itself between two attempts.
      refused = assert_raises(Hook0::ClientError) { client(options(max_attempts: 4)).send_event(an_event) }

      assert_includes refused.message, "Sending event"
      assert_equal 1, @api.received.size
    end

    def test_a_retry_answered_that_the_event_was_already_ingested_reports_success
      @api.will_answer(server_error, already_ingested)

      event_id = client(options(max_attempts: 3)).send_event(an_event)

      # The conflict is the mark of the attempt this one repeats having reached the API.
      assert_equal @api.event_id_of(0), event_id
      assert_equal 2, @api.received.size
    end

    def test_a_first_attempt_answered_that_the_event_was_already_ingested_reports_the_conflict
      @api.will_answer(already_ingested)

      # Nothing this send did can explain the conflict, so the caller has to hear about it.
      refused = assert_raises(Hook0::ClientError) { client(options(max_attempts: 3)).send_event(an_event) }

      assert_includes refused.message, "EventAlreadyIngested"
      assert_equal 1, @api.received.size
    end

    def test_a_client_that_does_not_retry_issues_one_request
      @api.will_answer(server_error, server_error, server_error)

      assert_raises(Hook0::ClientError) do
        client(Hook0::Options.new(retry_policy: Hook0::RetryPolicy.disabled)).send_event(an_event)
      end

      assert_equal 1, @api.received.size
    end

    def test_a_payload_above_the_maximum_is_refused_before_any_request
      maximum = 16
      @api.will_answer(ingested(INGESTED_ID))

      refused = assert_raises(Hook0::ClientError) do
        client(options(max_payload_bytes: maximum)).send_event(an_event(payload: "x" * (maximum + 1)))
      end

      assert_includes refused.message, "#{maximum} bytes this client sends at most"
      assert_empty @api.received
    end

    def test_an_answer_carrying_more_header_lines_than_the_maximum_is_refused_once
      # `Net::HTTP` holds however many header lines a server writes, so the ceiling is this client's
      # to apply. A repetition would read the same oversized head again, which is why it is one.
      maximum = 8
      crowded = ScriptedResponse.new(201, ingested(INGESTED_ID).body, 0.0,
                                     (1..(maximum + 1)).to_h { |i| ["x-pad-#{i}", "v"] })
      @api.will_answer(crowded, crowded, crowded, crowded)

      refused = assert_raises(Hook0::ClientError) do
        client(options(max_attempts: 4, max_response_headers: maximum)).send_event(an_event)
      end

      assert_includes refused.message, "#{maximum} header lines read at most"
      assert_equal 1, @api.received.size
    end

    def test_an_answer_carrying_a_header_line_above_the_maximum_is_refused_once
      maximum = 512
      padded = ScriptedResponse.new(201, ingested(INGESTED_ID).body, 0.0, { "x-pad" => "v" * (maximum + 1) })
      @api.will_answer(padded, padded, padded, padded)

      refused = assert_raises(Hook0::ClientError) do
        client(options(max_attempts: 4, max_header_bytes: maximum)).send_event(an_event)
      end

      assert_includes refused.message, "`x-pad` header above the #{maximum} bytes read at most"
      assert_equal 1, @api.received.size
    end

    def test_a_send_carries_the_application_and_the_credential
      @api.will_answer(ingested(INGESTED_ID))

      client.send_event(an_event)

      request = @api.received[0]

      assert_equal "POST", request.verb
      assert request.target.end_with?("/event")
      assert_equal "Bearer token-xyz", request.headers["authorization"]
      assert_equal "app-123", request.json["application_id"]
      assert_equal({ "environment" => "production" }, request.json["labels"])
    end

    def test_event_types_the_application_already_declares_are_not_created
      @api.will_answer(
        ScriptedResponse.new(200, [{ "event_type_name" => "auth.user.create" }]),
        ScriptedResponse.new(201, { "event_type_name" => "billing.invoice.paid" })
      )

      created = client.upsert_event_types(["auth.user.create", "billing.invoice.paid"])

      assert_equal ["billing.invoice.paid"], created
      assert_equal 2, @api.received.size
    end

    def test_an_event_type_that_does_not_read_as_three_parts_is_refused
      refused = assert_raises(Hook0::ClientError) { client.upsert_event_types(["not-an-event-type"]) }

      assert_includes refused.message, "does not have a valid syntax"
      assert_empty @api.received
    end

    def test_the_default_schedule_doubles_up_to_its_ceiling
      policy = Hook0::RetryPolicy.new

      assert_in_delta 0.1, policy.backoff_ceiling(1)
      assert_in_delta 0.2, policy.backoff_ceiling(2)
      assert_in_delta 0.4, policy.backoff_ceiling(3)
      assert_in_delta 2.0, policy.backoff_ceiling(6)
      assert_in_delta 2.0, policy.backoff_ceiling(16)

      # A source of randomness that gives nothing asks for the whole ceiling, which is what makes
      # the budget the thing that cuts the schedule short.
      delays = policy.delays([])

      assert_equal 3, delays.size
      assert_operator delays.sum, :<=, policy.max_total_delay
    end

    def test_a_disabled_policy_waits_for_nothing
      policy = Hook0::RetryPolicy.disabled

      assert_equal 1, policy.attempts
      assert_empty policy.delays([1.0, 1.0, 1.0])
    end

    def test_minted_identifiers_are_ordered_by_the_moment_they_were_minted
      # The leading 48 bits are the moment in milliseconds, so identifiers minted in sequence never
      # go back in time — which is what keeps the index they end up in from being written all over.
      # Two minted inside one millisecond differ only in what was drawn, so what is ordered is the
      # moment they carry rather than the whole of them.
      minted = Array.new(64) { Hook0.generate_event_id }
      moments = minted.map { |identifier| identifier[0, 8] + identifier[9, 4] }

      assert_equal moments.sort, moments
      minted.each do |identifier|
        assert_match UUID_PATTERN, identifier
        assert_equal "7", identifier[14]
        assert_includes %w[8 9 a b], identifier[19]
      end

      # Far enough apart to land in another millisecond, the whole identifier is ordered too.
      earlier = Hook0.generate_event_id
      sleep(0.005)

      assert_operator earlier, :<, Hook0.generate_event_id
    end
  end
end
