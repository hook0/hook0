# frozen_string_literal: true

# What declaring the event types an application uses does over a real socket.
#
# The two requests a declaration makes — asking the API what the application already declares, and
# creating the ones it does not — each have their own ways of failing, and what a caller is told has
# to name the one it was making rather than the transport's own words.

require_relative "test_helper"

module Hook0Test
  class EventTypesTest < ApiCase
    def test_event_types_the_application_already_declares_are_not_created
      @api.will_answer(
        ScriptedResponse.new(200, [{ "event_type_name" => "auth.user.create" }]),
        ScriptedResponse.new(201, { "event_type_name" => "billing.invoice.paid" })
      )

      created = client.upsert_event_types(["auth.user.create", "billing.invoice.paid"])

      assert_equal ["billing.invoice.paid"], created
      assert_equal 2, @api.received.size
    end

    def test_an_entry_of_the_list_that_is_not_an_entry_at_all_is_passed_over
      @api.will_answer(
        ScriptedResponse.new(200, ["not an entry", { "event_type_name" => "auth.user.create" }]),
        ScriptedResponse.new(201, { "event_type_name" => "billing.invoice.paid" })
      )

      created = client.upsert_event_types(["auth.user.create", "billing.invoice.paid"])

      assert_equal ["billing.invoice.paid"], created
    end

    def test_upserting_no_event_type_at_all_reaches_the_api_for_nothing
      assert_empty client.upsert_event_types([])
      assert_empty @api.received
    end

    # Nothing is created off a list that was never read: taking a failure for an empty list would
    # have this client declare every event type the application already has.
    def test_event_types_the_api_refused_to_answer_are_reported_rather_than_taken_as_none
      @api.will_answer(ScriptedResponse.new(503, { "id" => "ServiceUnavailable", "status" => 503 }))

      refused = assert_raises(Hook0::ClientError) { client.upsert_event_types(["auth.user.create"]) }

      assert_includes refused.message, "Getting available event types failed"
      assert_equal 1, @api.received.size
    end

    # The answer crosses a ceiling this client set for itself, so no list ever arrives. What the

    # The answer crosses a ceiling this client set for itself, so no list ever arrives. What the
    # caller is told is what was being asked for, rather than the transport's own words.
    def test_a_list_of_event_types_the_api_could_not_answer_at_all_is_reported_as_the_ask_it_was
      @api.will_answer(ScriptedResponse.new(200, [{ "event_type_name" => "a" * 2048 }]))

      refused = assert_raises(Hook0::ClientError) do
        client(options(max_response_bytes: 256)).upsert_event_types(["auth.user.create"])
      end

      assert_includes refused.message, "Getting available event types failed"
    end

    def test_a_list_of_event_types_that_is_not_a_list_is_reported_rather_than_walked
      @api.will_answer(ScriptedResponse.new(200, { "event_type_name" => "auth.user.create" }))

      refused = assert_raises(Hook0::ClientError) { client.upsert_event_types(["auth.user.create"]) }

      assert_includes refused.message, "did not answer a list of event types"
    end

    # The application declares one of the two, under an entry beside one this client cannot read:

    # The application declares one of the two, under an entry beside one this client cannot read:
    # the one it can read is still matched, and the other is still created.
    def test_an_entry_of_the_list_naming_no_event_type_is_passed_over_rather_than_stopping_the_read
      @api.will_answer(
        ScriptedResponse.new(200, [{ "no name here" => true }, { "event_type_name" => "auth.user.create" }]),
        ScriptedResponse.new(201, { "event_type_name" => "billing.invoice.paid" })
      )

      created = client.upsert_event_types(["auth.user.create", "billing.invoice.paid"])

      assert_equal ["billing.invoice.paid"], created
    end

    def test_an_event_type_the_api_refused_to_create_is_reported_under_the_name_it_was_asked_for
      @api.will_answer(
        ScriptedResponse.new(200, []),
        ScriptedResponse.new(409, { "id" => "EventTypeAlreadyExist", "status" => 409 })
      )

      refused = assert_raises(Hook0::ClientError) { client.upsert_event_types(["auth.user.create"]) }

      assert_includes refused.message, "Creating event type 'auth.user.create' failed"
      assert_includes refused.message, "EventTypeAlreadyExist"
    end

    def test_an_event_type_the_api_never_answered_for_is_reported_under_the_name_it_was_asked_for
      @api.will_answer(
        ScriptedResponse.new(200, []),
        ScriptedResponse.new(201, { "padding" => "a" * 2048 })
      )

      refused = assert_raises(Hook0::ClientError) do
        client(options(max_response_bytes: 256)).upsert_event_types(["auth.user.create"])
      end

      assert_includes refused.message, "Creating event type 'auth.user.create' failed"
    end

    def test_an_event_type_that_does_not_read_as_three_parts_is_refused
      refused = assert_raises(Hook0::ClientError) { client.upsert_event_types(["not-an-event-type"]) }

      assert_includes refused.message, "does not have a valid syntax"
      assert_empty @api.received
    end
  end
end
