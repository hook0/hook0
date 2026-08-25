--- Hook0 from Lua: send events, declare the event types an application uses, verify the signature of
--- an incoming webhook, and call every operation the API declares.
---
--- The rock is two halves. What the API declares — one table per schema, one table of constants per
--- closed list of strings, one error kind per problem, one method per operation — is generated from
--- the OpenAPI snapshot the API commits, and lands under `hook0.generated`. What it does not declare
--- — how a request reaches the network, how a send is retried, how a signature is verified — is
--- hand-written beside it and never regenerated.
---
--- ```lua
--- local Hook0 = require("hook0")
---
--- local client = Hook0.Client.new("https://app.hook0.com/api/v1", application_id, token)
--- local event_id = client:send_event({
---   event_type = "billing.invoice.created",
---   payload = '{"invoice":"in_1"}',
---   payload_content_type = "application/json",
---   labels = { environment = "production" },
--- })
--- ```

local Client = require("hook0.client")
local Errors = require("hook0.errors")
local Generated = require("hook0.generated.all")
local Json = require("hook0.json")
local Runtime = require("hook0.runtime")
local Sha256 = require("hook0.sha256")
local Signature = require("hook0.signature")
local Transport = require("hook0.transport")

local Hook0 = {}

--- What this rock is released as, which the rockspec is held against rather than repeats.
Hook0.VERSION = "2.0.2"

Hook0.Client = Client
Hook0.Options = Client.Options
Hook0.RetryPolicy = Client.RetryPolicy
Hook0.EventType = Client.EventType
Hook0.generate_event_id = Client.generate_event_id

Hook0.Transport = Transport
Hook0.Signature = Signature
Hook0.Sha256 = Sha256
Hook0.Json = Json
Hook0.Runtime = Runtime

--- Every failure this client raises is a kind of this one.
Hook0.ClientError = Errors.ClientError

--- A request the API never answered, whichever of the causes it was.
Hook0.TransportError = Transport.TransportError

--- What the API answered is not what it declares it answers.
Hook0.DecodeError = Runtime.DecodeError

--- Whether a raised value is of that kind, or of a kind deriving from it.
Hook0.is = Errors.is

--- What to say about any raised value, whether or not this client raised it.
Hook0.message = Errors.message

--- Declares a kind of failure, which is what the generated half declares its problems under.
Hook0.kind = Errors.kind

--- Everything the API declares: `models`, `errors` and `api`.
Hook0.Generated = Generated

--- Verifies a webhook against the current moment.
Hook0.verify_webhook_signature = Signature.verify

--- Verifies a webhook against a moment the caller names.
Hook0.verify_webhook_signature_with_current_time = Signature.verify_with_current_time

--- Every operation group the API declares, built on one transport.
---
--- @param transport table what one request is issued through, such as `client.transport`
--- @return table one built group per entity, under the name the generator gave it
function Hook0.api(transport)
  local built = {}
  for name, group in pairs(Generated.api) do
    built[name] = group.new(transport)
  end
  return built
end

return Hook0
