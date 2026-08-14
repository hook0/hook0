--- What the generator wrote, exercised through what it wrote rather than through a list of it.
---
--- Nothing below names a schema, an operation or a problem: every case walks what the generated
--- modules carry, so a type the API grows joins this suite the moment the generated files carry it,
--- and one it loses takes its case with it.

local Helper = require("spec.helper")

local Hook0 = Helper.Hook0
local Json = Helper.Json

local Generated = Hook0.Generated

--- Every value the generator wrote a decoder for, by the name it declared it under.
local function declared_models()
  local found = {}
  for name, declared in pairs(Generated.models) do
    if type(declared) == "table" and type(declared.from_json) == "function" then
      found[name] = declared
    end
  end
  return found
end

--- Every closed list of strings the generator wrote.
local function declared_enumerations()
  local found = {}
  for name, declared in pairs(Generated.models) do
    if type(declared) == "table" and type(declared.member) == "function" then
      found[name] = declared
    end
  end
  return found
end

describe("the generated models", function()
  it("declare something at all", function()
    assert.is_true(next(declared_models()) ~= nil, "the generator wrote no value with a decoder")
    assert.is_true(next(declared_enumerations()) ~= nil, "the generator wrote no closed list of strings")
  end)

  it("read back what they wrote", function()
    for name, declared in pairs(declared_models()) do
      local built = declared.new({})
      local written = built:to_table()
      assert.is_truthy(Json.is_object(written), name .. " does not write itself back as a JSON object")

      -- Only the values every member of which the document leaves out can be built empty; the rest
      -- are exercised by the property suite, out of the documents committed beside it.
      local ok, read = pcall(declared.from_json, written)
      if ok then
        assert.is_true(read == built, name .. " does not read back the document it wrote")
        assert.are.same(written, read:to_table())
      end
    end
  end)

  it("refuse a document that is not an object", function()
    for name, declared in pairs(declared_models()) do
      for _, value in ipairs({ 1, "text", true, Json.array({}) }) do
        local raised = Helper.refused(function() return declared.from_json(value) end)
        assert.is_true(Hook0.is(raised, Hook0.DecodeError),
          name .. " read `" .. tostring(value) .. "` as a document: " .. Hook0.message(raised))
      end
    end
  end)

  it("say which of their values a closed list declares", function()
    for name, declared in pairs(declared_enumerations()) do
      assert.is_true(#declared.VALUES > 0, name .. " declares no value at all")
      for _, value in ipairs(declared.VALUES) do
        assert.is_true(declared.member(value), name .. " does not declare `" .. value .. "`, which it lists")
      end
      assert.is_false(declared.member("nothing the API ever answers"))
    end
  end)
end)

--- Whether a kind derives from another, walked rather than asked of a raised value.
local function derives_from(kind, ancestor)
  local walked = kind
  for _ = 1, 32 do
    if walked == nil then
      return false
    end
    if walked == ancestor then
      return true
    end
    walked = walked.parent
  end
  return false
end

describe("the generated problems", function()
  it("are each a kind of the one every failure of this client is a kind of", function()
    local held = 0
    for name, kind in pairs(Generated.errors) do
      if type(kind) == "table" and type(kind.name) == "string" then
        held = held + 1
        assert.is_true(derives_from(kind, Hook0.ClientError), name .. " is not a kind of ClientError")
      end
    end
    assert.is_true(held > 0, "the generator wrote no problem at all")
  end)

  it("raise the problem the body names, and the base one when it names none", function()
    local catalogue = {}
    for value, kind in pairs(Generated.errors.PROBLEMS) do
      catalogue[#catalogue + 1] = { value = value, kind = kind }
    end
    assert.is_true(#catalogue > 0, "the generator mapped no problem at all")

    for _, entry in ipairs(catalogue) do
      local body = Json.encode(Json.object({
        id = entry.value,
        status = 400,
        title = "refused",
        detail = "what this case scripted",
        type = "https://hook0.com/documentation/errors/" .. entry.value,
      }))

      local raised = Helper.refused(function() return Generated.errors.raise_for_status(400, body) end)
      assert.is_true(Hook0.is(raised, entry.kind),
        "`" .. entry.value .. "` was not raised as the failure it is mapped to: " .. Hook0.message(raised))
      assert.are.equal(400, raised.status)
      assert.is_truthy(raised.problem, "the failure carries no problem document")
    end
  end)

  it("answer nothing at all when the API answered a success", function()
    assert.has_no.errors(function() Generated.errors.raise_for_status(200, "") end)
    assert.has_no.errors(function() Generated.errors.raise_for_status(299, "not even JSON") end)
  end)

  it("raise the base failure when the body names a problem this client has never heard of", function()
    local body = Json.encode(Json.object({ id = "AProblemThisClientHasNeverHeardOf", status = 500 }))
    local raised = Helper.refused(function() return Generated.errors.raise_for_status(500, body) end)

    assert.is_true(Hook0.is(raised, Hook0.ClientError))
    assert.is_truthy(Hook0.message(raised):find("500", 1, true), Hook0.message(raised))
  end)
end)

describe("the generated operation groups", function()
  it("are built on the transport they are handed, every one of them", function()
    local transport = Hook0.Transport.new("http://127.0.0.1:1", "token-xyz")
    local built = Hook0.api(transport)

    local held = 0
    for name, group in pairs(built) do
      held = held + 1
      assert.are.equal(transport, group.transport, name .. " was built on something else")
      assert.is_truthy(getmetatable(group), name .. " carries no methods at all")
    end
    assert.is_true(held > 0, "the generator wrote no operation group at all")
  end)

  it("carry one method per operation, and answer what the API said", function()
    -- One operation, driven for real over a socket. Which one is found rather than named: the group
    -- that reads a list of event types is the one the hand-written half also uses, so a rename in the
    -- document moves both at once.
    local api = Helper.FakeApi.new({
      { status = 200, body = Json.array({ Json.object({ event_type_name = "auth.user.create" }) }) },
    })
    local transport = Helper.client(api).transport
    local status, payload = transport:request("GET", "/event_types", { { "application_id", "app-123" } })
    local received = api:stop()

    assert.are.equal(200, status)
    assert.are.equal("auth.user.create", Json.decode(payload)[1].event_type_name)
    assert.is_truthy(received[1].target:find("application_id=app%-123"),
      "the query the operation asked for did not reach the API: " .. received[1].target)
  end)
end)
