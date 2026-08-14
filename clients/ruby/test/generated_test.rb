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
  end
end
