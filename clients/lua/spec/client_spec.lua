--- What a send does, driven against a real socket.
---
--- Everything the shared corpus dictates is exercised in `conformance_spec.lua`; what is here is what
--- this client decides for itself: the identifier it mints, the bounds it applies to what it sends,
--- and what a caller is left holding when a send does not land.

local Helper = require("spec.helper")

local Hook0 = Helper.Hook0
local Json = Helper.Json

local INGESTED_ID = "01961234-5678-7abc-8def-0123456789ac"

--- The shape a UUID has, whichever version it carries.
local UUID_PATTERN = "^%x%x%x%x%x%x%x%x%-%x%x%x%x%-%x%x%x%x%-%x%x%x%x%-%x%x%x%x%x%x%x%x%x%x%x%x$"

--- The event identifier one request carried, as the API read it.
local function event_id_of(request)
  return Json.decode(request.body).event_id
end

describe("a send", function()
  it("answers the identifier it minted, and mints one shaped like a UUIDv7", function()
    local api = Helper.FakeApi.new({ Helper.ingested(INGESTED_ID) })
    local answered = Helper.client(api):send_event(Helper.an_event())
    local received = api:stop()

    assert.are.equal(INGESTED_ID, answered, "a send answers the identifier the API said it ingested")
    assert.are.equal(1, #received)

    local minted = event_id_of(received[1])
    assert.is_truthy(minted:match(UUID_PATTERN), "`" .. minted .. "` is not shaped like a UUID")
    assert.are.equal("7", minted:sub(15, 15), "the identifier this client mints is not a UUIDv7")
  end)

  it("sends the identifier the caller set, when the caller set one", function()
    local api = Helper.FakeApi.new({ Helper.ingested(INGESTED_ID) })
    Helper.client(api):send_event(Helper.an_event({ event_id = INGESTED_ID }))
    local received = api:stop()

    assert.are.equal(INGESTED_ID, event_id_of(received[1]))
  end)

  it("repeats one identifier across every attempt it makes", function()
    -- The whole reason the identifier is minted here rather than by the API: a request repeated
    -- after a failure has to be the same request, or Hook0 ingests the event twice and delivers it
    -- to every subscriber twice.
    local api = Helper.FakeApi.new({
      { status = 500, body = Json.object({ id = "InternalServerError", status = 500 }) },
      { status = 500, body = Json.object({ id = "InternalServerError", status = 500 }) },
      Helper.ingested(INGESTED_ID),
    })
    Helper.client(api, Helper.options({ max_attempts = 4 })):send_event(Helper.an_event())
    local received = api:stop()

    assert.are.equal(3, #received, "a server error was not retried")

    -- That every attempt carried the *same* identifier says nothing on its own: a client that sent
    -- none at all would satisfy it too, and that is the client whose retries ingest the event twice.
    local first = event_id_of(received[1])
    assert.is_truthy(first, "the first attempt carried no event identifier at all")
    assert.is_truthy(first:match(UUID_PATTERN), "`" .. tostring(first) .. "` is not an identifier this client minted")

    for index = 2, #received do
      assert.are.equal(first, event_id_of(received[index]),
        "attempt " .. index .. " carried a different identifier from the first, so a retry could ingest twice")
    end
  end)

  it("reads a conflict on a repeated attempt as its own earlier attempt having landed", function()
    local api = Helper.FakeApi.new({
      { status = 500, body = Json.object({ id = "InternalServerError", status = 500 }) },
      {
        status = 409,
        body = Json.object({ id = "EventAlreadyIngested", status = 409, title = "Event already Ingested" }),
      },
    })
    local answered = Helper.client(api, Helper.options({ max_attempts = 4 })):send_event(Helper.an_event())
    local received = api:stop()

    assert.are.equal(2, #received)
    assert.are.equal(event_id_of(received[1]), answered,
      "a conflict answering a repeated attempt is that attempt having landed, and the send succeeded")
  end)

  it("reads a conflict on a first attempt as the conflict it is", function()
    local api = Helper.FakeApi.new({
      {
        status = 409,
        body = Json.object({ id = "EventAlreadyIngested", status = 409, title = "Event already Ingested" }),
      },
    })
    local raised = Helper.refused(function()
      return Helper.client(api):send_event(Helper.an_event({ event_id = INGESTED_ID }))
    end)
    local received = api:stop()

    assert.are.equal(1, #received, "a conflict on a first attempt was retried")
    assert.is_true(Hook0.is(raised, Hook0.ClientError))
    assert.is_truthy(Hook0.message(raised):find(INGESTED_ID, 1, true),
      "the failure does not name the event it was sending: " .. Hook0.message(raised))
  end)

  it("gives up on an attempt that runs out of the time it was given", function()
    -- One attempt, so what is measured is the timeout and nothing about how attempts are spaced.
    local api = Helper.FakeApi.new({ { status = 201, held_for = 2.0, body = Json.object({}) } })
    local socket = require("socket")

    local started = socket.gettime()
    local raised = Helper.refused(function()
      return Helper.client(api, Helper.options({ max_attempts = 1, request_timeout = 0.2 }))
        :send_event(Helper.an_event())
    end)
    local spent = socket.gettime() - started
    api:stop()

    assert.is_true(Hook0.is(raised, Hook0.ClientError))
    assert.is_true(spent < 1.5, string.format("a send given 0.2s to answer held its caller for %.3fs", spent))
  end)

  it("refuses a payload above the bound it sends, before a socket is opened", function()
    local api = Helper.FakeApi.new({ Helper.ingested(INGESTED_ID) })
    local raised = Helper.refused(function()
      return Helper.client(api, Helper.options({ max_payload_bytes = 16 }))
        :send_event(Helper.an_event({ payload = string.rep("x", 17) }))
    end)
    local received = api:stop()

    assert.are.equal(0, #received, "an oversized payload cost a round trip")
    assert.is_truthy(Hook0.message(raised):find("16", 1, true),
      "the refusal does not name the bound it crossed: " .. Hook0.message(raised))
  end)

  it("sends the whole event the API reads", function()
    local api = Helper.FakeApi.new({ Helper.ingested(INGESTED_ID) })
    Helper.client(api):send_event(Helper.an_event({ metadata = { region = "eu-west-1" } }))
    local received = api:stop()

    local sent = Json.decode(received[1].body)
    assert.are.equal("app-123", sent.application_id)
    assert.are.equal("auth.user.create", sent.event_type)
    assert.are.equal("application/json", sent.payload_content_type)
    assert.are.equal("production", sent.labels.environment)
    assert.are.equal("eu-west-1", sent.metadata.region)
    assert.is_truthy(sent.occurred_at:match("^%d%d%d%d%-%d%d%-%d%dT%d%d:%d%d:%d%dZ$"),
      "`" .. tostring(sent.occurred_at) .. "` is not a moment the API reads")
    assert.are.equal("POST", received[1].verb)
  end)
end)

describe("a schedule", function()
  it("never spends more than the budget the policy gave it", function()
    local policy = Hook0.RetryPolicy.new({
      max_attempts = 8,
      initial_backoff = 1.0,
      max_backoff = 4.0,
      max_total_delay = 2.5,
    })
    local delays = policy:delays({ 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0 })

    local spent = 0.0
    for index = 1, #delays do
      spent = spent + delays[index]
    end
    assert.is_true(spent <= 2.5 + 1e-9, "a schedule spent " .. spent .. " of a 2.5 second budget")
    assert.is_true(#delays <= policy:attempts() - 1)
  end)

  it("makes at most the attempts nothing may cross, whatever it was configured with", function()
    assert.are.equal(Hook0.RetryPolicy.MAX_ATTEMPTS_CAP,
      Hook0.RetryPolicy.new({ max_attempts = 10000 }):attempts())
    assert.are.equal(1, Hook0.RetryPolicy.new({ max_attempts = -4 }):attempts())
    assert.are.equal(1, Hook0.RetryPolicy.disabled():attempts())
  end)

  it("is in force with the default for a duration no schedule could be built on", function()
    -- A value that is not a finite number names no duration, and a policy holding one is in force
    -- with the default of that field. Both halves are held to it: what the request states and what
    -- the send would actually wait, because a header that named a policy the client does not run
    -- would be worse than no header at all.
    local default = Hook0.RetryPolicy.new()
    local draws = { 0.5, 0.5, 0.5 }

    for _, unusable in ipairs({ { "+INF", math.huge }, { "-INF", -math.huge }, { "NaN", 0 / 0 } }) do
      for _, held in ipairs({ "initial_backoff", "max_backoff", "max_total_delay" }) do
        local api = Helper.FakeApi.new({ { status = 200, body = Json.array({}) } })
        local policy = Hook0.RetryPolicy.new({ [held] = unusable[2] })
        Hook0.Transport.new(api:base_url(), "token-xyz", { retry_policy = policy })
          :request("GET", "/applications")
        local carried = api:stop()[1].headers["hook0-client-options"]
        local because = "a policy holding `" .. held .. ": " .. unusable[1] .. "`"

        assert.are.equal("attempts=4,backoff=100,ceiling=2000,budget=5000", carried, because)
        assert.are.same(default:delays(draws), policy:delays(draws), because)
      end
    end
  end)
end)

describe("upserting event types", function()
  it("creates only the ones the application does not declare yet", function()
    local api = Helper.FakeApi.new({
      { status = 200, body = Json.array({ Json.object({ event_type_name = "auth.user.create" }) }) },
      { status = 201, body = Json.object({ event_type_name = "billing.invoice.paid" }) },
    })
    local created = Helper.client(api):upsert_event_types({ "auth.user.create", "billing.invoice.paid" })
    local received = api:stop()

    assert.are.same({ "billing.invoice.paid" }, created)
    assert.are.equal(2, #received)
    assert.are.equal("GET", received[1].verb)
    assert.are.equal("POST", received[2].verb)
    assert.are.equal("billing", Json.decode(received[2].body).service)
  end)

  it("refuses an event type that does not name all three of its parts", function()
    local raised = Helper.refused(function()
      return Hook0.EventType.parse("auth.user")
    end)
    assert.is_true(Hook0.is(raised, Hook0.ClientError))
  end)
end)
