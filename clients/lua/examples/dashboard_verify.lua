--- What the dashboard shows under "Verify a webhook", for Lua.
---
--- Sending is only half of what a reader has come to do, and it is the easier half. This is the one
--- the SDK reference calls the half most often got wrong by hand, so the dashboard shows it beside
--- the send rather than leaving it to be found later.
---
--- The secret is read from the environment on purpose. The dashboard cannot know which subscription
--- a reader means — outside the onboarding it loads none, and an application may have several — so it
--- points at the subscription instead of guessing one, and no second secret is put on screen.
---
--- Read the markers as in `dashboard_send.lua`: `hook0:snippet` is what is displayed, everything
--- outside it is what makes the file lint.

-- hook0:snippet:begin
local Hook0 = require("hook0")

--- Verify against the *raw* body: one that has been parsed and serialised again no longer hashes to
--- what was signed. Verification raises rather than answering a flag, so it is wrapped in `pcall`.
--- The tolerance is a number of seconds and the window is bilateral, so a delivery dated too far
--- ahead is refused exactly like one dated too far behind.
---
--- @param signature string the `x-hook0-signature` header of the delivery
--- @param body string the bytes of the request, before anything parsed them
--- @param headers table the delivered headers, keyed by name or as `{name, value}` pairs
--- @return boolean, string|nil
local function accepted(signature, body, headers)
  -- The secret of the subscription being verified, which the dashboard links to rather than
  -- prints. Asserted, and outside the `pcall` rather than among its arguments. A variable nobody
  -- exported and one exported empty are the same defect: verifying against nothing hashes every
  -- genuine delivery to the wrong code, which this function would then report as a refusal like
  -- any other.
  local secret = os.getenv("HOOK0_SUBSCRIPTION_SECRET")
  assert(secret and #secret > 0, "HOOK0_SUBSCRIPTION_SECRET is not set")

  local ok, refused = pcall(
    Hook0.verify_webhook_signature,
    signature,
    body,
    headers,
    secret,
    300
  )
  if not ok then
    return false, Hook0.message(refused)
  end
  return true
end
-- hook0:snippet:end

-- What makes this file a module rather than a script, and what keeps luacheck from reporting a
-- function nothing reaches. A reader drops the region above into their own handler instead.
return accepted
