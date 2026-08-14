# frozen_string_literal: true

# What holds for every input, rather than for the ones a case happened to pick.
#
# Four things are checked here. A retry schedule never spends more than the policy that produced it
# allows, whichever way the randomness fell. Reading a signature header answers with the one failure
# this gem declares, whatever text reached the endpoint, and never with anything else. A value read
# out of a document the API could answer is written back as the value that was read. And identifiers
# minted in sequence never go back in time.
#
# There is no `hypothesis`-grade tool in Ruby's standard library, and this gem installs nothing at
# runtime, so the search is written here: a fixed seed, a bounded number of draws, and the
# counter-examples worth keeping committed under `regressions/` so they run as ordinary cases on
# every pipeline. A failing draw is one somebody can reproduce by running the suite again rather
# than one that goes away on a retry.

require_relative "test_helper"

module Hook0Test
  class PropertyTest < Minitest::Test
    # What the draws are made from. Fixed, so the suite explores the same inputs everywhere it runs.
    SEED = 20_260_814

    # How many draws each property makes. Bounded, so a pipeline can never be held by one.
    DRAWS = 200

    # How far two sums of the same floats may sit apart before the difference is a defect rather
    # than the order they were added in.
    ROUNDING = 1e-9

    # The bounds a drawn policy is built inside.
    MAX_DRAWN_ATTEMPTS = 64
    MAX_DRAWN_SECONDS = 10.0
    MAX_DRAWN_BUDGET = 60.0

    # Longest header a draw builds.
    MAX_DRAWN_HEADER = 96

    def setup
      @random = Random.new(SEED)
    end

    def test_a_retry_schedule_stays_within_every_bound_of_its_policy
      cases = Hook0Test.corpus("retry_policies") + Array.new(DRAWS) { drawn_policy }

      cases.each do |max_attempts, initial_backoff, max_backoff, max_total_delay, draws|
        policy = Hook0::RetryPolicy.new(
          max_attempts: max_attempts,
          initial_backoff: initial_backoff,
          max_backoff: max_backoff,
          max_total_delay: max_total_delay
        )
        holds_for(policy, draws.map { |drawn| unusable(drawn) })
      end
    end

    def test_reading_a_signature_answers_with_the_one_failure_this_gem_declares
      headers = Hook0Test.corpus("signatures") + Array.new(DRAWS) { drawn_header }

      headers.each do |header|
        begin
          Hook0::Signature.parse(header)
        rescue Hook0::ClientError
          next
        end

        # Parsing answered, so verifying has to answer the same way: a header that reads must not
        # find a way to fail that a caller cannot name.
        begin
          Hook0.verify_webhook_signature_with_current_time(header, "", {}, "secret", 300.0, Time.at(0))
        rescue Hook0::ClientError
          next
        end
      end
    end

    def test_a_generated_type_reads_back_what_it_wrote
      documents = Hook0Test.corpus("documents")
      drawn = documents.flat_map { |document| Array.new(4) { mutated(document) } }

      declared_models.each do |model|
        (documents + drawn).each do |document|
          begin
            read = model.from_json(document)
          rescue Hook0::Runtime::DecodeError
            next
          end

          written = read.to_h

          assert_equal read, model.from_json(written), "a #{model} read out of #{document} does not read back"
          assert_equal written, model.from_json(written).to_h
        end
      end
    end

    def test_reading_a_document_answers_with_the_one_failure_the_runtime_declares
      documents = Hook0Test.corpus("documents").flat_map { |document| Array.new(4) { mutated(document) } }

      declared_models.each do |model|
        documents.each do |document|
          model.from_json(document)
        rescue Hook0::Runtime::DecodeError
          next
        end
      end
    end

    def test_minted_identifiers_carry_a_moment_that_never_goes_back
      moments = Array.new(DRAWS) { Hook0.generate_event_id }.map { |id| id[0, 8] + id[9, 4] }

      assert_equal moments.sort, moments
    end

    private

    # Every class the generator wrote, found by looking at what it wrote.
    #
    # Nothing lists the types here: a schema the document adds joins this suite the moment the
    # generated files carry it.
    def declared_models
      Hook0::Generated.constants.map { |named| Hook0::Generated.const_get(named) }
                      .select { |declared| declared.is_a?(Class) && declared.respond_to?(:from_json) }
                      .sort_by(&:name)
    end

    def holds_for(policy, draws)
      delays = policy.delays(draws)
      budget = [policy.max_total_delay, 0.0].max

      assert_operator policy.attempts, :>=, 1
      assert_operator policy.attempts, :<=, Hook0::RetryPolicy::MAX_ATTEMPTS_CAP
      assert_operator delays.size, :<=, policy.attempts - 1
      assert_operator delays.sum, :<=, budget + ROUNDING

      delays.each_with_index do |delay, index|
        assert_operator delay, :>=, 0.0
        assert_operator delay, :<=, policy.backoff_ceiling(index + 1) + ROUNDING
        assert_operator delay, :<=, [policy.max_backoff, 0.0].max + ROUNDING
      end

      # A schedule never hurries up as it goes: the ceiling of a retry never sits below the one
      # before it.
      ceilings = (1..policy.attempts).map { |retry_number| policy.backoff_ceiling(retry_number) }

      assert_equal ceilings.sort, ceilings
    end

    # A draw that is no draw at all, which has to make the client wait longer rather than less.
    def unusable(drawn)
      case drawn
      when "nan" then Float::NAN
      when "infinity" then Float::INFINITY
      when "-infinity" then -Float::INFINITY
      else drawn
      end
    end

    def drawn_policy
      [
        @random.rand(-4..MAX_DRAWN_ATTEMPTS),
        @random.rand * MAX_DRAWN_SECONDS,
        @random.rand * MAX_DRAWN_SECONDS,
        @random.rand * MAX_DRAWN_BUDGET,
        Array.new(@random.rand(0..8)) { (@random.rand * 2) - 0.5 }
      ]
    end

    # A header built out of the pieces a signature is made of, put together every way a sender that
    # is not Hook0 might put them together.
    def drawn_header
      pieces = %w[t v0 v1 h = , 0 9 zz abc x-event-id 1800000000 -1 " " . { }]
      Array.new(@random.rand(0..MAX_DRAWN_HEADER)) { pieces.sample(random: @random) }.join
    end

    # A document with one of its members taken away, replaced by something of another type, or
    # buried inside something else.
    def mutated(document)
      return document unless document.is_a?(Hash) && !document.empty?

      key = document.keys.sample(random: @random)
      case @random.rand(0..3)
      when 0 then document.reject { |name, _| name == key }
      when 1 then document.merge(key => @random.rand(0..1_000))
      when 2 then document.merge(key => [document[key]])
      else document.merge(key => nil)
      end
    end
  end
end
