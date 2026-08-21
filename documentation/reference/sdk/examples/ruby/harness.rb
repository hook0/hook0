# The rest of the file, for every Ruby example of the SDK reference.
#
# A snippet on a page is written for a reader: it leaves out the `require` it already showed higher
# up the page, it assumes a client or an event is already built, and it names an application id, a
# token or a secret without saying where it came from. Each region below is the file that snippet
# would live in, with a hole where it goes. The page points at one by name on the fence, so what a
# snippet is standing on is one word away from the snippet itself.
#
# `ruby -c` reads a file whole without running it, so a region only has to parse — a name it never
# resolves does not have to exist. It is still written the way a working file would be, because a
# harness that drifted from the client would stop saying anything true about the page next to it,
# even if nothing here would fail on that account today.

# HARNESS gemfile
EXAMPLE

# END HARNESS

# HARNESS send
application_id = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"
token = "a-service-token"

EXAMPLE

# END HARNESS

# HARNESS event
event =
  EXAMPLE

# END HARNESS

# HARNESS options
application_id = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"
token = "a-service-token"

EXAMPLE

# END HARNESS

# HARNESS verify
subscription_secret = "a-subscription-secret"

# What the page calls `request`: a stand-in carrying the two things a verification reads off one,
# the same shape a Rack or Rails request carries them in.
Request = Struct.new(:headers, :body)
request = Request.new({ "X-Hook0-Signature" => "t=1700000000,v1=deadbeef" }, '{"invoice": "in_123"}')

EXAMPLE

# END HARNESS

# HARNESS rails
EXAMPLE

# END HARNESS

# HARNESS upsert
client = Hook0::Client.new("https://app.hook0.com/api/v1", "app-id", "token")

EXAMPLE

# END HARNESS

# HARNESS rest
token = "a-service-token"
application_id = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"

EXAMPLE

# END HARNESS

# HARNESS errors
require "logger"

logger = Logger.new($stdout)
client = Hook0::Client.new("https://app.hook0.com/api/v1", "app-id", "token")
event = Hook0::Event.new(
  event_type: "billing.invoice.paid",
  payload: '{"invoice": "in_123"}',
  payload_content_type: "application/json"
)

EXAMPLE

# END HARNESS
