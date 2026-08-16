--- Sending events to Hook0, idempotently and under bounds the caller sets.
---
--- Every event is sent under an identifier this client knows: the one set on the event, or a UUIDv7
--- it generates when the event carries none. Passing none does not mean the identifier comes from
--- Hook0 — the value comes from here, travels with the request, and is what `send_event` answers.
---
--- That is what makes retrying safe. Hook0 keys events on that identifier, so a request repeated
--- after a network failure or a server error ingests the event once rather than twice; without a
--- client-chosen identifier, a repeated request would create a second event and deliver it to every
--- subscriber. It also gives the answer to a retry its meaning: `EventAlreadyIngested` in reply to a
--- *repeated* request says an earlier attempt of that same send reached the API, so the send
--- succeeded. The same answer to a *first* attempt is a genuine conflict and is reported as one.
---
--- Only what could end differently is retried: a request that got no answer, a server error, and an
--- instance saying it is being reached faster than it accepts. What the API refuses outright — a
--- quota that is spent, a payload it will not read — is reported as is, since repeating it would only
--- spend the same round trip again. The verdict for every problem the API can report is written down
--- in the conformance corpus committed beside this rock, which the suite here reads.

local Errors = require("hook0.errors")
local Json = require("hook0.json")
local Runtime = require("hook0.runtime")
local Transport = require("hook0.transport")
local socket = require("socket")

local Client = {}
Client.__index = Client

--- How a client spaces out the attempts of a single send.
---
--- The delay before a retry doubles from `initial_backoff` and is capped by `max_backoff`; the delay
--- actually waited is then drawn anywhere between zero and that ceiling, so that emitters which
--- failed at the same moment do not come back at the same moment. Retrying stops as soon as the
--- delays of the send would add up to more than `max_total_delay`.
---
--- The defaults are four attempts spread over at most five seconds: three retries absorb the blips a
--- webhook emitter meets in production — a connection reset, a rolling deployment answering 503 —
--- without holding the caller for long, and the five-second budget bounds what the worst send costs
--- whatever the individual delays turn out to be.
local RetryPolicy = {}
RetryPolicy.__index = RetryPolicy

--- Most attempts a policy can ever make, whatever `max_attempts` says.
---
--- A policy is configuration, and configuration can be wrong; this cap keeps a mistyped
--- `max_attempts` from turning one send into an unbounded series of requests.
RetryPolicy.MAX_ATTEMPTS_CAP = 16

--- Beyond this many doublings any backoff has long since reached its ceiling.
RetryPolicy.MAX_BACKOFF_DOUBLINGS = 30

--- @param chosen table|nil `max_attempts`, `initial_backoff`, `max_backoff`, `max_total_delay`
--- @return table
function RetryPolicy.new(chosen)
  local asked = chosen or {}
  return setmetatable({
    max_attempts = asked.max_attempts or 4,
    initial_backoff = asked.initial_backoff or 0.1,
    max_backoff = asked.max_backoff or 2.0,
    max_total_delay = asked.max_total_delay or 5.0,
  }, RetryPolicy)
end

--- A policy that never retries: one attempt, and the caller hears what it answered.
--- @return table
function RetryPolicy.disabled()
  return RetryPolicy.new({ max_attempts = 1, initial_backoff = 0.0, max_backoff = 0.0, max_total_delay = 0.0 })
end

--- A number, brought inside a range whatever it was.
local function clamped(value, lowest, highest)
  if type(value) ~= "number" or value ~= value then
    return lowest
  end
  return math.max(lowest, math.min(highest, value))
end

--- Attempts this policy actually makes: `max_attempts`, brought inside `1..MAX_ATTEMPTS_CAP`.
--- @return integer
function RetryPolicy:attempts()
  local cap = RetryPolicy.MAX_ATTEMPTS_CAP
  local asked = math.tointeger(self.max_attempts) or math.floor(clamped(self.max_attempts, 1, cap))
  return math.tointeger(clamped(asked, 1, cap)) or 1
end

--- Ceiling of the delay before retry number `retry_number`, where `1` is the first retry.
---
--- It doubles from `initial_backoff` and never exceeds `max_backoff`, so the ceilings of successive
--- retries never decrease.
---
--- @param retry_number integer
--- @return number
function RetryPolicy:backoff_ceiling(retry_number)
  local doublings = clamped(retry_number - 1, 0, RetryPolicy.MAX_BACKOFF_DOUBLINGS)
  local ceiling = clamped(self.max_backoff, 0.0, math.huge)
  return clamped(clamped(self.initial_backoff, 0.0, math.huge) * (2 ^ doublings), 0.0, ceiling)
end

--- The draw for one retry, brought back inside `[0, 1]` whatever the randomness gave.
---
--- A draw that is missing or is not a finite number is read as `1`, which asks for the whole ceiling:
--- an unusable source of randomness makes the client wait longer, never less.
---
--- @param draws table
--- @param index integer
--- @return number
function RetryPolicy.draw(draws, index)
  local drawn = draws[index]
  if type(drawn) ~= "number" or drawn ~= drawn or drawn == math.huge or drawn == -math.huge then
    return 1.0
  end
  return clamped(drawn, 0.0, 1.0)
end

--- The delays this policy waits between the attempts of one send, one per retry.
---
--- Each delay lands between zero and the ceiling of its retry, and the schedule is cut short as soon
--- as the next delay would spend more than `max_total_delay`. There are therefore at most
--- `attempts - 1` delays, and they add up to at most `max_total_delay`.
---
--- @param draws table one draw in `[0, 1)` per retry
--- @return table
function RetryPolicy:delays(draws)
  local budget = clamped(self.max_total_delay, 0.0, math.huge)
  local waits = {}
  local spent = 0.0

  for retry_number = 1, self:attempts() - 1 do
    local delay = self:backoff_ceiling(retry_number) * RetryPolicy.draw(draws, retry_number)
    if spent + delay > budget then
      break
    end
    spent = spent + delay
    waits[#waits + 1] = delay
  end

  return waits
end

--- Every bound a client applies to one send.
local Options = {}
Options.__index = Options

--- Largest event payload the client agrees to send, in bytes.
---
--- Hook0's API refuses request bodies above 2 MiB, so a payload above 1 MiB is already at risk of
--- being refused once the JSON envelope around it — metadata, labels, identifiers — is counted. The
--- client rules such an event out rather than spending a round trip, and every retry after it, on a
--- request that cannot be accepted.
Options.DEFAULT_MAX_PAYLOAD_BYTES = 1024 * 1024

--- @param chosen table|nil
--- @return table
function Options.new(chosen)
  local asked = chosen or {}
  return setmetatable({
    retry_policy = asked.retry_policy or RetryPolicy.new(),
    request_timeout = asked.request_timeout or Transport.DEFAULT_REQUEST_TIMEOUT,
    max_payload_bytes = asked.max_payload_bytes or Options.DEFAULT_MAX_PAYLOAD_BYTES,
    max_response_bytes = asked.max_response_bytes or Transport.DEFAULT_MAX_RESPONSE_BYTES,
    max_response_headers = asked.max_response_headers or Transport.DEFAULT_MAX_RESPONSE_HEADERS,
    max_header_bytes = asked.max_header_bytes or Transport.DEFAULT_MAX_HEADER_BYTES,
    max_head_bytes = asked.max_head_bytes or Transport.DEFAULT_MAX_HEAD_BYTES,
  }, Options)
end

--- An event type, read out of the `service.resource_type.verb` it is written as.
local EventType = {}
EventType.__index = EventType

--- What an event type reads as.
EventType.PATTERN = "^([A-Za-z0-9_]+)%.([A-Za-z0-9_]+)%.([A-Za-z0-9_]+)$"

--- Reads an event type, refusing one that does not name all three of its parts.
--- @param written string
--- @return table
function EventType.parse(written)
  local service, resource_type, verb = tostring(written or ""):match(EventType.PATTERN)
  if service == nil then
    Errors.throw(Errors.ClientError,
      "`" .. Runtime.preview(written) .. "` is not an event type: it is written service.resource_type.verb")
  end
  return setmetatable({ service = service, resource_type = resource_type, verb = verb }, EventType)
end

--- @return string the event type as the API reads one
function EventType:written()
  return self.service .. "." .. self.resource_type .. "." .. self.verb
end

--- A UUIDv7, the shape of identifier Hook0 mints when it is the one choosing.
---
--- Its leading 48 bits are the current time in milliseconds, so identifiers generated in sequence are
--- ordered, which is what keeps the index they end up in from being written all over.
---
--- @return string
local function generate_event_id()
  local milliseconds = math.floor(socket.gettime() * 1000)

  local bytes = {}
  for index = 1, 6 do
    bytes[index] = (milliseconds >> (8 * (6 - index))) & 0xFF
  end
  for index = 7, 16 do
    bytes[index] = math.random(0, 255)
  end
  bytes[7] = (bytes[7] & 0x0F) | 0x70
  bytes[9] = (bytes[9] & 0x3F) | 0x80

  local written = {}
  for index = 1, 16 do
    written[index] = string.format("%02x", bytes[index])
  end
  local text = table.concat(written)
  return table.concat({
    text:sub(1, 8), text:sub(9, 12), text:sub(13, 16), text:sub(17, 20), text:sub(21, 32),
  }, "-")
end

--- The identifier Hook0 gives the problem it answers when an event identifier is already taken.
Client.ALREADY_INGESTED = "EventAlreadyIngested"

--- The identifier Hook0 gives the problem it answers when requests are reaching the instance faster
--- than it accepts them.
---
--- It shares its status with the quota problems, and is the only one of them worth repeating: a quota
--- clears when a plan changes or a day turns, neither of which happens inside the seconds a send is
--- given, while pacing clears on its own and the answer says when.
Client.RATE_LIMITED = "RateLimited"

--- What Hook0 answers when the event identifier a request carries is already taken.
Client.CONFLICT = 409

--- What Hook0 answers both when a quota is spent and when requests are coming in faster than the
--- instance accepts them. Which of the two it is only the problem the body names can say, which is
--- why this status alone decides nothing.
Client.PACED = 429

--- First status saying the failure is on Hook0's side, and so could clear on its own.
Client.LOWEST_SERVER_ERROR = 500

--- What the API names the delay before the request becomes servable in, in whole seconds.
Client.DELAY_HEADER = "retry-after"

--- Longest value of that header read, and the largest delay it may name. A header written by the
--- other end is bounded before it is turned into a number, and a delay above this is one nobody
--- meant.
Client.MAX_DELAY_HEADER_BYTES = 32
Client.MAX_NAMED_DELAY_SECONDS = (2 ^ 31) - 1

--- Where an event is ingested, under the API URL.
Client.EVENT_PATH = "event"

--- Where event types are read and created, under the API URL.
Client.EVENT_TYPES_PATH = "event_types"

--- Builds a client, once, to be shared wherever an application sends events.
---
--- @param api_url string base API URL of a Hook0 instance, such as https://app.hook0.com/api/v1
--- @param application_id string identifier of the Hook0 application events are sent to
--- @param token string an authentication token valid for that application
--- @param options table|nil the bounds one send is held to
--- @return table
function Client.new(api_url, application_id, token, options)
  local chosen = options or Options.new()
  return setmetatable({
    api_url = api_url,
    application_id = application_id,
    options = chosen,
    transport = Transport.new(api_url, token, {
      request_timeout = chosen.request_timeout,
      max_response_bytes = chosen.max_response_bytes,
      max_response_headers = chosen.max_response_headers,
      max_header_bytes = chosen.max_header_bytes,
      max_head_bytes = chosen.max_head_bytes,
      retry_policy = chosen.retry_policy,
    }),
  }, Client)
end

--- The value that member of a decoded document carries, when it carries a string.
local function text_member(payload, name)
  local ok, read = Json.try_decode(payload)
  if not ok or type(read) ~= "table" or type(read[name]) ~= "string" then
    return nil
  end
  return read[name]
end

--- The delay the API named before the request becomes servable, in seconds.
---
--- Only a whole number of seconds is read. The header may also carry a date, which is a clock this
--- client would be comparing against its own, and anything else is a header nobody meant: both leave
--- the client's own schedule in place rather than being guessed at.
local function named_delay(headers)
  local written = (headers[Client.DELAY_HEADER] or ""):gsub("^%s+", ""):gsub("%s+$", "")
  if written == "" or #written > Client.MAX_DELAY_HEADER_BYTES or not written:match("^%d+$") then
    return nil
  end

  local seconds = tonumber(written)
  if seconds == nil or seconds > Client.MAX_NAMED_DELAY_SECONDS then
    return nil
  end
  return seconds + 0.0
end

--- Whether repeating a request the API answered that way could end differently.
---
--- The status decides on its own everywhere but under the one it answers both a spent quota and a
--- paced instance with: a quota clears when a plan changes or a day turns, and neither is something a
--- send spending seconds can wait for. Only the problem the body names tells the two apart, and a
--- body naming a problem this client has never heard of falls back to what the status says.
local function retryable(status, problem)
  if status == Client.PACED then
    return problem == Client.RATE_LIMITED
  end
  return status >= Client.LOWEST_SERVER_ERROR
end

--- What the API answered one attempt, and whether repeating it could end differently.
local function read_attempt(status, headers, payload)
  if status >= 200 and status < 300 then
    local ingested = text_member(payload, "event_id")
    if ingested == nil then
      -- The API accepted the event but answered something this client cannot read; repeating the
      -- request would meet the same answer.
      return { detail = "Hook0 answered " .. status .. " without an event id" }
    end
    return { ingested = ingested }
  end

  local problem = text_member(payload, "id")
  if status == Client.CONFLICT and problem == Client.ALREADY_INGESTED then
    return { already_ingested = true, detail = payload }
  end

  return {
    detail = payload,
    retryable = retryable(status, problem),
    retry_after = named_delay(headers),
  }
end

--- One attempt at sending an already-bounded event.
local function attempt(client, body)
  local ok, answered = pcall(function()
    return { client.transport:deliver("POST", Client.EVENT_PATH, nil, body) }
  end)
  if ok then
    return read_attempt(answered[1], answered[2], answered[3])
  end
  if Errors.is(answered, Transport.TransportError) then
    return { detail = answered.message, retryable = answered.retryable }
  end
  error(answered, 0)
end

--- An event as the API reads one.
local function full_event(client, event, event_id)
  local body = Runtime.document({
    event_id = event_id,
    application_id = client.application_id,
    event_type = event.event_type,
    payload = event.payload,
    payload_content_type = event.payload_content_type,
    occurred_at = event.occurred_at or os.date("!%Y-%m-%dT%H:%M:%SZ"),
    labels = Runtime.document(event.labels or {}),
  })
  if event.metadata ~= nil then
    body.metadata = Runtime.document(event.metadata)
  end
  return body
end

--- Sends an event, and answers the identifier it was sent under.
---
--- @param event table `event_type`, `payload`, `payload_content_type`, and optionally `labels`,
---   `metadata`, `occurred_at` and `event_id`
--- @return string
function Client:send_event(event)
  local event_id = event.event_id
  if type(event_id) ~= "string" or event_id == "" then
    event_id = generate_event_id()
  end

  local size = #tostring(event.payload or "")
  if size > self.options.max_payload_bytes then
    Errors.throw(Errors.ClientError, "the payload of event " .. event_id .. " is " .. size ..
      " bytes, above the " .. self.options.max_payload_bytes .. " this client sends")
  end

  local body = full_event(self, event, event_id)
  local policy = self.options.retry_policy
  local draws = {}
  for index = 1, policy:attempts() - 1 do
    draws[index] = math.random()
  end
  local delays = policy:delays(draws)

  local issued = 0
  local waited = 0.0
  while true do
    issued = issued + 1
    local outcome = attempt(self, body)

    if outcome.ingested ~= nil then
      return outcome.ingested
    end
    if outcome.already_ingested then
      if issued > 1 then
        return event_id
      end
      Errors.throw(Errors.ClientError,
        "sending event " .. event_id .. " failed: " .. Runtime.preview(outcome.detail))
    end

    local scheduled = outcome.retryable and delays[issued] or nil
    if scheduled == nil then
      if issued <= 1 then
        Errors.throw(Errors.ClientError,
          "sending event " .. event_id .. " failed: " .. Runtime.preview(outcome.detail))
      end
      Errors.throw(Errors.ClientError, string.format(
        "sending event %s gave up after %d attempts over %.3f seconds: %s",
        event_id, issued, waited, Runtime.preview(outcome.detail)))
    end

    -- What the API asked for when it asked for anything, and this client's own schedule otherwise.
    -- Either way it is cut down to what is left of the budget every delay of one send shares, so a
    -- delay written by the other end cannot stretch a send past what the caller allowed for it.
    local wanted = outcome.retry_after or scheduled
    local waiting = clamped(wanted, 0.0, math.max(policy.max_total_delay - waited, 0.0))
    socket.sleep(waiting)
    waited = waited + waiting
  end
end

--- The event types an application already declares, out of what the API answered.
local function declared_event_types(client)
  local ok, answered = pcall(function()
    return { client.transport:request("GET", Client.EVENT_TYPES_PATH,
      { { "application_id", client.application_id } }) }
  end)
  if not ok then
    Errors.throw(Errors.ClientError, "reading the available event types failed: " .. Errors.message(answered))
  end

  local status, payload = answered[1], answered[2]
  if status < 200 or status >= 300 then
    Errors.throw(Errors.ClientError, "reading the available event types failed: " .. Runtime.preview(payload))
  end

  local read_ok, read = Json.try_decode(payload)
  if not read_ok or type(read) ~= "table" or Json.is_object(read) then
    Errors.throw(Errors.ClientError, "reading the available event types failed: the API did not answer a list")
  end

  local declared = {}
  for index = 1, #read do
    local entry = read[index]
    if type(entry) == "table" and type(entry.event_type_name) == "string" then
      declared[entry.event_type_name] = true
    end
  end
  return declared
end

--- Declares one event type on the application.
local function create_event_type(client, event_type)
  local ok, answered = pcall(function()
    return { client.transport:request("POST", Client.EVENT_TYPES_PATH, nil, Runtime.document({
      application_id = client.application_id,
      service = event_type.service,
      resource_type = event_type.resource_type,
      verb = event_type.verb,
    })) }
  end)
  if not ok then
    Errors.throw(Errors.ClientError,
      "creating event type " .. event_type:written() .. " failed: " .. Errors.message(answered))
  end

  local status, payload = answered[1], answered[2]
  if status < 200 or status >= 300 then
    Errors.throw(Errors.ClientError,
      "creating event type " .. event_type:written() .. " failed: " .. Runtime.preview(payload))
  end
end

--- Creates the event types the application does not declare yet, and answers those.
---
--- @param event_types table
--- @return table
function Client:upsert_event_types(event_types)
  local wanted = {}
  for index = 1, #event_types do
    wanted[index] = EventType.parse(event_types[index])
  end
  if #wanted == 0 then
    return {}
  end

  local declared = declared_event_types(self)
  local created = {}
  for index = 1, #wanted do
    local written = wanted[index]:written()
    if not declared[written] then
      create_event_type(self, wanted[index])
      created[#created + 1] = written
    end
  end
  return created
end

Client.RetryPolicy = RetryPolicy
Client.Options = Options
Client.EventType = EventType
Client.generate_event_id = generate_event_id

return Client
