--- The Lua client against a Hook0 that is really running.
---
--- Two things happen here, and the second is the reason the first is worth having.
---
--- The control: whether an application secret the API minted is accepted, whether a second send
--- under an identifier already ingested is reported as the conflict it is, and whether a signature
--- the output worker computed verifies. Those are the three questions no loopback suite can ask
--- itself, because a suite that signs and verifies with the same sources only proves the sources
--- agree with themselves.
---
--- The surface: every operation the API document declares, driven through the generated layer
--- against the same instance, and every model type it decodes out of a real answer.
--- `clients/lua/spec` already drives all of them — against an API the suite itself writes, out of
--- the same document the client was generated from. That proves the client matches the document. It
--- cannot prove the document matches Hook0, and a field the API really answers under another name
--- passes there and fails on a consumer's first call.

--- Where this smoke sits, so it runs the same wherever it is started from.
---
--- The directory is whatever the interpreter was handed, which is nothing at all when the file was
--- named without one — as it is when the harness starts it with this directory as the working one.
local source = debug.getinfo(1, "S").source:sub(2)
local here = source:match("^(.*)[/\\][^/\\]*$") or "."
local client_root = here .. "/../../../clients/lua"

--- Loads the rock the way installing it would.
---
--- A searcher rather than a `package.path`, because the rock's module names and its source layout
--- do not follow one another closely enough for a path to express: `hook0` is `src/hook0.lua`, and
--- everything under it drops the prefix. That correspondence is not assumed — `spec/rockspec_spec`
--- walks `src` and fails when the rockspec and the tree disagree — so applying it here loads what
--- installing the rock would put in place. Ahead of the filesystem searchers, so a rock installed
--- on the machine is never the one exercised.
table.insert(package.searchers, 2, function(name)
  local under = name == "hook0" and "hook0" or name:match("^hook0%.(.+)$")
  if under == nil then
    return "\n\t`" .. name .. "` is not a module of this client"
  end
  local path = client_root .. "/src/" .. under:gsub("%.", "/") .. ".lua"
  local chunk, refused = loadfile(path)
  if chunk == nil then
    return "\n\t" .. tostring(refused)
  end
  return chunk, path
end)

local socket = require("socket")
local Hook0 = require("hook0")

local Errors = Hook0.Generated.errors
local Json = Hook0.Json
local Models = Hook0.Generated.models
local Api = Hook0.Generated.api

--- The conflict the API answers a duplicated ingestion with.
local ALREADY_INGESTED = "EventAlreadyIngested"

--- What this smoke labels everything it creates with, so that the subscription it makes and the
--- event it sends find each other.
local LANGUAGE = "lua"

--- Where the subscription this smoke creates points. Nothing listens there, deliberately: what a
--- delivery proves is proved once, by the webhook the harness catches and every language verifies.
local NOWHERE = "http://127.0.0.1:1/"

--- Reads a whole file, refusing one larger than this smoke reads back.
local MAX_PART_BYTES = 1024 * 1024

--- What a paced instance answers.
local TOO_MANY_REQUESTS = 429

--- The most times one request is sent again after that answer.
local PACED_AGAIN = 8

--- The shortest this waits between two tries, and the longest whatever the answer asked for.
local SHORTEST_PAUSE = 0.2
local LONGEST_PAUSE = 10.0

--- What every generated method is issued through, waiting out a paced instance.
---
--- Hook0 paces callers per credential, and a flow driving three dozen operations one after another
--- is exactly what that is for. The answer says the request was not processed and is safe to send
--- again after the delay it names, so this waits and sends it again rather than handing the caller a
--- problem that says nothing about the operation it was asking about.
---
--- It wraps the transport the rock ships rather than replacing it: `deliver` is what that transport
--- offers a caller who needs what the answer carried beside its body, which is precisely the delay.
local Paced = {}
Paced.__index = Paced

--- @param inner table the transport this one issues through
--- @return table
function Paced.new(inner)
  return setmetatable({ inner = inner }, Paced)
end

--- How long the answer says to wait, held between a floor and a ceiling of this smoke's own.
---
--- The floor is there because the header counts in whole seconds and the delay being waited out is
--- a fraction of one, so a truthful `Retry-After: 0` would otherwise mean sending the same request
--- again immediately, forever. The ceiling is there because a header is written by a server this
--- smoke does not control.
local function pause(headers)
  local asked = tonumber(tostring(headers["retry-after"] or ""):match("^%s*(%d+)%s*$")) or SHORTEST_PAUSE
  return math.max(SHORTEST_PAUSE, math.min(asked, LONGEST_PAUSE))
end

--- The shape the generated half reads, which is the status and the bytes.
--- @return integer, string
function Paced:request(method, path, query, body)
  for sent = 1, PACED_AGAIN + 1 do
    local status, headers, payload = self.inner:deliver(method, path, query, body)
    if status ~= TOO_MANY_REQUESTS or sent > PACED_AGAIN then
      return status, payload
    end
    socket.sleep(pause(headers))
  end
end

--- A setting the harness passes, or a refusal naming it: a smoke that ran without one would report
--- a failure of the client for something the harness never handed it.
local function setting(name)
  local value = os.getenv(name)
  if value == nil or value == "" then
    error(name .. " is not set", 0)
  end
  return value
end

--- One part of the delivery, as the harness wrote it down.
local function read_part(delivery, part)
  local handle = assert(io.open(delivery .. "/" .. part, "rb"), part .. " is not readable")
  local what = handle:read(MAX_PART_BYTES)
  handle:close()
  return what or ""
end

--- The event both sends carry, under the identifier the caller names.
local function event(event_type, event_id)
  return {
    event_id = event_id,
    event_type = event_type,
    payload = '{"from":"the lua smoke"}',
    payload_content_type = "application/json",
    labels = { language = LANGUAGE },
  }
end

--- The same event, twice, under the identifier the API minted for the first of them.
local function send_twice()
  local client = Hook0.Client.new(
    setting("HOOK0_API_URL"),
    setting("HOOK0_APPLICATION_ID"),
    setting("HOOK0_TOKEN")
  )
  local event_type = setting("HOOK0_EVENT_TYPE")

  local sent = client:send_event(event(event_type, nil))
  print("ingested " .. sent)

  local accepted, raised = pcall(function()
    return client:send_event(event(event_type, sent))
  end)
  if accepted then
    error("sending the same event twice was accepted twice", 0)
  end
  local said = Hook0.message(raised)
  if not said:find(ALREADY_INGESTED, 1, true) then
    error("the second send failed without naming " .. ALREADY_INGESTED .. ": " .. said, 0)
  end
  print("the second send reported " .. ALREADY_INGESTED)
end

--- Reports one operation the flow goes on to use the answer of, which has to be a success.
local function read(operation, asking)
  local ok, answered = pcall(asking)
  if not ok then
    error(
      operation .. ": the flow needs what it answers, and it answered " .. Hook0.message(answered),
      0
    )
  end
  print("exercised " .. operation .. " accepted")
  return answered
end

--- Reports one operation driven for its own sake, whichever way the instance answered it.
---
--- A success and a problem are both complete round trips through the generated layer: the request
--- was composed, the instance answered, and this client read the answer. What is neither — the API
--- not reached, a body this client cannot read, a problem it does not know — stops the smoke,
--- because none of those say the client and the instance agree on anything.
local function exercised(operation, asking)
  local ok, raised = pcall(asking)
  if ok then
    print("exercised " .. operation .. " accepted")
    return
  end

  if not Hook0.is(raised, Errors.ProblemError) or raised.problem == nil then
    error(operation .. ": " .. Hook0.message(raised), 0)
  end
  print("exercised " .. operation .. " refused:" .. raised.problem.id)
end

--- Reports one generated model type as decoded out of a real answer.
---
--- The value is taken rather than only named, so the line cannot outlive what it is about. Taking it
--- is not enough on its own here: reading a member this client no longer carries answers `nil` in
--- Lua rather than raising, so the line would go on being printed about a type nothing decoded.
--- Every value below is a member the document marks as always there, so `nil` is the one thing it
--- cannot be — which is what makes this refuse rather than decorate.
local function decoded(model, value)
  if value == nil then
    error(model .. " was not decoded out of what the API answered", 0)
  end
  print("decoded " .. model)
end

--- The instance without the path the hand-written half is built with.
---
--- The generated half composes paths that already carry `/api/v1`, since the API document's own
--- server URL is the bare origin. Handing this transport the whole of `HOOK0_API_URL` happens to
--- reach the same request, because a path of its own replaces the base's — but that is how one
--- language joins two URLs rather than a contract, and the TypeScript client was posting to
--- `/api/event` until the first live run found it. So this points at the origin, which is what the
--- contract says.
local function origin_of(api_url)
  local origin = api_url:match("^(%a[%w+.%-]*://[^/]+)")
  if origin == nil then
    error("`" .. api_url .. "` is not somewhere a request can be sent", 0)
  end
  return origin
end

--- Every operation the API document declares, driven against the instance in the order a consumer
--- would: what it needs is created, read and listed, updated, and destroyed last.
---
--- Two credentials, because the API takes two and one of them cannot do everything. An application
--- secret is scoped to the application it belongs to; what belongs to the organization — listing its
--- applications, everything about service tokens, its per-day counts — needs the organization-scoped
--- token beside it.
local function surface()
  local origin = origin_of(setting("HOOK0_API_URL"))
  local application = setting("HOOK0_APPLICATION_ID")
  local organization = setting("HOOK0_ORGANIZATION_ID")
  local seeded = setting("HOOK0_SEEDED_APPLICATION_ID")
  local labels = { language = LANGUAGE }

  local held = Paced.new(Hook0.Transport.new(origin, setting("HOOK0_TOKEN")))
  local organization_wide = Paced.new(Hook0.Transport.new(origin, setting("HOOK0_SERVICE_TOKEN")))

  local applications = Api.ApplicationsApi.new(held)
  local secrets = Api.ApplicationSecretsApi.new(held)
  local event_types = Api.EventTypesApi.new(held)
  local subscriptions = Api.SubscriptionsApi.new(held)
  local events = Api.EventsApi.new(held)
  local events_per_day = Api.EventsPerDayApi.new(held)
  local instance = Api.InstanceApi.new(held)
  local quotas = Api.QuotasApi.new(held)
  local payload_content_types = Api.PayloadContentTypesApi.new(held)
  local error_catalogue = Api.ErrorsApi.new(held)

  local organization_applications = Api.ApplicationsApi.new(organization_wide)
  local organization_events_per_day = Api.EventsPerDayApi.new(organization_wide)
  local request_attempts = Api.RequestAttemptsApi.new(organization_wide)
  local responses = Api.ResponseApi.new(organization_wide)
  local service_tokens = Api.ServiceTokenApi.new(organization_wide)

  -- What the instance says about itself, which is what an application asks before it has anything
  -- of its own: how it is configured, what it will let this account do, what a payload may be, and
  -- every problem it can report.
  decoded("InstanceConfig", read("instance.get", function() return instance:get() end))

  local allowed = read("quotas.get", function() return quotas:get() end)
  decoded("QuotasResponseLimits", allowed.limits)
  decoded("QuotasResponse", allowed)

  exercised("payload_content_types.list", function() return payload_content_types:list() end)

  local catalogue = read("errors.list", function() return error_catalogue:list() end)
  if #catalogue == 0 then
    error("the instance published an empty catalogue of the problems it can report", 0)
  end
  decoded("ProblemId", catalogue[1].id)
  decoded("Problem", catalogue[1])

  -- The application this smoke owns. One per language, so that the three deletions at the end of
  -- this flow are real deletions rather than something eleven other smokes have to live with.
  local info = read("applications.get", function() return applications:get(application) end)
  decoded("ApplicationInfoConsumption", info.consumption)
  decoded("ApplicationInfoQuotas", info.quotas)
  decoded("ApplicationInfoOnboardingStepsEvent", info.onboarding_steps.event)
  decoded("ApplicationInfoOnboardingStepsEventType", info.onboarding_steps.event_type)
  decoded("ApplicationInfoOnboardingStepsSubscription", info.onboarding_steps.subscription)
  decoded("ApplicationInfoOnboardingSteps", info.onboarding_steps)
  decoded("ApplicationInfo", info)

  local renamed = read("applications.update", function()
    return applications:update(application, Models.ApplicationPost.new({
      name = "the application the lua smoke drives",
      organization_id = organization,
    }))
  end)
  decoded("Application", renamed)

  -- The organization's, so the organization credential. Listing what an account has is the first
  -- thing a console does.
  exercised("applications.list", function()
    return organization_applications:list(organization)
  end)

  -- This one is driven with the *application* secret on purpose, and it is the flow's one refusal.
  -- Creating an application is the organization's business and an application secret is not the
  -- organization's, so the instance answers a problem document and this client reads it — which is
  -- the half of the client that nothing else here would exercise.
  exercised("applications.create", function()
    return applications:create(Models.ApplicationPost.new({
      name = "an application the lua smoke's application secret may not create",
      organization_id = organization,
    }))
  end)

  -- A second secret, so that the one this smoke is authenticating with is never the one it revokes.
  -- Deleting that one succeeds and then locks the flow out of everything below.
  local minted = read("applicationSecrets.create", function()
    return secrets:create(Models.ApplicationSecretPost.new({
      application_id = application,
      name = "a secret the lua smoke minted",
    }))
  end)
  decoded("ApplicationSecret", minted)

  exercised("applicationSecrets.list", function() return secrets:list(application) end)
  exercised("applicationSecrets.update", function()
    return secrets:update(minted.token, Models.ApplicationSecretPost.new({
      application_id = application,
      name = "a secret the lua smoke renamed",
    }))
  end)
  exercised("applicationSecrets.delete", function()
    return secrets:delete(minted.token, application)
  end)

  -- An event type of this smoke's own, rather than the one the harness declared: what is created
  -- here is what is subscribed to, sent, replayed and deleted below.
  local declared = read("eventTypes.create", function()
    return event_types:create(Models.EventTypePost.new({
      application_id = application,
      resource_type = "smoke",
      service = LANGUAGE,
      verb = "ran",
    }))
  end)
  decoded("EventType", declared)

  exercised("eventTypes.get", function()
    return event_types:get(declared.event_type_name, application)
  end)
  exercised("eventTypes.list", function() return event_types:list(application) end)

  -- Marked as an object rather than left as a bare table: an empty Lua table says nothing about
  -- whether it is a list or a map, and the API reads the headers of a target as a map.
  local target = Models.SubscriptionPostTarget.new({
    headers = Json.object({}),
    method = "POST",
    type = "http",
    url = NOWHERE,
  })
  local subscription = read("subscriptions.create", function()
    return subscriptions:create(Models.SubscriptionPost.new({
      application_id = application,
      event_types = { declared.event_type_name },
      is_enabled = true,
      target = target,
      description = "what the lua smoke subscribes to its own events with",
      labels = labels,
    }))
  end)
  decoded("SubscriptionTarget", subscription.target)
  decoded("Subscription", subscription)

  exercised("subscriptions.get", function()
    return subscriptions:get(subscription.subscription_id)
  end)
  exercised("subscriptions.list", function() return subscriptions:list(application) end)
  exercised("subscriptions.update", function()
    return subscriptions:update(subscription.subscription_id, Models.SubscriptionPost.new({
      application_id = application,
      event_types = { declared.event_type_name },
      is_enabled = true,
      target = target,
      description = "what the lua smoke renamed it to",
      labels = labels,
    }))
  end)

  -- The event the subscription above selects, sent through the generated layer rather than through
  -- send_event: the hand-written half has its own three questions above, and this is the operation
  -- the document declares.
  local ingested = read("events.ingest", function()
    return events:ingest(Models.EventPost.new({
      application_id = application,
      event_type = declared.event_type_name,
      labels = labels,
      occurred_at = os.date("!%Y-%m-%dT%H:%M:%SZ"),
      payload = '{"from":"the lua smoke"}',
      payload_content_type = "application/json",
      event_id = Hook0.generate_event_id(),
    }))
  end)
  decoded("IngestedEvent", ingested)

  decoded("EventWithPayload", read("events.get", function()
    return events:get(ingested.event_id, application)
  end))

  local listed = read("events.list", function() return events:list(application) end)
  if #listed == 0 then
    error("the instance ingested an event and then listed none", 0)
  end
  decoded("Event", listed[1])

  exercised("events.replay", function()
    return events:replay(ingested.event_id, Models.ReplayEvent.new({ application_id = application }))
  end)

  -- This application was created a moment ago and the counts come out of a view the instance
  -- refreshes on a cycle of its own, so this answers a list with nothing in it — which is an answer,
  -- and one a client has to be able to read.
  exercised("events_per_day.list_for_application", function()
    return events_per_day:list_for_application(application)
  end)

  -- The organization's counts do have something in them: the harness waited for the instance to
  -- refresh them before running any of this, precisely so that the type they are answered with is
  -- one a client decodes rather than one nothing ever produces.
  local per_day = read("events_per_day.list_for_organization", function()
    return organization_events_per_day:list_for_organization(organization)
  end)
  if #per_day == 0 then
    error("the organization has ingested events and its per-day counts are empty", 0)
  end
  decoded("EventsPerDayEntry", per_day[1])

  -- An attempt and a response exist only once the output worker has finished a delivery. The
  -- harness waited for one, in the application it caught the shared delivery from, and handed the
  -- ids on — so this reads them back with the organization credential rather than waiting again.
  exercised("requestAttempts.list", function() return request_attempts:list(seeded) end)

  local attempted = read("requestAttempts.get", function()
    return request_attempts:get(setting("HOOK0_REQUEST_ATTEMPT_ID"), seeded)
  end)
  decoded("RequestAttemptEvent", attempted.event)
  decoded("RequestAttemptSubscription", attempted.subscription)
  decoded("RequestAttemptStatusType", attempted.status.type)
  decoded("RequestAttemptStatus", attempted.status)
  decoded("RequestAttempt", attempted)

  decoded("Response", read("response.get", function()
    return responses:get(setting("HOOK0_RESPONSE_ID"), seeded)
  end))

  -- Service tokens belong to the organization, so they are minted, read and revoked with the
  -- organization credential. The one revoked below is the one minted here — never the one this half
  -- of the flow is authenticating with.
  local issued = read("serviceToken.create", function()
    return service_tokens:create(Models.ServiceTokenPost.new({
      name = "a token the lua smoke minted",
      organization_id = organization,
    }))
  end)
  decoded("ServiceToken", issued)

  exercised("serviceToken.list", function() return service_tokens:list(organization) end)
  exercised("serviceToken.get", function()
    return service_tokens:get(issued.token_id, organization)
  end)
  exercised("serviceToken.update", function()
    return service_tokens:update(issued.token_id, Models.ServiceTokenPost.new({
      name = "a token the lua smoke renamed",
      organization_id = organization,
    }))
  end)
  exercised("serviceToken.delete", function()
    return service_tokens:delete(issued.token_id, organization)
  end)

  -- Destroyed in the order the instance can accept: the subscription that references the event
  -- type, then the event type, then the application — which is last because the secret this whole
  -- flow authenticates with stops authenticating the moment its application is gone.
  exercised("subscriptions.delete", function()
    return subscriptions:delete(subscription.subscription_id, application)
  end)
  exercised("eventTypes.delete", function()
    return event_types:delete(declared.event_type_name, application)
  end)
  exercised("applications.delete", function() return applications:delete(application) end)
end

--- Verifies what the output worker really delivered, with this client's own verification.
local function verify(delivery)
  local headers = {}
  for line in (read_part(delivery, "headers") .. "\n"):gmatch("([^\n]*)\n") do
    local name, value = line:match("^([^:]+): (.*)$")
    if name ~= nil then
      headers[name] = value
    end
  end

  Hook0.verify_webhook_signature(
    (read_part(delivery, "signature"):gsub("%s+$", "")),
    read_part(delivery, "body"),
    headers,
    (read_part(delivery, "secret"):gsub("%s+$", "")),
    tonumber((read_part(delivery, "tolerance"):gsub("%s+$", "")))
  )
end

local ok, refused = pcall(function()
  send_twice()
  surface()

  -- Last, and on purpose: it needs no instance at all, so it still answers after the flow above has
  -- deleted the application it was run against.
  verify(setting("HOOK0_DELIVERY"))
  print("the signature the instance produced verifies")
end)

if not ok then
  io.stderr:write(Hook0.message(refused) .. "\n")
  os.exit(1)
end
