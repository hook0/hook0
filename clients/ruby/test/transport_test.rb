# frozen_string_literal: true

# Where a request lands, which is decided before a socket is opened.
#
# What a base URL and a path add up to is the one thing about a request this gem settles on its own:
# everything else is either what the caller asked for or what the API answered. A base that names
# nowhere is therefore refused rather than turned into a message accusing the network, and a query
# the path already carries is kept beside the pairs an operation adds to it.

require_relative "test_helper"

module Hook0Test
  class TransportTest < ApiCase
    def transport(base_url = @api.base_url)
      Hook0::Transport.new(base_url, "token-xyz")
    end

    def test_a_base_url_no_path_can_be_resolved_against_is_refused_before_anything_is_sent
      refused = assert_raises(Hook0::TransportError) { transport("not a base url").request("GET", "/event") }

      assert_equal "unusable_api_url", refused.cause_name
      refute_predicate refused, :retryable?, "building the same unusable request again cannot end differently"
      assert_empty @api.received
    end

    def test_a_base_url_naming_a_scheme_nothing_can_be_sent_over_is_refused
      refused = assert_raises(Hook0::TransportError) { transport("gopher://nowhere.invalid").request("GET", "/event") }

      assert_equal "unusable_api_url", refused.cause_name
      assert_includes refused.message, "not somewhere this transport can send a request"
    end

    def test_a_query_the_path_already_carries_is_kept_beside_the_pairs_an_operation_adds
      @api.will_answer(ScriptedResponse.new(200, []))

      transport.request("GET", "/api/v1/event_types?trace=1", [%w[application_id app-123]])

      assert_equal "/api/v1/event_types?trace=1&application_id=app-123", @api.received[0].target
    end

    def test_a_path_carrying_a_scheme_of_its_own_goes_there_rather_than_under_the_base
      # A caller handed an absolute URL — a link the API answered with, say — is sending to that URL.
      # Pasting it under the base would build a target neither of them names.
      @api.will_answer(ScriptedResponse.new(200, { "reached" => true }))

      status, payload = transport("http://nowhere.invalid/api/v1").request("GET", "#{@api.base_url}/somewhere-else")

      assert_equal 200, status
      assert_equal({ "reached" => true }, JSON.parse(payload))
      assert_equal "/somewhere-else", @api.received[0].target
    end
  end
end
