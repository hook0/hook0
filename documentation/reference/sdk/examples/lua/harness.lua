-- The rest of the file, for every Lua example of the SDK reference.
--
-- A snippet on a page is written for a reader: some are complete, copy-paste-able scripts that
-- `require("hook0")` themselves; the rest are body fragments that assume `Hook0`, a client or an
-- event already exist, the way the surrounding prose describes them. Each region below is the file
-- that snippet would live in, with a hole where it goes. The page points at one by name on the
-- fence, so what a snippet is standing on is one word away from the snippet itself.
--
-- Where a region's tail reaches for a name the snippet only assigned (`return event_id`, `return
-- client`), that is not filler: luacheck flags a local nothing reads afterward, and a documentation
-- example that assigns a value only to show what a call returns is exactly that, on purpose. The
-- tail is this file's way of reading it, the way a caller's next line would.

-- HARNESS send
local application_id = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"
local token = "a-service-token"

EXAMPLE

return event_id
-- END HARNESS

-- HARNESS event
-- The value the page shows, held so it stays reachable rather than being read as a plain unused
-- literal.
local event =
  EXAMPLE

return event
-- END HARNESS

-- HARNESS bounds
local Hook0 = require("hook0")
local application_id = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"
local token = "a-service-token"

EXAMPLE

return client
-- END HARNESS

-- HARNESS verify
local Hook0 = require("hook0")

-- A stand-in for whatever request table a caller's own server framework hands it; the page shows
-- the shape any of them can be read through, not a table this client declares.
local request = {
  headers = { ["x-hook0-signature"] = "t=1700000000,v1=deadbeef" },
  body = "{}",
}
local subscription_secret = "a-subscription-secret"

EXAMPLE
-- END HARNESS

-- HARNESS match
local Hook0 = require("hook0")

local client = Hook0.Client.new(
  "https://app.hook0.com/api/v1", "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21", "a-service-token")
local event = {
  event_type = "billing.invoice.paid",
  payload = '{"invoice": "in_123"}',
  payload_content_type = "application/json",
}

EXAMPLE
-- END HARNESS

-- HARNESS upsert
local Hook0 = require("hook0")

local client = Hook0.Client.new(
  "https://app.hook0.com/api/v1", "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21", "a-service-token")

EXAMPLE

return created
-- END HARNESS

-- HARNESS api
local Hook0 = require("hook0")

local client = Hook0.Client.new(
  "https://app.hook0.com/api/v1", "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21", "a-service-token")
local application_id = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"

EXAMPLE

return secrets
-- END HARNESS
