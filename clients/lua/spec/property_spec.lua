--- What holds for every input, rather than for the ones a case happened to pick.
---
--- Five things are checked here. A retry schedule never spends more than the policy that produced it
--- allows, whichever way the randomness fell. Reading a signature header answers with the one failure
--- this rock declares, whatever text reached the endpoint, and never with anything else. A document
--- written out reads back as the value that was written. A value read out of a document the API could
--- answer is written back as the value that was read. And identifiers minted in sequence never go
--- back in time.
---
--- There is no `hypothesis`-grade tool for Lua, and this rock installs nothing at runtime beyond a
--- socket, so the search is written here: a fixed seed, a bounded number of draws, and the
--- counter-examples worth keeping committed under `regressions/` so they run as ordinary cases on
--- every pipeline. A failing draw is one somebody can reproduce by running the suite again rather
--- than one that goes away on a retry.

local Helper = require("spec.helper")

local Hook0 = Helper.Hook0
local Json = Helper.Json

--- What the draws are made from. Fixed, so the suite explores the same inputs everywhere it runs.
local SEED = 20260814

--- How many draws each property makes. Bounded, so a pipeline can never be held by one.
local DRAWS = 200

--- How far two sums of the same numbers may sit apart before the difference is a defect rather than
--- the order they were added in.
local ROUNDING = 1e-9

--- The bounds a drawn policy is built inside.
local MAX_DRAWN_ATTEMPTS = 64
local MAX_DRAWN_SECONDS = 10.0
local MAX_DRAWN_BUDGET = 60.0

--- Longest header a draw builds, in pieces.
local MAX_DRAWN_HEADER = 96

--- The pieces a signature is made of, put together every way a sender that is not Hook0 might put
--- them together.
local PIECES = { "t", "v0", "v1", "h", "=", ",", "0", "9", "zz", "abc", "x-event-id", "1800000000",
  "-1", '"', " ", ".", "{", "}" }

--- A draw that is no draw at all, which has to make the client wait longer rather than less.
local function unusable(drawn)
  if drawn == "nan" then
    return 0 / 0
  end
  if drawn == "infinity" then
    return math.huge
  end
  if drawn == "-infinity" then
    return -math.huge
  end
  return drawn
end

--- Every value the generator wrote a decoder for.
local function declared_models()
  local found = {}
  for name, declared in pairs(Hook0.Generated.models) do
    if type(declared) == "table" and type(declared.from_json) == "function" then
      found[name] = declared
    end
  end
  return found
end

describe("a retry schedule", function()
  it("stays within every bound of the policy that produced it", function()
    math.randomseed(SEED)

    local cases = {}
    for _, written in ipairs(Helper.regressions("retry_policies")) do
      cases[#cases + 1] = written
    end
    for _ = 1, DRAWS do
      local draws = {}
      for index = 1, math.random(0, 8) do
        draws[index] = (math.random() * 2) - 0.5
      end
      cases[#cases + 1] = {
        math.random(-4, MAX_DRAWN_ATTEMPTS),
        math.random() * MAX_DRAWN_SECONDS,
        math.random() * MAX_DRAWN_SECONDS,
        math.random() * MAX_DRAWN_BUDGET,
        draws,
      }
    end

    for _, drawn in ipairs(cases) do
      local policy = Hook0.RetryPolicy.new({
        max_attempts = drawn[1],
        initial_backoff = drawn[2],
        max_backoff = drawn[3],
        max_total_delay = drawn[4],
      })
      local draws = {}
      for index = 1, #drawn[5] do
        draws[index] = unusable(drawn[5][index])
      end

      local delays = policy:delays(draws)
      local budget = math.max(policy.max_total_delay, 0.0)
      local spent = 0.0

      assert.is_true(policy:attempts() >= 1)
      assert.is_true(policy:attempts() <= Hook0.RetryPolicy.MAX_ATTEMPTS_CAP)
      assert.is_true(#delays <= policy:attempts() - 1)

      for index = 1, #delays do
        spent = spent + delays[index]
        assert.is_true(delays[index] >= 0.0, "a schedule waits " .. delays[index] .. " seconds")
        assert.is_true(delays[index] <= policy:backoff_ceiling(index) + ROUNDING)
        assert.is_true(delays[index] <= math.max(policy.max_backoff, 0.0) + ROUNDING)
      end
      assert.is_true(spent <= budget + ROUNDING,
        "a schedule spent " .. spent .. " of a " .. budget .. " second budget")

      -- A schedule never hurries up as it goes: the ceiling of a retry never sits below the one
      -- before it.
      for index = 2, policy:attempts() do
        assert.is_true(policy:backoff_ceiling(index) >= policy:backoff_ceiling(index - 1))
      end
    end
  end)
end)

describe("reading a signature", function()
  it("answers with the one failure this rock declares, whatever reached the endpoint", function()
    math.randomseed(SEED)

    local headers = {}
    for _, written in ipairs(Helper.regressions("signatures")) do
      headers[#headers + 1] = written
    end
    for _ = 1, DRAWS do
      local pieces = {}
      for index = 1, math.random(0, MAX_DRAWN_HEADER) do
        pieces[index] = PIECES[math.random(1, #PIECES)]
      end
      headers[#headers + 1] = table.concat(pieces)
    end

    for _, header in ipairs(headers) do
      local ok, raised = pcall(Hook0.Signature.parse, header)
      if not ok then
        assert.is_true(Hook0.is(raised, Hook0.ClientError),
          "`" .. tostring(header) .. "` was refused by something a caller cannot name: " .. Hook0.message(raised))
      else
        -- Parsing answered, so verifying has to answer the same way: a header that reads must not
        -- find a way to fail that a caller cannot name.
        local verified, refusal = pcall(Hook0.verify_webhook_signature_with_current_time,
          header, "", {}, "secret", 300.0, 0)
        assert.is_true(verified or Hook0.is(refusal, Hook0.ClientError),
          "`" .. tostring(header) .. "` verified into something a caller cannot name: " ..
          Hook0.message(refusal))
      end
    end
  end)
end)

describe("a document", function()
  it("reads back as the value that was written", function()
    math.randomseed(SEED)

    for _, document in ipairs(Helper.regressions("documents")) do
      local written = Json.encode(document)
      assert.are.same(document, Json.decode(written), "`" .. written .. "` does not read back")
      assert.are.equal(written, Json.encode(Json.decode(written)),
        "`" .. written .. "` does not write back out the same way")
    end
  end)

  it("is refused by the one failure this rock declares, whatever the text", function()
    math.randomseed(SEED)

    local characters = { "{", "}", "[", "]", '"', ":", ",", "0", "9", "e", "-", "n", "u", "l", "t", "\\", " " }
    for _ = 1, DRAWS do
      local pieces = {}
      for index = 1, math.random(0, 48) do
        pieces[index] = characters[math.random(1, #characters)]
      end

      local ok, read = pcall(Json.decode, table.concat(pieces))
      if not ok then
        assert.is_truthy(Json.reason(read),
          "`" .. table.concat(pieces) .. "` was refused by something this rock does not declare: " ..
          tostring(read))
      else
        assert.has_no.errors(function() Json.encode(read) end)
      end
    end
  end)
end)

describe("a generated type", function()
  it("reads back what it wrote, and refuses what it cannot read", function()
    math.randomseed(SEED)

    local documents = Helper.regressions("documents")
    local drawn = {}
    for _, document in ipairs(documents) do
      drawn[#drawn + 1] = document
      for _ = 1, 4 do
        -- A document with one of its members taken away, replaced by something of another type, or
        -- buried inside something else.
        local keys = {}
        for key in pairs(document) do
          keys[#keys + 1] = key
        end
        if #keys > 0 then
          local mutated = Json.object({})
          local chosen = keys[math.random(1, #keys)]
          for key, value in pairs(document) do
            mutated[key] = value
          end
          local how = math.random(0, 3)
          if how == 0 then
            mutated[chosen] = nil
          elseif how == 1 then
            mutated[chosen] = math.random(0, 1000)
          elseif how == 2 then
            mutated[chosen] = Json.array({ document[chosen] })
          else
            mutated[chosen] = Json.null
          end
          drawn[#drawn + 1] = mutated
        end
      end
    end

    for name, declared in pairs(declared_models()) do
      for _, document in ipairs(drawn) do
        local ok, read = pcall(declared.from_json, document)
        if ok then
          local written = read:to_table()
          assert.is_true(read == declared.from_json(written),
            "a " .. name .. " read out of a document does not read back")
          assert.are.same(written, declared.from_json(written):to_table())
        else
          assert.is_true(Hook0.is(read, Hook0.DecodeError),
            name .. " refused a document with something a caller cannot name: " .. Hook0.message(read))
        end
      end
    end
  end)
end)

describe("a minted identifier", function()
  it("carries a moment that never goes back", function()
    -- Not that two identifiers land in the same millisecond, which nothing guarantees and which
    -- would fail about once a week: what is asserted is that the moment never runs backwards.
    local moments = {}
    for index = 1, DRAWS do
      local minted = Hook0.generate_event_id()
      moments[index] = minted:sub(1, 8) .. minted:sub(10, 13)
    end

    for index = 2, #moments do
      assert.is_true(moments[index] >= moments[index - 1],
        "identifier " .. index .. " carries an earlier moment than the one before it: " ..
        moments[index - 1] .. " then " .. moments[index])
    end
  end)

  it("is shaped like a UUIDv7, every time", function()
    for _ = 1, DRAWS do
      local minted = Hook0.generate_event_id()
      assert.is_truthy(minted:match("^%x%x%x%x%x%x%x%x%-%x%x%x%x%-7%x%x%x%-[89ab]%x%x%x%-%x%x%x%x%x%x%x%x%x%x%x%x$"),
        "`" .. minted .. "` is not a UUIDv7")
    end
  end)
end)
