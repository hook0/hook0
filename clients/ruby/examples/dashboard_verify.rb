# frozen_string_literal: true

# What the dashboard shows under "Verify a webhook", for Ruby.
#
# Sending is only half of what a reader has come to do, and it is the easier half. This is the one
# the SDK reference calls the half most often got wrong by hand, so the dashboard shows it beside
# the send rather than leaving it to be found later.
#
# The secret is read from the environment on purpose. The dashboard cannot know which subscription
# a reader means — outside the onboarding it loads none, and an application may have several — so
# it points at the subscription instead of guessing one, and no second secret is put on screen.
#
# Read the markers as in `dashboard_send.rb`: `hook0:snippet` is what is displayed, everything
# outside it is what holds this file against the client.

# hook0:snippet:begin
require "hook0"

# Verify against the *raw* body: one that has been parsed and serialised again no longer hashes to
# what was signed. The tolerance is bilateral, so a delivery dated too far ahead is refused exactly
# like one dated too far behind.
def accept?(signature, body, headers)
  # The secret of the subscription being verified, which the dashboard links to rather than prints:
  # it cannot know which subscription a reader means, and an application may have several. Fetched
  # without a default, so a variable nobody exported raises here naming itself, and one exported
  # empty raises on the line below: both hash every genuine delivery to the wrong code, and each
  # one comes back refused as forged.
  secret = ENV.fetch("HOOK0_SUBSCRIPTION_SECRET")
  raise "HOOK0_SUBSCRIPTION_SECRET is not set" if secret.empty?

  Hook0.verify_webhook_signature(signature, body, headers, secret, 300)
  true
rescue Hook0::ClientError
  false
end
# hook0:snippet:end

# Nothing here is ever run by the pipeline: this file exists to be held against the real client.
accepted = accept?("", "", { "x-hook0-signature" => "" })
puts "accepted: #{accepted}"
