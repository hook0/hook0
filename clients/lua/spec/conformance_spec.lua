--- The cases the shared conformance corpus dictates, run against this client.
---
--- The corpus sits at `clients/conformance`, is hand-authored, and is read by the suite of every SDK.
--- Nothing below writes down a verdict, a bound or a signature of its own: they are read out of the
--- committed documents and this client is driven against them over a real socket. A case added to
--- the corpus is therefore exercised here without this file being touched, and a verdict changed
--- there fails here until this client agrees with it again.

local Helper = require("spec.helper")

local Hook0 = Helper.Hook0
local Json = Helper.Json

local RETRY = Helper.contract("retry.json")
local BOUNDS = Helper.contract("bounds.json").bounds
local SIGNATURE = Helper.contract("signature.json")
local REQUEST = Helper.contract("request.json")

local INGESTED_ID = "01961234-5678-7abc-8def-0123456789ac"

--- The budget the delay cases share. A delay the API names above it is expected to be cut down to
--- it, so this also bounds what those cases cost.
local DELAY_BUDGET = 1.1

--- What a wait may overshoot by before it is read as more than what was asked for: a loopback round
--- trip, a timer and a scheduler all sit inside it.
local DELAY_SLACK = 0.6

--- How a refusal the corpus names reads in this client's own words. Every name the corpus declares
--- is looked up here, so one added there stops this suite until it is mapped rather than passing
--- under whatever the client happened to say.
local REFUSALS = {
  code_not_hexadecimal = "not hexadecimal",
  header_not_delivered = "was not delivered",
  code_mismatch = "does not match",
  outside_tolerance = "outside the",
}

--- How this client is made to meet each cause the corpus names, over a real socket.
local PROVOKED = {
  --- A socket that hangs up without answering.
  no_answer = function()
    return { { close = true }, Helper.ingested(INGESTED_ID) }, Helper.options({ max_attempts = 4 })
  end,
  --- An answer larger than what this client agreed to read off the socket.
  answer_above_a_bound = function()
    local oversized = {
      status = 201,
      body = Json.object({ event_id = INGESTED_ID, padding = string.rep("x", 2048) }),
    }
    return { oversized, Helper.ingested(INGESTED_ID) },
      Helper.options({ max_attempts = 4, max_response_bytes = 256 })
  end,
}

--- Two problems answering the same status, one worth repeating and one not.
---
--- That pair is the whole reason the corpus classifies problems rather than statuses, and the
--- retryable one is the answer the API names a delay beside.
local function paced_problem()
  for _, rule in ipairs(RETRY.problems) do
    if rule.retryable then
      for _, other in ipairs(RETRY.problems) do
        if other.status == rule.status and not other.retryable then
          return rule
        end
      end
    end
  end
  error("no status of the corpus carries opposite verdicts")
end

--- What a send says it did, out of the message it failed with.
local function attempts_of(message)
  local named = message:match("gave up after (%d+) attempts")
  return named and math.tointeger(tonumber(named)) or 1
end

--- How many requests a send made when the API answered that way, and whether it ended up ingesting
--- the event.
---
--- A send that reached a server is counted by what that server received. One that never reached
--- anything — an API URL nothing can be sent to is the corpus's own example — is counted by what the
--- client says it did, which is also the message a caller is left holding: a misconfiguration retried
--- four times reads as a network that would not answer.
local function issued_by(responses, build)
  local api = Helper.FakeApi.new(responses)
  local ok, answered = pcall(function()
    return build(api):send_event(Helper.an_event())
  end)
  local received = api:stop()

  if ok then
    return #received, true, answered
  end
  return math.max(#received, attempts_of(Hook0.message(answered))), false, answered
end

describe("the shared conformance corpus", function()
  it("says what every problem does to a send", function()
    -- The status is not what decides: the corpus carries problems answering the same status with
    -- opposite verdicts, and a client reading the status alone fails half of them.
    for _, rule in ipairs(RETRY.problems) do
      local issued, ingested = issued_by(
        { Helper.refusal(rule.status, rule.problem), Helper.ingested(INGESTED_ID) },
        function(api) return Helper.client(api, Helper.options({ max_attempts = 4 })) end
      )
      local expected = rule.retryable and 2 or 1

      assert.are.equal(expected, issued,
        "`" .. rule.problem .. "` under " .. rule.status .. " issued " .. issued ..
        " requests where the corpus expects " .. expected .. ": " .. rule.reason)
      assert.are.equal(rule.retryable, ingested)
    end
  end)

  it("says what every status does to a send", function()
    -- A body naming no problem this client could read is also what an older client meets when the
    -- API names a problem it has never heard of.
    for _, rule in ipairs(RETRY.statuses) do
      local issued = issued_by(
        { Helper.refusal(rule.status, "AProblemThisClientHasNeverHeardOf"), Helper.ingested(INGESTED_ID) },
        function(api) return Helper.client(api, Helper.options({ max_attempts = 4 })) end
      )
      local expected = rule.retryable and 2 or 1

      assert.are.equal(expected, issued,
        "a status of " .. rule.status .. " issued " .. issued .. " requests where the corpus expects " ..
        expected .. ": " .. rule.reason)
    end
  end)

  it("says what a request the API never answered does", function()
    -- Every cause the corpus names is provoked for real rather than reported: a socket that hangs
    -- up, an answer above a ceiling this client set for itself, and a URL nothing can be sent to.
    for _, rule in ipairs(RETRY.transport.causes) do
      local expected = rule.retryable and 2 or 1
      local issued, survived

      if rule.cause == "unusable_api_url" then
        -- Nothing is ever sent, so nothing reaches an API: what this counts is what the client says
        -- it did, which is the message its caller is left holding.
        local api = Helper.FakeApi.new({ Helper.ingested(INGESTED_ID) })
        local built = Hook0.Client.new("gopher://nowhere.invalid", "app-123", "token-xyz",
          Helper.options({ max_attempts = 4 }))
        local ok, raised = pcall(function() return built:send_event(Helper.an_event()) end)
        local received = api:stop()

        issued = math.max(#received, attempts_of(Hook0.message(raised)))
        survived = ok
      else
        local provoke = PROVOKED[rule.cause]
        assert.is_truthy(provoke,
          "the corpus names a cause `" .. rule.cause .. "` this suite does not know how to provoke")
        local responses, options = provoke()
        issued, survived = issued_by(responses, function(api) return Helper.client(api, options) end)
      end

      assert.are.equal(expected, issued,
        "`" .. rule.cause .. "` issued " .. issued .. " requests where the corpus expects " ..
        expected .. ": " .. rule.reason)
      assert.are.equal(rule.retryable, survived)
    end
  end)

  it("has the delay the API names honoured and bounded", function()
    -- The header is written by the other end, so honouring it whole would hand a stranger the length
    -- of this client's send. What the corpus asks for is that a delay be waited out when the budget
    -- can afford it and cut down to what is left of the budget when it cannot.
    local paced = paced_problem()
    local socket = require("socket")

    for _, delay in ipairs(RETRY.retry_after.cases) do
      local api = Helper.FakeApi.new({
        Helper.refusal(paced.status, paced.problem, { { RETRY.retry_after.header, delay.header } }),
        Helper.ingested(INGESTED_ID),
      })
      local chosen = Hook0.Options.new({
        retry_policy = Hook0.RetryPolicy.new({
          max_attempts = 4,
          initial_backoff = 0.005,
          max_backoff = 0.005,
          max_total_delay = DELAY_BUDGET,
        }),
        request_timeout = 5.0,
      })

      local started = socket.gettime()
      Helper.client(api, chosen):send_event(Helper.an_event())
      local waited = socket.gettime() - started
      local received = api:stop()

      local expected = delay.honoured and math.min(delay.seconds + 0.0, DELAY_BUDGET) or 0.0
      assert.are.equal(2, #received, "a paced answer was not retried")
      assert.is_true(waited >= expected, string.format(
        "`%s: %s` was retried after %.3fs, sooner than the %.3fs it asked for",
        RETRY.retry_after.header, delay.header, waited, expected))
      assert.is_true(waited <= expected + DELAY_SLACK, string.format(
        "`%s: %s` held the send for %.3fs, above the %.3fs it is bounded to",
        RETRY.retry_after.header, delay.header, waited, expected))
    end
  end)

  it("names the bounds this client applies", function()
    -- This client's defaults, held against the one place the numbers are written down. What is
    -- asserted is read from the corpus rather than listed here, so a bound added there and left
    -- unapplied fails instead of passing unnoticed.
    local built = Hook0.Client.new("http://127.0.0.1:1", "app-123", "token-xyz")
    local policy = built.options.retry_policy
    local applied = {
      max_attempts = policy.max_attempts,
      max_attempts_cap = Hook0.RetryPolicy.MAX_ATTEMPTS_CAP,
      initial_backoff_ms = policy.initial_backoff * 1000,
      max_backoff_ms = policy.max_backoff * 1000,
      max_total_delay_ms = policy.max_total_delay * 1000,
      request_timeout_ms = built.options.request_timeout * 1000,
      max_payload_bytes = built.options.max_payload_bytes,
      max_response_bytes = built.options.max_response_bytes,
      max_response_headers = built.options.max_response_headers,
      max_header_bytes = built.options.max_header_bytes,
      max_head_bytes = built.options.max_head_bytes,
    }

    for name, wanted in pairs(BOUNDS) do
      assert.is_truthy(applied[name], "the corpus names the bound `" .. name .. "`, which this client does not apply")
      assert.is_true(math.abs(applied[name] - wanted) < 0.001,
        "this client applies " .. tostring(applied[name]) .. " where the corpus names " .. tostring(wanted) ..
        " for `" .. name .. "`")
    end
  end)

  it("refuses an answer above every ceiling it sets on what the other end may send", function()
    -- A bound is a safety property, and conformance to it is shown by the refusal above it: what is
    -- exercised here is a head and a body well over each ceiling, never one just under it.
    local over = {
      max_response_headers = function(count)
        local headers = {}
        for index = 1, count + 1 do
          headers[index] = { "x-padding-" .. index, "x" }
        end
        return headers, ""
      end,
      max_header_bytes = function(size)
        return { { "x-padding", string.rep("x", size + 1) } }, ""
      end,
      max_head_bytes = function(size)
        local headers = {}
        local line = math.ceil(size / 16) + 1
        for index = 1, 17 do
          headers[index] = { "x-padding-" .. index, string.rep("x", line) }
        end
        return headers, ""
      end,
      max_response_bytes = function(size)
        return {}, string.rep("x", size + 1)
      end,
    }

    -- Every ceiling on what the other end may send is exercised, and the set is worked out rather
    -- than written down: what the client applies, less what it applies to its own sending.
    local built = Hook0.Client.new("http://127.0.0.1:1", "app-123", "token-xyz")
    for name, build in pairs(over) do
      local ceiling = built.options[name]
      assert.is_truthy(ceiling, "this client applies no `" .. name .. "`")

      -- A ceiling exercised at what the corpus names would answer a megabyte-scale body over a
      -- loopback socket for every case; what is lowered is the client's own bound, and what is
      -- shown is that crossing it is refused at all.
      local lowered = math.min(ceiling, 4096)
      local headers, body = build(lowered)
      local api = Helper.FakeApi.new({ { status = 200, headers = Json.array(headers), body = body } })

      local raised = Helper.refused(function()
        Helper.client(api, Helper.options({ max_attempts = 1, [name] = lowered }))
          :send_event(Helper.an_event())
      end)
      api:stop()

      assert.is_true(Hook0.is(raised, Hook0.ClientError),
        "an answer above `" .. name .. "` was not refused: " .. Hook0.message(raised))
      assert.is_truthy(Hook0.message(raised):find(tostring(lowered), 1, true),
        "the refusal of an answer above `" .. name .. "` does not name the ceiling it crossed: " ..
        Hook0.message(raised))
    end
  end)

  it("carries every header its occasion declares, and only those", function()
    -- Read back off the socket, on both occasions the corpus declares: a send carries a body, and a
    -- read does not. What separates them is the point — a client that sets `Content-Type` on a
    -- request with nothing in it is describing a body that is not there.
    for _, header in ipairs(REQUEST.headers) do
      local declared = false
      for _, occasion in ipairs(REQUEST.occasions) do
        declared = declared or occasion == header.when
      end
      assert.is_true(declared, "the corpus carries `" .. header.name .. "` on `" .. header.when ..
        "`, which is not one of the occasions it declares")
    end

    local api = Helper.FakeApi.new({ Helper.ingested(INGESTED_ID), { status = 200, body = Json.array({}) } })
    local built = Helper.client(api, Helper.options({ max_attempts = 4 }))
    built:send_event(Helper.an_event())
    built.transport:request("GET", "/applications")
    local received = api:stop()

    local carrying = { [1] = { "every request", "a request carrying a body" }, [2] = { "every request" } }
    for index, occasions in pairs(carrying) do
      local request = received[index]
      assert.is_truthy(request, "the API received no request number " .. index)

      for _, header in ipairs(REQUEST.headers) do
        local carried = request.headers[header.name:lower()]
        local declared = false
        for _, occasion in ipairs(occasions) do
          declared = declared or occasion == header.when
        end

        if declared then
          local wanted = header.value:gsub("%${token}", function() return "token-xyz" end)
          assert.are.equal(wanted, carried, "a request carried `" .. header.name .. ": " ..
            tostring(carried) .. "` where the corpus says `" .. wanted .. "`: " .. header.reason)
        else
          assert.is_nil(carried, "a request carried `" .. header.name .. ": " .. tostring(carried) ..
            "`, which the corpus carries only on `" .. header.when .. "`: " .. header.reason)
        end
      end
    end
  end)

  it("refuses every delivery of the corpus for the reason it names", function()
    -- A refused delivery has to be refused for the reason the corpus names: a client that computed a
    -- code over a header that never arrived and reported a mismatch would otherwise look right.
    for _, name in ipairs(SIGNATURE.refusals) do
      assert.is_truthy(REFUSALS[name], "the corpus declares the refusal `" .. name ..
        "`, which this suite maps to nothing this client says")
    end

    for _, vector in ipairs(SIGNATURE.vectors) do
      local verified = function()
        return Hook0.verify_webhook_signature_with_current_time(
          vector.signature, vector.payload, vector.headers, vector.secret,
          vector.tolerance_seconds + 0.0, vector.current_time)
      end

      if vector.verdict == "accepted" then
        local ok, raised = pcall(verified)
        assert.is_true(ok, vector.name .. ": " .. (ok and "" or Hook0.message(raised)) .. " — " .. vector.reason)
      else
        local raised = Helper.refused(verified)
        assert.is_true(Hook0.is(raised, Hook0.ClientError), vector.name .. " was refused by something else")
        assert.is_truthy(Hook0.message(raised):find(REFUSALS[vector.refusal], 1, true),
          "a delivery the corpus refuses as `" .. vector.refusal .. "` was answered `" ..
          Hook0.message(raised) .. "`: " .. vector.reason)
      end
    end
  end)
end)
