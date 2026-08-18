# frozen_string_literal: true

# What the generated request layer puts on the wire, and what it does with what comes back.
#
# The generated half is handed a transport and nothing else, so these cases hand it the real one and
# watch a real API answer: the path it filled in, the query it assembled, the credential it carried,
# the type it read back, and the exception it raised when the answer was a problem.
#
# Nothing here names an operation the API does not declare. What is exercised is reached through the
# generated namespace, so an operation that stops being declared takes its case with it rather than
# leaving one that runs against nothing.

require_relative "api_document"
require_relative "generated_surface"
require_relative "test_helper"

module Hook0Test
  class GeneratedTest < ApiCase
    APPLICATION_ID = "3f2504e0-4f89-41d3-9a0c-0305e82c3301"
    ORGANIZATION_ID = "3f2504e0-4f89-41d3-9a0c-0305e82c3302"

    # Every name the generator declared, under what declared it: the members of each value and the
    # methods of each operation group, discovered rather than listed.
    def declared_by_the_generator
      Hook0::Generated.constants.filter_map do |named|
        declared = Hook0::Generated.const_get(named)
        next unless declared.is_a?(Class)

        [declared, declared.instance_methods(false)]
      end
    end

    def applications
      Hook0::Generated::ApplicationsApi.new(Hook0::Transport.new(@api.base_url, "token-xyz"))
    end

    def an_application
      {
        "application_id" => APPLICATION_ID,
        "name" => "an application",
        "organization_id" => ORGANIZATION_ID,
        "consumption" => { "events_per_day" => 12 },
        "onboarding_steps" => { "event" => "Done", "event_type" => "ToDo", "subscription" => "ToDo" },
        "quotas" => { "days_of_events_retention_limit" => 7, "events_per_day_limit" => 100 }
      }
    end

    def a_problem
      {
        "id" => "NotFound",
        "title" => "Not found",
        "detail" => "This application does not exist.",
        "status" => 404,
        "type" => "https://documentation.hook0.com/problems"
      }
    end

    def test_a_generated_method_reads_back_the_type_the_api_declares
      @api.will_answer(ScriptedResponse.new(200, an_application))

      application = applications.get(APPLICATION_ID)

      assert_instance_of Hook0::Generated::ApplicationInfo, application
      assert_equal APPLICATION_ID, application.application_id
      assert_equal 100, application.quotas.events_per_day_limit
      assert_equal "Done", application.onboarding_steps.event
    end

    def test_a_generated_method_fills_the_path_and_carries_the_credential
      @api.will_answer(ScriptedResponse.new(200, an_application))

      applications.get(APPLICATION_ID)

      request = @api.received[0]

      assert_equal "GET", request.verb
      # The identifier lands in the path rather than staying as the placeholder that named it.
      assert_equal "/api/v1/applications/#{APPLICATION_ID}", request.target
      assert_equal "Bearer token-xyz", request.headers["authorization"]
    end

    def test_a_generated_method_assembles_the_query_the_operation_declares
      @api.will_answer(ScriptedResponse.new(200, []))

      assert_empty applications.list(ORGANIZATION_ID)
      assert_equal "/api/v1/applications/?organization_id=#{ORGANIZATION_ID}", @api.received[0].target
    end

    def test_a_generated_method_reads_a_list_of_the_type_the_api_declares
      @api.will_answer(
        ScriptedResponse.new(
          200,
          [{ "application_id" => APPLICATION_ID, "name" => "an application", "organization_id" => ORGANIZATION_ID }]
        )
      )

      listed = applications.list(ORGANIZATION_ID)

      assert_equal 1, listed.size
      assert_equal "an application", listed[0].name
    end

    def test_a_problem_the_api_reports_is_raised_as_the_exception_it_names
      @api.will_answer(ScriptedResponse.new(404, a_problem))

      reported = assert_raises(Hook0::Generated::NotFoundError) { applications.get(APPLICATION_ID) }

      assert_equal 404, reported.status
      assert_equal "This application does not exist.", reported.problem.detail
      # Every problem is a kind of the one exception a caller may rescue instead of naming each.
      assert_kind_of Hook0::Generated::ProblemError, reported
    end

    def test_a_failure_that_is_not_a_problem_document_is_still_reported
      @api.will_answer(ScriptedResponse.new(502, "a gateway wrote this, and it is not a problem document"))

      reported = assert_raises(Hook0::Generated::ProblemError) { applications.get(APPLICATION_ID) }

      assert_equal 502, reported.status
      assert_nil reported.problem
    end

    def test_a_member_the_document_does_not_require_is_absent_rather_than_written_back_as_nothing
      problem = Hook0::Generated::Problem.from_json(a_problem)

      refute_includes problem.to_h, "validation"
      assert_equal a_problem, problem.to_h
    end

    def test_a_document_that_does_not_say_what_it_declares_stops_the_read
      unreadable = an_application.merge("name" => 42)

      assert_raises(Hook0::Runtime::DecodeError) { Hook0::Generated::ApplicationInfo.from_json(unreadable) }
    end

    def test_a_value_outside_a_closed_list_is_refused_rather_than_carried
      unreadable = an_application.merge("onboarding_steps" => { "event" => "Maybe", "event_type" => "ToDo",
                                                                "subscription" => "ToDo" })

      assert_raises(Hook0::Runtime::DecodeError) { Hook0::Generated::ApplicationInfo.from_json(unreadable) }
    end

    def test_nothing_the_generator_declared_replaces_what_every_object_already_answers_to
      # A member or a method spelled like one of `Object`'s own takes the place of it: a value whose
      # `hash` is a string it happens to carry cannot key a hash, and a group whose `class` names an
      # operation no longer says what it is. `json` is loaded here as it is wherever the gem runs,
      # so `to_json` counts too. Keywords are deliberately absent: `def next` is a name Ruby accepts
      # and a caller can write, and escaping it would be a wart rather than a fix.
      # What the emitter declares on purpose is the exception, and only these: a value object that
      # overrode `==` without `eql?` and `hash` would be one two equal instances could not key a
      # hash with, so those three are the contract rather than a collision.
      deliberate = %w[== eql? hash]
      answered = Object.new.methods.map(&:to_s) - deliberate

      declared_by_the_generator.each do |owner, names|
        (names.map(&:to_s) & answered).each do |taken|
          flunk "#{owner} declares `#{taken}`, which every object already answers to"
        end
      end
    end

    def test_the_generated_surface_is_reachable_under_the_names_the_api_spells
      # The wire name is what travels; the Ruby name is only how it is written here. A member the
      # language forced out of the way says so with a trailing underscore and nothing else.
      status = Hook0::Generated::RequestAttemptStatus.instance_methods(false).map(&:to_s)

      assert_includes status, "until_", "a member named after a keyword is spelled out of its way"
      refute_includes status, "until"

      target = Hook0::Generated::SubscriptionTarget.instance_methods(false).map(&:to_s)

      assert_includes target, "method_", "a member named after an Object method is spelled out of its way"
      refute_includes target, "method"

      # The wire is unmoved either way: what Ruby had to rename travels under the name the API
      # spells, which is the whole point of renaming only the Ruby side.
      written = Hook0::Generated::RequestAttemptStatus.from_json(
        { "type" => "waiting", "since" => "2026-08-14T12:00:00Z", "until" => "2026-08-14T12:05:00Z" }
      ).to_h

      assert_includes written, "until"
      refute_includes written, "until_"

      problem = Hook0::Generated::Problem.instance_methods(false).map(&:to_s)

      assert_includes problem, "id", "`id` is not a method of Object and must not be renamed"
      assert_includes problem, "type", "`type` is not a method of Object and must not be renamed"
    end

    def test_every_problem_the_catalogue_names_is_an_exception_of_its_own
      catalogue = Hook0::Generated::PROBLEMS

      assert_equal Hook0::Generated::ProblemId::VALUES.sort, catalogue.keys.sort
      assert_equal catalogue.values.uniq.size, catalogue.size
      catalogue.each_value { |declared| assert_operator declared, :<, Hook0::Generated::ProblemError }
    end

    # A value written out and read back in is the value it started as, member for member.
    #
    # Run once with every member the schema may leave out set and once with none of them, which is
    # what tells a member that was read apart from one that was defaulted to the same thing.
    def test_every_schema_the_document_declares_reads_back_what_it_wrote
      [true, false].each do |optionals|
        GeneratedSurface.models.each do |declared|
          held = GeneratedSurface.model(declared, optionals)
          written = held.to_h

          assert_kind_of Hash, written, "#{declared} does not write itself as a JSON object"
          assert_equal held, declared.from_json(written), "#{declared} does not read back what it wrote"
          assert_equal written, declared.from_json(written).to_h, "#{declared} does not write back what it read"

          unset_of(declared, held).each do |absent|
            refute_includes written, absent, "#{declared} wrote `#{absent}` out although it holds nothing"
          end

          # A value object overriding `==` without `eql?` and `hash` is one two equal instances
          # cannot key a hash with, which is why the generator writes all three.
          other = declared.from_json(written)

          assert_equal held.hash, other.hash, "#{declared} gives two equal values two different keys"
          assert_equal 1, { held => 1 }.fetch(other), "#{declared} cannot key a hash"
        end
      end
    end

    def test_a_schema_refuses_a_document_that_is_not_the_object_it_declares
      GeneratedSurface.models.each do |declared|
        [1, "text", true, [], nil].each do |answered|
          refused = assert_raises(Hook0::Runtime::DecodeError) { declared.from_json(answered) }

          assert_includes refused.message, declared.name.split("::").last,
                          "#{declared} did not say what it was that could not be read"
        end
      end
    end

    # Everything a schema describes is refused when it arrives as something else, and says which
    # member it was. What it leaves undescribed is kept as it arrived and so is refused by nothing,
    # and there are exactly as many members that accept anything as it leaves undescribed.
    def test_a_schema_refuses_a_member_the_document_does_not_declare_it_as
      GeneratedSurface.models.each do |declared|
        written = GeneratedSurface.model(declared, true).to_h
        accepted = written.keys.reject { |name| refuses?(declared, written, name) }

        assert_equal opaque_count(declared), accepted.size,
                     "#{declared} read #{accepted.join(", ")} although it describes what they hold"
      end
    end

    # Which members carry a list is read off what each value wrote rather than named here, so a
    # schema that grows one is held to this the moment it does.
    def test_a_member_that_carries_a_list_refuses_a_document_that_is_not_one
      walked = 0
      GeneratedSurface.models.each do |declared|
        written = GeneratedSurface.model(declared, true).to_h
        written.each do |name, value|
          next unless value.is_a?(Array)

          refused = assert_raises(Hook0::Runtime::DecodeError) do
            declared.from_json(written.merge(name => "not a list at all"))
          end

          assert_includes refused.message, "expected an array"
          assert_includes refused.message, name
          walked += 1
        end
      end

      assert_operator walked, :>, 0, "no value the API declares carries a list"
    end

    # Which members carry a moment is not named here: it is read off what each value wrote, so a
    # schema that grows one is held to this the moment it does.
    def test_a_moment_is_refused_when_it_names_no_moment_and_when_it_names_no_day
      walked = 0
      GeneratedSurface.models.each do |declared|
        written = GeneratedSurface.model(declared, true).to_h
        written.each do |name, value|
          whole = value.is_a?(String) && value.match?(/\A\d{4}-\d{2}-\d{2}T/)
          day = value.is_a?(String) && value.match?(/\A\d{4}-\d{2}-\d{2}\z/)
          next unless whole || day

          # A day that never existed is refused where the reader can tell — `Date.iso8601` does,
          # and `Time.iso8601` rolls `2026-02-31` forward to March instead of refusing it.
          wrong_ones = whole ? ["not a moment at all"] : ["not a moment at all", "2026-02-31"]
          wrong_ones.each do |wrong|
            refused = assert_raises(Hook0::Runtime::DecodeError) { declared.from_json(written.merge(name => wrong)) }

            assert_includes refused.message, name
            walked += 1
          end
        end
      end

      assert_operator walked, :>, 0, "no value the API declares carries a moment"
    end

    # A document that is nothing but brackets would grow the stack of whatever reads it, so how deep
    # one may nest is bounded and a deeper one is refused rather than parsed.
    def test_a_success_nested_deeper_than_this_gem_reads_is_refused_rather_than_parsed
      deep = Hook0::Runtime::MAX_PAYLOAD_NESTING + 4
      @api.will_answer(ScriptedResponse.new(200, deep.times.reduce("bottom") { |held, _| [held] }))

      refused = assert_raises(Hook0::Runtime::DecodeError) { applications.get(APPLICATION_ID) }

      assert_includes refused.message, "the response is not JSON"
    end

    def test_every_closed_list_reads_the_values_it_declares_and_no_other
      GeneratedSurface.enumerations.each do |declared|
        refute_empty declared::VALUES, "#{declared} declares no value at all"
        declared::VALUES.each do |value|
          assert declared.member?(value), "#{declared} does not declare `#{value}`, which it lists"
        end
        refute declared.member?("a value the API never declared")
      end
    end

    # A caller may rescue the one problem it cares about, or every problem, and both work.
    def test_every_problem_the_catalogue_names_is_raised_as_the_failure_it_is
      catalogue = Hook0::Generated::PROBLEMS

      refute_empty catalogue, "the generator mapped no problem at all"
      catalogue.each do |problem, expected|
        @api.will_answer(
          ScriptedResponse.new(400, { "id" => problem, "status" => 400, "title" => "refused",
                                      "detail" => "what this case scripted",
                                      "type" => "https://hook0.com/documentation/errors/#{problem}" })
        )

        reported = assert_raises(expected) { applications.get(APPLICATION_ID) }

        assert_equal 400, reported.status
        assert_equal problem, reported.problem.id
        assert_equal "what this case scripted", reported.problem.detail
        assert_kind_of Hook0::Generated::ProblemError, reported
      end
    end

    # What a failure carries is written by a server this gem does not control and is cut to a budget
    # in bytes, which lands wherever it lands inside a character. What is reported is held to being
    # text all the same, rather than travelling half a character into what a caller logs.
    def test_a_failure_too_long_to_report_whole_is_cut_without_breaking_a_character_in_two
      answered = "#{"e" * (Hook0::Runtime::MAX_PREVIEW_BYTES - 1)}éclat, and not a problem document"
      @api.will_answer(ScriptedResponse.new(500, answered))

      reported = assert_raises(Hook0::Generated::ProblemError) { applications.get(APPLICATION_ID) }

      assert_equal 500, reported.status
      assert_nil reported.problem
      assert reported.message.valid_encoding?, "the report is not text"
      assert_includes reported.message, "…", "the report was not cut"
      refute_includes reported.message, "éclat"
    end

    private

    # Every member of a value that holds nothing, which is every one the case left out.
    def unset_of(declared, held)
      GeneratedSurface.wire_names_of(declared.instance_method(:initialize)).filter_map do |name, wire|
        wire if held.public_send(name).nil?
      end
    end

    # Whether a member answered as neither an object nor a scalar any reader accepts stops the read.
    def refuses?(declared, written, name)
      declared.from_json(written.merge(name => [{ "neither" => "a scalar" }]))
      false
    rescue Hook0::Runtime::DecodeError => e
      assert_includes e.message, name, "#{declared} did not say which member it could not read"
      true
    end

    # How many members of a value the document describes nothing about.
    def opaque_count(declared)
      GeneratedSurface.params_of(declared.instance_method(:initialize))
                      .count { |_, documented| documented.type.start_with?("Object") }
    end
  end
end
