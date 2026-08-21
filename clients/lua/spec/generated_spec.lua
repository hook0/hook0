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

local Document = require("spec.api_document")
local Surface = require("spec.generated_surface")

--- Every operation the generator wrote, in one order, so that what is scripted lines up with what is
--- driven: the API answers in the order it is asked, and it is asked in the order below.
local function every_operation()
  local found = {}
  for _, group in ipairs(Surface.groups()) do
    for _, operation in ipairs(Surface.operations_of(group.declared)) do
      found[#found + 1] = {
        named = group.name .. ":" .. operation.name,
        group = group.declared,
        operation = operation.declared,
      }
    end
  end
  return found
end

--- A value the API answered, written back into the document it came out of.
---
--- A list of them is a plain sequence rather than a value of its own, so it is walked here: what is
--- compared either way is the document, which is the only thing both sides of the wire agree on.
local function written_answer(value, listed)
  if value == nil then
    return Json.object({})
  end
  if not listed then
    return Surface.written(value)
  end

  local out = {}
  for index = 1, #value do
    out[index] = Surface.written(value[index])
  end
  return Json.array(out)
end

--- What the API answers one operation, and the document that operation is expected to read out of it.
local function answer_for(operation, optionals)
  local kind, listed = Surface.answered(operation)
  if kind == nil then
    return { status = 204, body = Json.object({}) }, Json.object({}), false
  end

  local written = written_answer({ Surface.value_of(kind, optionals) }, true)
  if listed then
    return { status = 200, body = written }, written, true
  end
  return { status = 200, body = written[1] }, written[1], false
end

--- Every operation the document declares issues the request it is declared as, and reads it back.
---
--- Asked twice: once giving every argument an operation may be asked with, once giving only the ones
--- it requires, which is what says an argument left out leaves the query it would have filled empty.
--- The closing assertion is the one that grows — an operation the API adds fails it until something
--- drives it.
local function reaches_every_operation(optionals)
  local driven = every_operation()
  local scripted = {}
  local expected = {}
  local listed = {}
  for index, entry in ipairs(driven) do
    scripted[index], expected[index], listed[index] = answer_for(entry.operation, optionals)
  end

  local api = Helper.FakeApi.new(scripted)
  local transport = Helper.client(api).transport
  local read = {}
  local ok, raised = pcall(function()
    for index, entry in ipairs(driven) do
      local built = entry.group.new(transport)
      local given = Surface.arguments(entry.operation, optionals)
      read[index] = entry.operation(built, table.unpack(given, 1, given.n))
    end
  end)
  local received = api:stop()
  assert.is_true(ok, ok and "" or Hook0.message(raised))

  assert.are.equal(#driven, #received, "the API was not asked once per operation")

  local reached = {}
  for index, entry in ipairs(driven) do
    local request = received[index]
    local declared = Document.reached(request.verb, request.target)

    assert.are.equal(Json.encode(expected[index]), Json.encode(written_answer(read[index], listed[index])),
      entry.named .. " did not read back what the API answered")
    assert.are.equal("Bearer token-xyz", request.headers.authorization, entry.named)
    assert.are.equal("application/json", request.headers.accept, entry.named)

    -- The value lands in the path escaped, so that nothing in it can name a segment the operation
    -- never had.
    local sent = Document.Declared.segments_of(request.target)
    local wanted = Document.Declared.segments_of(declared.template)
    for at = 1, #wanted do
      if wanted[at]:match("^{.*}$") then
        assert.are.equal("a%20value%2Fwith%20a%20space", sent[at],
          entry.named .. " left " .. wanted[at] .. " unescaped")
      end
    end

    local carried = Document.query_of(request.target)
    local names = {}
    for name in pairs(carried) do
      names[#names + 1] = name
      assert.are.equal(Surface.ARGUMENT_TEXT, carried[name], entry.named .. " carried `" .. name .. "` altered")
    end
    table.sort(names)
    assert.are.same(declared:query_names(optionals), names,
      entry.named .. " assembled a query the document does not declare")

    reached[declared:named()] = true
  end

  local named = {}
  for one in pairs(reached) do
    named[#named + 1] = one
  end
  table.sort(named)

  local declares = {}
  for _, one in ipairs(Document.operations()) do
    declares[#declares + 1] = one:named()
  end
  table.sort(declares)

  assert.are.same(declares, named, "the generated groups reach other operations than the document declares")
end

describe("every operation the API document declares", function()
  it("is reached with everything it may carry", function()
    reaches_every_operation(true)
  end)

  it("is reached with only what it requires", function()
    reaches_every_operation(false)
  end)
end)

--- Every member of a value, by the name it travels under.
local function wire_names_of(model)
  local names = {}
  for _, declared in ipairs(Surface.params_of(model.new)) do
    names[declared.name] = declared.rest:match("^carries%s+`([^`]+)`")
  end
  return names
end

--- The members of a value the document describes nothing about, by the name they travel under.
---
--- Whatever the API answers under one of these is kept as it arrived, so nothing is ever refused
--- there and the cases below have nothing to hold them to.
local function opaque_names(model)
  local found = {}
  for _, declared in ipairs(Surface.params_of(model.new)) do
    if declared.type:gsub("|nil$", "") == "any" then
      found[declared.rest:match("^carries%s+`([^`]+)`")] = true
    end
  end
  return found
end

--- How many members of a value the document describes nothing about.
local function opaque_count(model)
  local held = 0
  for _ in pairs(opaque_names(model)) do
    held = held + 1
  end
  return held
end

describe("every value the API document declares", function()
  -- Run once with every member the schema may leave out set and once with none of them, which is
  -- what tells a member that was read apart from one that was defaulted to the same thing.
  for _, optionals in ipairs({ true, false }) do
    local how = optionals and "with everything it may carry" or "with only what it requires"

    it("reads back what it wrote " .. how, function()
      for _, entry in ipairs(Surface.models()) do
        local held = Surface.built(entry.declared, optionals)
        local written = held:to_table()
        assert.is_truthy(Json.is_object(written), entry.name .. " does not write itself back as a JSON object")

        local read = entry.declared.from_json(written)
        assert.is_true(read == held, entry.name .. " does not read back what it wrote")
        assert.are.equal(Json.encode(written), Json.encode(read:to_table()),
          entry.name .. " does not write back what it read")

        for name, wire in pairs(wire_names_of(entry.declared)) do
          if held[name] == nil then
            assert.is_nil(written[wire], entry.name .. " wrote `" .. wire .. "` out although it holds nothing")
          end
        end
      end
    end)
  end

  -- Everything a schema describes is refused when it arrives as something else, and says which
  -- member it was. What it leaves undescribed is kept as it arrived and so is refused by nothing,
  -- and there are exactly as many members that accept anything as it leaves undescribed.
  it("refuses a member the document does not declare it as", function()
    for _, entry in ipairs(Surface.models()) do
      local written = Surface.built(entry.declared, true):to_table()
      local accepted = {}

      for name in pairs(written) do
        local wrong = {}
        for key, value in pairs(written) do
          wrong[key] = value
        end
        -- Neither an object nor a scalar any of the readers accept, whichever the member is.
        wrong[name] = Json.array({ Json.object({ neither = "a scalar" }) })

        local ok, raised = pcall(entry.declared.from_json, Json.object(wrong))
        if ok then
          accepted[#accepted + 1] = name
        else
          assert.is_truthy(Hook0.message(raised):find(name, 1, true),
            entry.name .. " did not say which member it could not read: " .. Hook0.message(raised))
        end
      end

      assert.are.equal(opaque_count(entry.declared), #accepted,
        entry.name .. " read " .. table.concat(accepted, ", ") .. " although it describes what they hold")
    end
  end)

  -- Which members carry a list is read off what each value wrote rather than named here, so a schema
  -- that grows one is held to this the moment it does.
  it("refuses a document that is not a list where it declares one", function()
    local walked = 0
    for _, entry in ipairs(Surface.models()) do
      local written = Surface.built(entry.declared, true):to_table()
      local opaque = opaque_names(entry.declared)
      for name, value in pairs(written) do
        if Json.is_array(value) and not opaque[name] then
          local wrong = {}
          for key, carried in pairs(written) do
            wrong[key] = carried
          end
          wrong[name] = "not a list at all"

          local raised = Helper.refused(function() return entry.declared.from_json(Json.object(wrong)) end)
          assert.is_truthy(Hook0.message(raised):find("expected an array", 1, true), Hook0.message(raised))
          assert.is_truthy(Hook0.message(raised):find(name, 1, true), Hook0.message(raised))
          walked = walked + 1
        end
      end
    end

    assert.is_true(walked > 0, "no value the API declares carries a list")
  end)
end)

describe("a failure too long to report whole", function()
  it("is cut rather than carried into whatever the caller logs", function()
    local written = string.rep("e", Hook0.Runtime.MAX_PREVIEW_BYTES + 64) .. ", and not a problem document"
    local api = Helper.FakeApi.new({ { status = 500, body = written } })
    local transport = Helper.client(api).transport
    local group = Surface.groups()[1].declared.new(transport)
    local operation = Surface.operations_of(Surface.groups()[1].declared)[1]

    local raised = Helper.refused(function()
      local given = Surface.arguments(operation.declared, false)
      return operation.declared(group, table.unpack(given, 1, given.n))
    end)
    api:stop()

    local message = Hook0.message(raised)
    assert.are.equal(500, raised.status)
    assert.is_truthy(message:find("…", 1, true), "the report was not cut: " .. message)
    assert.is_falsy(message:find("not a problem document", 1, true), message)
  end)
end)
