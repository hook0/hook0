--- Everything the generator wrote, found by looking at what it wrote, and one value of each.
---
--- Lua spells a UUID, a moment and a name all as `string`, and holds no types at all at run time, so
--- what tells them apart is what the generator wrote beside them: the docstring of a constructor
--- names the type of every member, and the docstring of an operation names both the type of every
--- argument, in the order it takes them, and the wire name it travels under. Nothing here lists a
--- schema, a member or an operation — a value of anything the generator writes is built by reading
--- what it wrote about it.

local Helper = require("spec.helper")

local Hook0 = Helper.Hook0
local Json = Helper.Json

local Surface = {}

--- Where the API document the generator was run against sits, from where this file sits.
Surface.DOCUMENT = Helper.CLIENT_ROOT .. "/../../api/openapi.snapshot.json"

--- Largest document read back, far above what the API's snapshot ever is.
Surface.MAX_DOCUMENT_BYTES = 8 * 1024 * 1024

--- No file the generator writes is longer than this, which bounds what is read back to find a
--- docstring in it.
Surface.MAX_SOURCE_LINES = 50000

--- What every string-shaped member of a value is given.
Surface.MODEL_TEXT = "3f2504e0-4f89-41d3-9a0c-0305e82c3301"

--- What every string-shaped argument of an operation is given. It carries the two characters a path
--- segment may not leave as they are, so a value reaching a path proves it was escaped.
Surface.ARGUMENT_TEXT = "a value/with a space"

--- What a member the document describes nothing about carries, kept as it arrived.
Surface.AN_OPAQUE_VALUE = { "describes", "none", "of", "this" }

--- No schema the API declares nests anywhere near this deep.
Surface.MAX_DEPTH = 8

local sources = {}
local docstrings = {}

--- Every line of a file the generator wrote, read back once.
--- @param path string
--- @return table
local function lines_of(path)
  if sources[path] == nil then
    local read = {}
    for line in Helper.read_file(path, Surface.MAX_DOCUMENT_BYTES):gmatch("([^\n]*)\n?") do
      read[#read + 1] = line
      assert(#read <= Surface.MAX_SOURCE_LINES, path .. " is longer than the lines read back")
    end
    sources[path] = read
  end
  return sources[path]
end

--- What the generator wrote above a function, one tag per entry with its continuations folded in.
--- @param declared function
--- @return table
local function docstring_of(declared)
  local where = debug.getinfo(declared, "S")
  local path = where.source:match("^@(.*)")
  assert(path ~= nil, "a function the generator wrote was not written down anywhere")

  local key = path .. ":" .. where.linedefined
  if docstrings[key] == nil then
    local lines = lines_of(path)
    local tags = {}
    local index = where.linedefined - 1
    local block = {}
    while index >= 1 and lines[index]:match("^%s*%-%-%-") do
      table.insert(block, 1, (lines[index]:gsub("^%s*%-%-%-%s?", "")))
      index = index - 1
    end
    for _, line in ipairs(block) do
      if line:match("^@") or #tags == 0 then
        tags[#tags + 1] = line
      else
        tags[#tags] = tags[#tags] .. " " .. line:gsub("^%s+", "")
      end
    end
    docstrings[key] = tags
  end
  return docstrings[key]
end

--- What the generator wrote about every argument or member, in the order it wrote them.
---
--- A constructor takes one table and names its members `fields.<name>`; an operation takes them one
--- after another and names them plainly. Both read the same way here, and the order is the order.
--- @param declared function
--- @return table
function Surface.params_of(declared)
  local found = {}
  for _, tag in ipairs(docstring_of(declared)) do
    local name, written = tag:match("^@param%s+([%w_.]+)%s+(.*)$")
    if name ~= nil then
      -- A map is the one type the generator writes a space inside, so it is read off first and
      -- everything else is the one word up to what was said about it.
      local kind, rest = written:match("^(table<[^>]*>|nil)%s*(.*)$")
      if kind == nil then
        kind, rest = written:match("^(table<[^>]*>)%s*(.*)$")
      end
      if kind == nil then
        kind, rest = written:match("^(%S+)%s*(.*)$")
      end
      found[#found + 1] = { name = (name:gsub("^fields%.", "")), type = kind, rest = rest }
    end
  end
  return found
end

--- What the generator said a function answers, as it wrote it.
--- @param declared function
--- @return string
function Surface.returned_of(declared)
  for _, tag in ipairs(docstring_of(declared)) do
    local written = tag:match("^@return%s+(%S+)")
    if written ~= nil then
      return written
    end
  end
  return "nil"
end

--- Everything the generator declared under one of its modules, sorted so a run repeats.
--- @param module table
--- @param kept function
--- @return table
local function declared_in(module, kept)
  local found = {}
  for name, value in pairs(module) do
    if kept(value) then
      found[#found + 1] = { name = name, declared = value }
    end
  end
  table.sort(found, function(one, other)
    return one.name < other.name
  end)
  return found
end

--- Every value the generator wrote a decoder for, by the name it declared it under.
--- @return table
function Surface.models()
  local found = declared_in(Hook0.Generated.models, function(value)
    return type(value) == "table" and type(value.from_json) == "function"
  end)
  assert(#found > 0, "the generator wrote no value with a decoder")
  return found
end

--- Every group of operations the generator wrote, by the name it declared it under.
--- @return table
function Surface.groups()
  local found = declared_in(Hook0.Generated.api, function(value)
    return type(value) == "table" and type(value.new) == "function"
  end)
  assert(#found > 0, "the generator wrote no group of operations at all")
  return found
end

--- Every operation one group carries, under the name it is called by.
--- @param group table
--- @return table
function Surface.operations_of(group)
  local found = declared_in(group, function(value)
    return type(value) == "function"
  end)
  local kept = {}
  for _, entry in ipairs(found) do
    if entry.name ~= "new" then
      kept[#kept + 1] = entry
    end
  end
  return kept
end

local function value_for(declared, optionals, text, depth)
  assert(depth <= Surface.MAX_DEPTH, "`" .. declared.type .. "` nests more than " .. Surface.MAX_DEPTH .. " deep")

  local carried = declared.type:gsub("|nil$", "")
  local listed = carried:match("^(.+)%[%]$")
  if listed ~= nil then
    return Json.array({ value_for({ type = listed, rest = declared.rest }, optionals, text, depth + 1) })
  end
  local keyed = carried:match("^table<[^,]+,%s*(.-)>$")
  if keyed ~= nil then
    return Json.object({ ["a key"] = value_for({ type = keyed, rest = declared.rest }, optionals, text, depth + 1) })
  end

  if carried == "string" then
    -- Which value of a closed list it is does not matter; that it is one of them does.
    local named = declared.rest:match("one of%s+`Models%.([%w_]+)%.VALUES`")
    if named ~= nil then
      return Hook0.Generated.models[named].VALUES[1]
    end
    return text
  end
  if carried == "integer" then
    return 12
  end
  if carried == "number" then
    return 1.5
  end
  if carried == "boolean" then
    return true
  end
  if carried == "any" or carried == "table" then
    return Json.array(Surface.AN_OPAQUE_VALUE)
  end

  local model = Hook0.Generated.models[carried]
  assert(model ~= nil, "the generator wrote a `" .. carried .. "` nothing here can build")
  return Surface.built(model, optionals, depth + 1)
end

--- One value of a schema the API declares, with every member it may leave out set or not.
--- @param model table
--- @param optionals boolean
--- @param depth integer|nil
--- @return table
function Surface.built(model, optionals, depth)
  local held = {}
  for _, declared in ipairs(Surface.params_of(model.new)) do
    if optionals or not declared.type:match("|nil$") then
      held[declared.name] = value_for(declared, optionals, Surface.MODEL_TEXT, depth or 0)
    end
  end
  return model.new(held)
end

--- One value of the type the generator wrote, as a member of a value carries it.
--- @param kind string
--- @param optionals boolean
--- @return any
function Surface.value_of(kind, optionals)
  return value_for({ type = kind, rest = "" }, optionals, Surface.MODEL_TEXT, 0)
end

--- What one operation is asked with, in the order it takes them: everything it requires, and what it
--- does not as asked for.
--- @param operation function
--- @param optionals boolean
--- @return table
function Surface.arguments(operation, optionals)
  local declared_by_the_generator = Surface.params_of(operation)
  -- An argument left out is a hole rather than a shorter call, so how many there are travels
  -- beside them: `table.unpack` cannot find the end of a table whose last entry is nothing.
  local given = { n = #declared_by_the_generator }
  for index, declared in ipairs(declared_by_the_generator) do
    if optionals or not declared.type:match("|nil$") then
      given[index] = value_for(declared, optionals, Surface.ARGUMENT_TEXT, 0)
    end
  end
  return given
end

--- The wire name every argument travels under, by the name the generator calls it.
--- @param operation function
--- @return table
function Surface.wire_names_of(operation)
  local names = {}
  for _, declared in ipairs(Surface.params_of(operation)) do
    names[declared.name] = declared.rest:match("^carries%s+`([^`]+)`")
  end
  return names
end

--- What an operation answers, as the generator wrote it: the type of the value, and whether the
--- operation answers a list of them rather than one. Nothing at all when it answers nothing.
--- @param operation function
--- @return string|nil, boolean
function Surface.answered(operation)
  local written = Surface.returned_of(operation)
  if written == "nil" then
    return nil, false
  end
  local listed = written:match("^(.+)%[%]$")
  if listed ~= nil then
    return listed, true
  end
  return written, false
end

--- How a value the API answered is written back into the document it came out of.
--- @param value any
--- @return any
function Surface.written(value)
  if type(value) == "table" and type(value.to_table) == "function" then
    return value:to_table()
  end
  if Json.is_array(value) then
    local written = {}
    for index = 1, #value do
      written[index] = Surface.written(value[index])
    end
    return Json.array(written)
  end
  return value
end

return Surface
