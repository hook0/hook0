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

    def test_an_answer_whose_whole_head_is_comfortably_below_the_maximum_is_read
      # Eight kilobytes of head, half the ceiling and below what the strictest runtime any target
      # runs on refuses. The band just under the bound is deliberately not exercised anywhere: a
      # runtime is free to refuse there, so a case that expected an answer to be read would be
      # green or red depending on the version of the day rather than on this client.
      each = 1024
      narrow = ScriptedResponse.new(201, ingested(INGESTED_ID).body, 0.0,
                                    (1..8).to_h { |i| ["x-pad-#{i}", "v" * (each - "x-pad-#{i}: ".bytesize)] })
      @api.will_answer(narrow)

      assert_equal INGESTED_ID, client.send_event(an_event)
      assert_equal 1, @api.received.size
    end

    def test_an_answer_whose_whole_head_is_above_the_maximum_is_refused_once
      # The case that isolates the total from the components: sixty lines of a kilobyte each sits
      # inside both the line count and the per-line size, and is four times the whole-head ceiling.
      # Sixty-four lines would have been refused on the count instead — `Content-Type` and
      # `Content-Length` bring it to sixty-six — and a refusal for that reason proves nothing here.
      #
      # What is asserted is the refusal above the bound, never an acceptance just below it: the
      # first is a safety property every runtime can honour, the second depends on ceilings the
      # runtime enforces before this client is consulted.
      maximum = 16 * 1024
      each = 1024
      wide = ScriptedResponse.new(201, ingested(INGESTED_ID).body, 0.0,
                                  (1..60).to_h { |i| ["x-pad-#{i}", "v" * (each - "x-pad-#{i}: ".bytesize)] })
      @api.will_answer(wide, wide, wide, wide)

      refused = assert_raises(Hook0::ClientError) do
        client(options(max_attempts: 4, max_head_bytes: maximum)).send_event(an_event)
      end

      assert_includes refused.message, "a head above the #{maximum} bytes read at most"
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

    def test_a_duration_no_schedule_could_be_built_on_is_the_default_both_on_the_wire_and_in_the_wait
      # A value that is not a finite number names no duration, and a policy holding one is in force
      # with the default of that field. Both halves are held to it: what the request states and what
      # the send would actually wait, because a header that named a policy the client does not run
      # would be worse than no header at all.
      unusable = { "+INF" => Float::INFINITY, "-INF" => -Float::INFINITY, "NaN" => Float::NAN }
      cases = unusable.to_a.product(%i[initial_backoff max_backoff max_total_delay])
      @api.will_answer(*Array.new(cases.size) { ScriptedResponse.new(200, []) })

      default = Hook0::RetryPolicy.new
      draws = [0.5, 0.5, 0.5]

      cases.each_with_index do |((named, seconds), held), index|
        policy = Hook0::RetryPolicy.new(held => seconds)
        Hook0::Transport.new(@api.base_url, "token-xyz", retry_policy: policy)
                        .request("GET", "/applications")

        stated = @api.received.fetch(index).headers["hook0-client-options"]
        because = "a policy holding `#{held}: #{named}`"

        assert_equal "attempts=4,backoff=100,ceiling=2000,budget=5000", stated, because
        assert_equal default.delays(draws), policy.delays(draws), because
      end
    end

    def test_an_event_carrying_a_moment_of_its_own_is_sent_under_it_rather_than_under_now
      @api.will_answer(ingested(INGESTED_ID))
      occurred_at = Time.utc(2026, 1, 2, 3, 4, 5)

      client.send_event(an_event(occurred_at: occurred_at))

      assert_equal occurred_at.iso8601, @api.received[0].json["occurred_at"]
    end

    # A document that is nothing but brackets would grow the stack of whatever reads it, so a body
    # nested deeper than this client reads is one it cannot find an identifier in — and an event it
    # cannot name is one whose send has to be reported rather than repeated.
    def test_an_answer_nested_deeper_than_this_client_reads_is_reported_rather_than_repeated
      deep = Hook0::Runtime::MAX_PAYLOAD_NESTING + 4
      @api.will_answer(ScriptedResponse.new(201, deep.times.reduce("bottom") { |held, _| [held] }))

      refused = assert_raises(Hook0::ClientError) { client.send_event(an_event) }

      assert_includes refused.message, "without an event id"
      assert_equal 1, @api.received.size
    end

    # The header is written by the other end, so a delay further out than any schedule could hold is
    # one this client leaves its own schedule in place for rather than waiting out.
    def test_a_delay_the_api_named_further_out_than_any_schedule_could_hold_is_left_unwaited
      named = ScriptedResponse.new(503, { "id" => "ServiceUnavailable", "status" => 503 }, 0.0,
                                   { "retry-after" => "4294967295" })
      @api.will_answer(named, ingested(INGESTED_ID))

      started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      event_id = client(options(max_attempts: 2)).send_event(an_event)

      assert_equal INGESTED_ID, event_id
      assert_operator Process.clock_gettime(Process::CLOCK_MONOTONIC) - started, :<, 1.0,
                      "the client waited out a delay no schedule could hold"
    end

    # The one status that says both a spent quota and a paced instance is told apart by the problem
    # the body names, and a body naming one that is not even text names none.
    def test_a_refusal_naming_a_problem_that_is_not_text_falls_back_to_what_its_status_says
      @api.will_answer(ScriptedResponse.new(429, { "id" => 42, "status" => 429 }))

      refused = assert_raises(Hook0::ClientError) { client.send_event(an_event) }

      assert_includes refused.message, "42"
      assert_equal 1, @api.received.size
    end

    def test_an_event_carrying_metadata_sends_it_beside_the_labels
      @api.will_answer(ingested(INGESTED_ID))

      client.send_event(an_event(metadata: { "tenant" => "acme" }))

      assert_equal({ "tenant" => "acme" }, @api.received[0].json["metadata"])
    end

    # The event was ingested, so repeating the request would meet the same answer; what the caller
    # cannot be given is the identifier it was ingested under.
    def test_an_accepted_event_the_api_named_no_identifier_for_is_reported_rather_than_repeated
      @api.will_answer(ScriptedResponse.new(201, { "received_at" => "2026-01-01" }))

      refused = assert_raises(Hook0::ClientError) { client.send_event(an_event) }

      assert_includes refused.message, "without an event id"
      assert_equal 1, @api.received.size
    end

    # The one status that says both a spent quota and a paced instance is told apart by the problem
    # the body names. A body naming none — here, one that is not a document at all — leaves the
    # status to decide, and that status is one a repeat cannot clear.
    def test_a_refusal_the_api_wrote_no_problem_document_for_falls_back_to_what_its_status_says
      @api.will_answer(ScriptedResponse.new(429, "a gateway wrote this, and it is not JSON"))

      refused = assert_raises(Hook0::ClientError) { client.send_event(an_event) }

      assert_includes refused.message, "a gateway wrote this"
      assert_equal 1, @api.received.size
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
