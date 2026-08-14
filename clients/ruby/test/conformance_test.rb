# frozen_string_literal: true

# The cases the shared conformance corpus dictates, run against this client.
#
# The corpus sits at `clients/conformance`, is hand-authored, and is read by the suite of every SDK.
# Nothing below writes down a verdict, a bound or a signature of its own: they are read out of the
# committed documents and this client is driven against them over a real socket. A case added to the
# corpus is therefore exercised here without this file being touched, and a verdict changed there
# fails here until this client agrees with it again.

require_relative "test_helper"

module Hook0Test
  class ConformanceTest < ApiCase
    RETRY = Hook0Test.contract("retry.json")
    BOUNDS = Hook0Test.contract("bounds.json")["bounds"]
    SIGNATURE = Hook0Test.contract("signature.json")

    INGESTED_ID = "01961234-5678-7abc-8def-0123456789ac"

    # The schedule a case that is not about waiting spends between attempts, in seconds.
    PROMPT_BACKOFF = 0.005

    # The budget the delay cases share. A delay the API names above it is expected to be cut down to
    # it, so this also bounds what those cases cost.
    DELAY_BUDGET = 1.1

    # What a wait may overshoot by before it is read as more than what was asked for: a loopback
    # round trip, a timer and a scheduler all sit inside it.
    DELAY_SLACK = 0.6

    # How a refusal the corpus names reads in this client's own words. Every name the corpus
    # declares is looked up here, so one added there stops this suite until it is mapped rather than
    # passing under whatever the client happened to say.
    REFUSALS = {
      "code_not_hexadecimal" => "not hexadecimal",
      "header_not_delivered" => "was not delivered",
      "code_mismatch" => "does not match",
      "outside_tolerance" => "outside the"
    }.freeze

    def test_the_corpus_says_what_every_problem_does_to_a_send
      # The status is not what decides: the corpus carries problems answering the same status with
      # opposite verdicts, and a client reading the status alone fails half of them.
      RETRY["problems"].each do |rule|
        issued, ingested_event = issued_for(refusal(rule["status"], rule["problem"]))
        expected = rule["retryable"] ? 2 : 1

        assert_equal expected, issued,
                     "`#{rule["problem"]}` under #{rule["status"]} issued #{issued} requests where the " \
                     "corpus expects #{expected}: #{rule["reason"]}"
        assert_equal rule["retryable"], ingested_event
      end
    end

    def test_the_corpus_says_what_every_status_does_to_a_send
      # A body naming no problem this client could read is also what an older client meets when the
      # API names a problem it has never heard of.
      RETRY["statuses"].each do |rule|
        issued, = issued_for(refusal(rule["status"], "AProblemThisClientHasNeverHeardOf"))
        expected = rule["retryable"] ? 2 : 1

        assert_equal expected, issued,
                     "a status of #{rule["status"]} issued #{issued} requests where the corpus " \
                     "expects #{expected}: #{rule["reason"]}"
      end
    end

    def test_the_corpus_says_what_a_request_the_api_never_answered_does
      # Every cause the corpus names is provoked for real rather than reported: a server that sits
      # on an answer past the timeout, an answer above a ceiling this client set for itself, and a
      # URL nothing can be sent to.
      RETRY["transport"]["causes"].each do |rule|
        issued, survived = provoked(rule["cause"])
        expected = rule["retryable"] ? 2 : 1

        assert_equal expected, issued,
                     "`#{rule["cause"]}` issued #{issued} requests where the corpus expects " \
                     "#{expected}: #{rule["reason"]}"
        assert_equal rule["retryable"], survived
      end
    end

    def test_the_delay_the_api_names_is_honoured_and_bounded
      # The header is written by the other end, so honouring it whole would hand a stranger the
      # length of this client's send. What the corpus asks for is that a delay be waited out when
      # the budget can afford it and cut down to what is left of the budget when it cannot.
      RETRY["retry_after"]["cases"].each do |delay|
        waited = waited_for(delay)
        expected = delay["honoured"] ? [delay["seconds"].to_f, DELAY_BUDGET].min : 0.0

        assert_operator waited, :>=, expected,
                        "`#{RETRY["retry_after"]["header"]}: #{delay["header"]}` was retried after " \
                        "#{format("%.3f", waited)}s, sooner than the #{format("%.3f", expected)}s it asked for"
        assert_operator waited, :<=, expected + DELAY_SLACK,
                        "`#{RETRY["retry_after"]["header"]}: #{delay["header"]}` held the send for " \
                        "#{format("%.3f", waited)}s, above the #{format("%.3f", expected)}s it is bounded to"
      end
    end

    def test_the_bounds_are_the_ones_the_corpus_names
      # This client's defaults, held against the one place the numbers are written down. What is
      # asserted is read from the corpus rather than listed here, so a bound added there and left
      # unapplied fails instead of passing unnoticed.
      built = Hook0::Client.new("http://127.0.0.1:1", "app-123", "token-xyz")
      policy = built.options.retry_policy
      applied = {
        "max_attempts" => policy.max_attempts,
        "max_attempts_cap" => Hook0::RetryPolicy::MAX_ATTEMPTS_CAP,
        "initial_backoff_ms" => policy.initial_backoff * 1000,
        "max_backoff_ms" => policy.max_backoff * 1000,
        "max_total_delay_ms" => policy.max_total_delay * 1000,
        "request_timeout_ms" => built.options.request_timeout * 1000,
        "max_payload_bytes" => built.options.max_payload_bytes,
        "max_response_bytes" => built.options.max_response_bytes,
        "max_response_headers" => built.options.max_response_headers,
        "max_header_bytes" => built.options.max_header_bytes
      }

      assert_empty BOUNDS.keys - applied.keys, "the corpus names a bound this client does not apply"
      BOUNDS.each { |name, wanted| assert_in_delta wanted, applied.fetch(name), 0.001, name }
    end

    def test_every_refusal_the_corpus_declares_reads_as_one_of_this_client_s
      # A refusal named in the corpus and mapped to nothing here would pass under any wording.
      assert_empty SIGNATURE["refusals"] - REFUSALS.keys
    end

    def test_every_delivery_of_the_corpus_is_verified_as_it_says
      # A refused delivery has to be refused for the reason the corpus names: a client that computed
      # a code over a header that never arrived and reported a mismatch would otherwise look right.
      SIGNATURE["vectors"].each do |vector|
        if vector["verdict"] == "accepted"
          assert_nil verified(vector), vector["reason"]
          next
        end

        refused = assert_raises(Hook0::ClientError, vector["name"]) { verified(vector) }

        assert_includes refused.message, REFUSALS.fetch(vector["refusal"]),
                        "a delivery the corpus refuses as `#{vector["refusal"]}` was answered " \
                        "`#{refused.message}`: #{vector["reason"]}"
      end
    end

    private

    # One cause of a request the API never answered, provoked over a real socket: how many requests
    # the send issued, and whether it ended up ingesting the event.
    def provoked(cause)
      case cause
      when "no_answer" then provoked_by_no_answer
      when "answer_above_a_bound" then provoked_by_an_answer_above_a_bound
      when "unusable_api_url" then provoked_by_an_unusable_api_url
      else raise "the corpus names a cause `#{cause}` this suite does not know how to provoke"
      end
    end

    # An attempt that runs out of time before the API writes anything.
    def provoked_by_no_answer
      restarted do
        @api.will_answer(ScriptedResponse.new(201, ingested(INGESTED_ID).body, 1.0), ingested(INGESTED_ID))
        issued_by { client(options(max_attempts: 4, request_timeout: 0.2)) }
      end
    end

    # An answer larger than what this client agreed to read off the socket.
    def provoked_by_an_answer_above_a_bound
      restarted do
        oversized = ScriptedResponse.new(201, { "event_id" => INGESTED_ID, "padding" => "x" * 2048 })
        @api.will_answer(oversized, ingested(INGESTED_ID))
        issued_by { client(options(max_attempts: 4, max_response_bytes: 256)) }
      end
    end

    # A base URL nothing can be sent to, which means nothing is ever sent.
    def provoked_by_an_unusable_api_url
      restarted do
        @api.will_answer(ingested(INGESTED_ID))
        issued_by do
          Hook0::Client.new("gopher://nowhere.invalid", "app-123", "token-xyz", options(max_attempts: 4))
        end
      end
    end

    # How many attempts a send made, and whether it ended up ingesting the event.
    #
    # A send that reached a server is counted by what that server received. One that never reached
    # anything — an API URL nothing can be sent to is the corpus's own example — is counted by what
    # the client says it did, which is also the message a caller is left holding: a misconfiguration
    # retried four times reads as a network that would not answer.
    def issued_by
      built = yield
      begin
        built.send_event(an_event)
        [@api.received.size, true]
      rescue Hook0::ClientError => e
        [[@api.received.size, attempts_of(e.message)].max, false]
      end
    end

    # What a send says it did, out of the message it failed with.
    def attempts_of(message)
      named = /gave up after (\d+) attempts/.match(message)
      named.nil? ? 1 : Integer(named[1], 10)
    end

    def verified(vector)
      Hook0.verify_webhook_signature_with_current_time(
        vector["signature"],
        vector["payload"],
        vector["headers"].map { |name, value| [name, value] },
        vector["secret"],
        vector["tolerance_seconds"].to_f,
        Time.at(vector["current_time"]).utc
      )
    end

    # What the API says when it refuses a request, in the shape every Hook0 failure takes.
    def refusal(status, problem, headers = {})
      ScriptedResponse.new(
        status,
        {
          "id" => problem,
          "status" => status,
          "title" => "refused",
          "detail" => "what the corpus scripted",
          "type" => "https://hook0.com/documentation/errors/#{problem}"
        },
        0.0,
        headers
      )
    end

    # How many requests a send made when the API answered that way and then took the event.
    #
    # One API per case rather than one per suite: what is counted is what this send issued, and a
    # count carried over from the case before it would say nothing.
    def issued_for(answer)
      restarted do
        @api.will_answer(answer, ingested(INGESTED_ID))
        begin
          client(options(max_attempts: 4)).send_event(an_event)
          [@api.received.size, true]
        rescue Hook0::ClientError
          [@api.received.size, false]
        end
      end
    end

    # How long a send spent waiting when the API named that delay beside a paced answer.
    def waited_for(delay)
      paced, = paced_pair
      restarted do
        @api.will_answer(
          refusal(paced["status"], paced["problem"], { RETRY["retry_after"]["header"] => delay["header"] }),
          ingested(INGESTED_ID)
        )
        chosen = Hook0::Options.new(
          retry_policy: Hook0::RetryPolicy.new(
            max_attempts: 4,
            initial_backoff: PROMPT_BACKOFF,
            max_backoff: PROMPT_BACKOFF,
            max_total_delay: DELAY_BUDGET
          ),
          request_timeout: 5.0
        )

        started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
        client(chosen).send_event(an_event)
        waited = Process.clock_gettime(Process::CLOCK_MONOTONIC) - started

        assert_equal 2, @api.received.size, "a paced answer was not retried"
        waited
      end
    end

    # Two problems answering the same status, one worth repeating and one not.
    #
    # That pair is the whole reason the corpus classifies problems rather than statuses, and the
    # retryable one is the answer the API names a delay beside.
    def paced_pair
      RETRY["problems"].each do |rule|
        next unless rule["retryable"]

        other = RETRY["problems"].find { |candidate| candidate["status"] == rule["status"] && !candidate["retryable"] }
        return [rule, other] unless other.nil?
      end
      raise "no status of the corpus carries opposite verdicts"
    end

    # One case of the corpus against an API of its own.
    def restarted
      @api.close
      @api = FakeApi.new
      yield
    end
  end
end
