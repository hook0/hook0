# frozen_string_literal: true

# What a webhook has to carry to be accepted, and every way it can fail to.
#
# The signatures below are produced the way Hook0 produces them — an HMAC-SHA256 over the moment,
# the covered headers and the body — rather than read back from the code under test, so a case fails
# when that code changes what it computes over.

require "openssl"

require_relative "test_helper"

module Hook0Test
  class SignatureTest < Minitest::Test
    SECRET = "a-subscription-secret"
    PAYLOAD = '{"hello": "world"}'
    NOW = Time.utc(2026, 8, 14, 12, 0, 0)
    TOLERANCE = 300.0

    def body_code(timestamp, payload: PAYLOAD, secret: SECRET)
      code = OpenSSL::HMAC.new(secret, "SHA256")
      code << "#{timestamp}."
      code << payload
      code.hexdigest
    end

    def headers_code(timestamp, covered, payload: PAYLOAD, secret: SECRET)
      code = OpenSSL::HMAC.new(secret, "SHA256")
      code << "#{timestamp}."
      code << covered.map(&:first).join(" ")
      code << "."
      code << covered.map(&:last).join(".")
      code << "."
      code << payload
      code.hexdigest
    end

    def verify(signature, headers, current_time: NOW)
      Hook0.verify_webhook_signature_with_current_time(signature, PAYLOAD, headers, SECRET, TOLERANCE, current_time)
    end

    def test_a_v0_signature_over_the_body_is_accepted
      timestamp = NOW.to_i

      assert_nil verify("t=#{timestamp},v0=#{body_code(timestamp)}", {})
    end

    def test_a_v1_signature_over_the_covered_headers_is_accepted
      timestamp = NOW.to_i
      covered = [%w[x-event-id abc], %w[x-event-type auth.user.create]]
      signature = "t=#{timestamp},h=#{covered.map(&:first).join(" ")},v1=#{headers_code(timestamp, covered)}"

      assert_nil verify(signature, covered.to_h)
    end

    def test_the_covered_headers_are_read_in_the_order_the_signature_lists_them
      timestamp = NOW.to_i
      covered = [%w[x-event-id abc], %w[x-event-type auth.user.create]]
      # The values are signed in the order `h` lists the names, so the same two headers signed the
      # other way round is a different message and must not verify.
      swapped = covered.reverse
      signature = "t=#{timestamp},h=#{covered.map(&:first).join(" ")},v1=#{headers_code(timestamp, swapped)}"

      assert_raises(Hook0::ClientError) { verify(signature, covered.to_h) }
    end

    def test_the_delivered_headers_are_found_whatever_case_they_arrived_in
      timestamp = NOW.to_i
      covered = [%w[x-event-id abc]]
      signature = "t=#{timestamp},h=x-event-id,v1=#{headers_code(timestamp, covered)}"

      assert_nil verify(signature, { "X-Event-Id" => "abc" })
    end

    def test_a_v1_signature_wins_over_a_v0_one_that_would_have_verified
      timestamp = NOW.to_i
      # A sender offering both must not have the weaker of the two accepted on the strength of the
      # stronger one being wrong.
      elsewhere = headers_code(timestamp, [%w[x-event-id other]])
      signature = "t=#{timestamp},h=x-event-id,v0=#{body_code(timestamp)},v1=#{elsewhere}"

      assert_raises(Hook0::ClientError) { verify(signature, { "x-event-id" => "abc" }) }
    end

    def test_a_header_the_signature_covers_but_the_request_did_not_carry_is_refused
      timestamp = NOW.to_i
      covered = [%w[x-event-id abc]]
      signature = "t=#{timestamp},h=x-event-id,v1=#{headers_code(timestamp, covered)}"

      refused = assert_raises(Hook0::ClientError) { verify(signature, {}) }

      assert_includes refused.message, "was not delivered"
    end

    def test_a_code_that_is_not_whole_hexadecimal_is_refused_rather_than_truncated
      timestamp = NOW.to_i
      # A decoder that stops at the first bad character compares a prefix of the right code, which
      # is a signature anyone can produce.
      whole = body_code(timestamp)
      truncated = "#{whole[0, 20]}zz#{whole[22..]}"

      refused = assert_raises(Hook0::ClientError) { verify("t=#{timestamp},v0=#{truncated}", {}) }

      assert_includes refused.message, "not hexadecimal"
    end

    def test_a_code_of_odd_length_is_refused
      timestamp = NOW.to_i

      assert_raises(Hook0::ClientError) { verify("t=#{timestamp},v0=#{body_code(timestamp)[0..-2]}", {}) }
    end

    def test_a_signature_signed_too_long_ago_is_refused
      timestamp = (NOW - (TOLERANCE + 1)).to_i

      refused = assert_raises(Hook0::ClientError) { verify("t=#{timestamp},v0=#{body_code(timestamp)}", {}) }

      assert_includes refused.message, "outside the"
    end

    def test_a_signature_signed_too_far_in_the_future_is_refused
      timestamp = (NOW + (TOLERANCE + 1)).to_i

      # A window that only looks backwards is one a forged timestamp can widen without limit.
      refused = assert_raises(Hook0::ClientError) { verify("t=#{timestamp},v0=#{body_code(timestamp)}", {}) }

      assert_includes refused.message, "outside the"
    end

    def test_a_signature_at_the_edge_of_the_window_is_accepted_on_both_sides
      [NOW - TOLERANCE, NOW + TOLERANCE].each do |edge|
        timestamp = edge.to_i

        assert_nil verify("t=#{timestamp},v0=#{body_code(timestamp)}", {})
      end
    end

    def test_a_signature_carrying_no_moment_is_refused
      refused = assert_raises(Hook0::ClientError) { verify("v0=#{body_code(0)},h=", {}) }

      assert_includes refused.message, "`t`"
    end

    def test_a_signature_carrying_no_code_is_refused
      timestamp = NOW.to_i

      assert_raises(Hook0::ClientError) { verify("t=#{timestamp},h=x-event-id", { "x-event-id" => "abc" }) }
    end

    def test_a_moment_no_clock_could_hold_is_refused
      refused = assert_raises(Hook0::ClientError) { verify("t=#{"9" * 200},v0=#{body_code(0)}", {}) }

      assert_includes refused.message, "from the epoch"
    end

    def test_a_body_that_changed_after_it_was_signed_is_refused
      timestamp = NOW.to_i
      signature = "t=#{timestamp},v0=#{body_code(timestamp, payload: "something else")}"

      refused = assert_raises(Hook0::ClientError) { verify(signature, {}) }

      assert_includes refused.message, "does not match"
    end

    def test_a_signature_made_under_another_secret_is_refused
      timestamp = NOW.to_i

      assert_raises(Hook0::ClientError) do
        verify("t=#{timestamp},v0=#{body_code(timestamp, secret: "another-secret")}", {})
      end
    end

    def test_the_parts_of_a_signature_are_read_around_the_spaces_they_arrived_with
      timestamp = NOW.to_i

      assert_nil verify(" t = #{timestamp} , v0 = #{body_code(timestamp)} ", {})
    end

    def test_only_the_first_assignator_of_a_part_separates_its_name_from_its_value
      timestamp = NOW.to_i
      # `h=a=b` names one header, `a=b`, which is not a header name; reading it as `h=a` would let a
      # sender drop everything past the second assignator and keep the signature valid.
      signature = "t=#{timestamp},v0=#{body_code(timestamp)},h=a=b"

      assert_raises(Hook0::ClientError) { verify(signature, {}) }
    end

    def test_verifying_against_the_current_moment_accepts_a_signature_made_now
      timestamp = Time.now.to_i

      assert_nil Hook0.verify_webhook_signature(
        "t=#{timestamp},v0=#{body_code(timestamp)}",
        PAYLOAD,
        {},
        SECRET,
        TOLERANCE
      )
    end

    def test_headers_given_as_pairs_are_read_like_headers_given_as_a_mapping
      timestamp = NOW.to_i
      covered = [%w[x-event-id abc]]
      signature = "t=#{timestamp},h=x-event-id,v1=#{headers_code(timestamp, covered)}"

      assert_nil verify(signature, covered.to_h)
      assert_nil Hook0.verify_webhook_signature_with_current_time(signature, PAYLOAD, covered, SECRET, TOLERANCE, NOW)
    end
  end
end
