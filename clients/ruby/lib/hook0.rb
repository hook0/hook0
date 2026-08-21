# frozen_string_literal: true

require_relative "hook0/version"
require_relative "hook0/errors"
require_relative "hook0/runtime"
require_relative "hook0/transport"
require_relative "hook0/signature"
require_relative "hook0/client"
require_relative "hook0/generated/all"

# The Ruby SDK for Hook0.
#
# Two halves live here. This one is hand-written: sending an event, upserting the event types an
# application uses, and verifying that a webhook came from Hook0 unchanged. The other is generated
# from the OpenAPI snapshot the API commits — one class per schema it declares, one exception per
# problem it can report, one method per operation — and is reached through {Hook0::Generated}, over
# the transport this half exports.
#
# The gem reaches the network, verifies signatures and decodes what the API answers with the
# standard library alone, so installing it never drags a transitive dependency into an application
# that only wanted to send an event.
module Hook0
end
