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
    REQUEST = Hook0Test.contract("request.json")

    INGESTED_ID = "01961234-5678-7abc-8def-0123456789ac"

    # The holes of a header value that are the same whichever client a case builds: the credential
    # every case uses, and the target reading the corpus.
    #
    # This is half of what a case binds. The other half is the retry policy the client under test
    # was built with, which differs per case and is merged in by `assert_carries` through
    # `Hook0Test.stated_by`. Reading this constant alone says nothing about how many holes are
    # filled.
    BOUND_IN_EVERY_CASE = { "token" => "token-xyz", "language" => "ruby" }.freeze

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
        "max_header_bytes" => built.options.max_header_bytes,
        "max_head_bytes" => built.options.max_head_bytes
      }

      assert_empty BOUNDS.keys - applied.keys, "the corpus names a bound this client does not apply"
      BOUNDS.each { |name, wanted| assert_in_delta wanted, applied.fetch(name), 0.001, name }
    end

    def test_every_occasion_a_header_names_is_one_the_corpus_declares
      # A `when` spelled differently from the occasion it means would match no request, and the case
      # below would then assert nothing about that header while still passing.
      REQUEST["headers"].each do |header|
        assert_includes REQUEST["occasions"], header["when"],
                        "the corpus carries `#{header["name"]}` on `#{header["when"]}`, " \
                        "which is not one of the occasions it declares"
      end
    end

    def test_a_request_carries_the_headers_its_occasion_declares_and_only_those
      # Read back off the socket, on both occasions the corpus declares: a send carries a body, and
      # a read does not. What separates them is the point — a client that sets `Content-Type` on a
      # request with nothing in it is describing a body that is not there.
      chosen = options(max_attempts: 4)
      @api.will_answer(ingested(INGESTED_ID))
      client(chosen).send_event(an_event)
      carrying_a_body = @api.received.fetch(0)

      restarted do
        @api.will_answer(ScriptedResponse.new(200, []))
        transport.request("GET", "/applications")
        carrying_nothing = @api.received.fetch(0)

        # The two are held to different policies on purpose: the send goes through a client this
        # case configured, and the read through a transport built with none, which states the
        # default. A header carrying one value on both is one that reports something other than the
        # policy behind the request.
        assert_carries carrying_a_body, ["every request", "a request carrying a body"], chosen.retry_policy
        assert_carries carrying_nothing, ["every request"], Hook0::RetryPolicy.new
      end
    end

    def test_a_client_asking_for_more_attempts_than_the_cap_states_the_cap
      # The contract pins the clamped reading: a policy asking for more attempts than anything may
      # make states the cap, because the cap is what its traffic will show and the number it asked
      # for would send a reader looking for a burst that cannot happen. The expected number is the
      # corpus's own rather than this client's, which is what keeps two SDKs from describing the
      # same setup differently — the disagreement is invisible until a policy crosses the cap.
      cap = BOUNDS.fetch("max_attempts_cap").to_i
      greedy = Hook0::RetryPolicy.new(max_attempts: cap * 100, initial_backoff: 0.0,
                                      max_backoff: 0.0, max_total_delay: 0.0)
      @api.will_answer(ingested(INGESTED_ID))
      client(Hook0::Options.new(retry_policy: greedy, request_timeout: 5.0)).send_event(an_event)

      stated = @api.received.fetch(0).headers["hook0-client-options"]

      assert_equal "attempts=#{cap},backoff=0,ceiling=0,budget=0", stated,
                   "a client asked for #{cap * 100} attempts and stated `#{stated}`, where the " \
                   "corpus caps what any policy may make at #{cap}"
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

    # The transport the generated half is handed, pointed at the API this case is listening on.
    def transport
      Hook0::Transport.new(@api.base_url, "token-xyz")
    end

    # What the corpus says a request on these occasions carries, held against what arrived.
    #
    # Both directions are checked: a header the occasion declares has to be there with the value the
    # corpus spells, and one it does not declare has to be absent. Only the second direction catches
    # a client that sets every header it knows on every request it makes.
    def assert_carries(request, occasions, policy)
      bound = BOUND_IN_EVERY_CASE.merge(Hook0Test.stated_by(policy))

      REQUEST["headers"].each do |header|
        name = header["name"].downcase
        carried = request.headers[name]

        unless occasions.include?(header["when"])
          assert_nil carried,
                     "a request carried `#{header["name"]}: #{carried}`, which the corpus carries " \
                     "only on `#{header["when"]}`: #{header["reason"]}"
          next
        end

        chunks = Hook0Test.template_chunks(header["value"], bound)

        assert Hook0Test.matches_chunks?(chunks, carried.to_s),
               "a request carried `#{header["name"]}: #{carried}` where the corpus says " \
               "`#{header["value"]}`: #{header["reason"]}"
        next if chunks.size == 1

        # A value with a hole this suite cannot fill is one the client composed out of what the
        # platform told it, and what the platform says is as long as it feels like.
        assert_operator carried.to_s.bytesize, :<=, REQUEST["max_composed_bytes"],
                        "a request carried #{carried.to_s.bytesize} bytes of `#{header["name"]}`, above the " \
                        "#{REQUEST["max_composed_bytes"]} the corpus cuts a composed value to"
      end
    end

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
