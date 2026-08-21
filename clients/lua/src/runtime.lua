--- What the generated half of this rock reads and writes values through.
---
--- Everything here is hand-written and never regenerated. It is the one seam between what the API
--- declares — the tables, the problems and the methods the generator writes under `generated/` — and
--- what it does not: how a JSON document is turned into a value, and what happens to a document that
--- does not say what it was declared to say.
---
--- Reading is deliberately strict. A member the document declares as a string and the API answered
--- as a number stops the read with the name of that member, rather than yielding a value whose
--- documentation lies about what it holds. Every failure of that kind is a `DecodeError`, so a
--- caller has one kind to match whatever the shape of the answer was.
---
--- A reader is a function of one value. The scalar ones are fields, since there is exactly one of
--- each; the ones built around another reader are functions, since there is one per shape.

local Errors = require("hook0.errors")
local Json = require("hook0.json")

local Runtime = {}

--- What the API answered is not what it declares it answers.
Runtime.DecodeError = Errors.kind("DecodeError", Errors.ClientError)

--- Longest fragment of a response body a message carries. Bodies are answered by a server this rock
--- does not control, so they are cut at a fixed budget rather than echoed whole into whatever the
--- caller logs.
Runtime.MAX_PREVIEW_BYTES = 256

--- Largest JSON document read out of a response body, in bytes. The transport caps what it reads off
--- a socket; this caps what is handed to the parser whichever way the bytes arrived.
Runtime.MAX_PAYLOAD_BYTES = 8 * 1024 * 1024

--- Deepest a JSON document may nest before the parser gives up.
Runtime.MAX_PAYLOAD_NESTING = Json.MAX_DEPTH

--- Most values one query string carries, which bounds what one request line costs.
Runtime.MAX_QUERY_PAIRS = 128

local function refuse(message)
  Errors.throw(Runtime.DecodeError, message)
end

--- What a value is, in the words a message uses.
local function named_type(value)
  if value == Json.null then
    return "null"
  end
  return type(value)
end

--- A string, refusing what merely spells like one.
--- @param value any
--- @return string
function Runtime.TEXT(value)
  if type(value) ~= "string" then
    refuse("expected a string, got " .. named_type(value))
  end
  return value
end

--- A whole number. A boolean is not one, here or on the wire, and neither is a number the document
--- wrote with a fractional part.
--- @param value any
--- @return integer
function Runtime.INTEGER(value)
  if math.type(value) ~= "integer" then
    refuse("expected a whole number, got " .. named_type(value))
  end
  return value
end

--- A number, whether the document wrote it with a fractional part or not.
--- @param value any
--- @return number
function Runtime.NUMBER(value)
  if type(value) ~= "number" then
    refuse("expected a number, got " .. named_type(value))
  end
  return value + 0.0
end

--- A boolean, refusing the numbers that stand in for one elsewhere.
--- @param value any
--- @return boolean
function Runtime.BOOLEAN(value)
  if type(value) ~= "boolean" then
    refuse("expected a boolean, got " .. named_type(value))
  end
  return value
end

--- A value the document does not describe, which is therefore kept as it arrived.
--- @param value any
--- @return any
function Runtime.JSON_VALUE(value)
  return value
end

--- As much of a response body as a message may carry.
--- @param payload any
--- @return string
function Runtime.preview(payload)
  local written = tostring(payload)
  if #written <= Runtime.MAX_PREVIEW_BYTES then
    return written
  end
  return written:sub(1, Runtime.MAX_PREVIEW_BYTES) .. "…"
end

--- What to say about an answer the API document does not describe.
--- @param status integer
--- @param payload string
--- @return string
function Runtime.unreadable(status, payload)
  return "the API answered " .. tostring(status) .. " with a body this client cannot read: " .. Runtime.preview(payload)
end

--- What to say about a problem the API reported.
--- @param status integer
--- @param problem table the problem document the API answered
--- @return string
function Runtime.reported(status, problem)
  local ok, written = Json.try_encode(problem:to_table())
  return "the API answered " .. tostring(status) .. ": " .. (ok and Runtime.preview(written) or "a problem it carried")
end

--- The JSON document a response body carries.
--- @param payload string
--- @return any
function Runtime.decode_payload(payload)
  if type(payload) ~= "string" then
    refuse("the response is " .. type(payload) .. ", not a body")
  end
  if #payload > Runtime.MAX_PAYLOAD_BYTES then
    refuse("the response is " .. #payload .. " bytes, above the " .. Runtime.MAX_PAYLOAD_BYTES .. " accepted")
  end

  local ok, read = Json.try_decode(payload)
  if not ok then
    local reason = Json.reason(read) or Errors.message(read)
    refuse("the response is not JSON: " .. Runtime.preview(payload) .. " (" .. reason .. ")")
  end
  return read
end

--- The problem document a body names, or nothing when this client cannot read one out of it.
---
--- What the generated half calls before it decides which failure to raise: a body that is not the
--- problem shape is a body naming no problem, rather than a second failure on top of the first.
---
--- @param schema table the generated table for the error contract
--- @param payload string the body the API answered
--- @return table|nil
function Runtime.problem_of(schema, payload)
  local ok, read = pcall(function()
    return schema.from_json(Runtime.decode_payload(payload))
  end)
  if ok then
    return read
  end
  return nil
end

--- The members of an object the document declares, under the name it declares it with.
--- @param value any
--- @param owner string what the document calls the object being read
--- @return table
function Runtime.as_fields(value, owner)
  if type(value) ~= "table" or Json.is_array(value) or value == Json.null then
    refuse(owner .. " is not a JSON object")
  end
  return value
end

--- Reads a member, saying which member it was that could not be read.
local function named(key, read)
  local ok, answered = pcall(read)
  if ok then
    return answered
  end
  if Errors.is(answered, Runtime.DecodeError) then
    refuse("`" .. key .. "`: " .. answered.message)
  end
  error(answered, 0)
end

--- A member the document requires, which is therefore missing when it is absent.
--- @param fields table
--- @param key string the name the member travels under
--- @param reader function
--- @return any
function Runtime.read(fields, key, reader)
  local carried = fields[key]
  if carried == nil then
    refuse("`" .. key .. "` is required and was not answered")
  end
  return named(key, function()
    return reader(carried)
  end)
end

--- A member the document does not require, absent as readily as answered as null.
--- @param fields table
--- @param key string the name the member travels under
--- @param reader function
--- @return any|nil
function Runtime.maybe(fields, key, reader)
  local carried = fields[key]
  if carried == nil or carried == Json.null then
    return nil
  end
  return named(key, function()
    return reader(carried)
  end)
end

--- Every item of an array, each one read the same way.
--- @param reader function
--- @return function
function Runtime.list(reader)
  return function(value)
    if type(value) ~= "table" or Json.is_object(value) or value == Json.null then
      refuse("expected an array, got " .. named_type(value))
    end
    local read = {}
    for index = 1, #value do
      read[index] = reader(value[index])
    end
    return read
  end
end

--- Every value of an object whose keys the document leaves open.
--- @param reader function
--- @return function
function Runtime.map(reader)
  return function(value)
    if type(value) ~= "table" or Json.is_array(value) or value == Json.null then
      refuse("expected an object, got " .. named_type(value))
    end
    local read = {}
    for key, carried in pairs(value) do
      read[Runtime.TEXT(key)] = reader(carried)
    end
    return read
  end
end

--- One of the values a closed list declares, refusing anything the list does not carry.
--- @param declared table the table the generator wrote for that list
--- @return function
function Runtime.member_of(declared)
  return function(value)
    local text = Runtime.TEXT(value)
    if not declared.member(text) then
      refuse("`" .. text .. "` is not one of the values that list declares")
    end
    return text
  end
end

--- Whether that list of values carries the one asked about.
--- @param values table
--- @param value any
--- @return boolean
function Runtime.declares(values, value)
  for index = 1, #values do
    if values[index] == value then
      return true
    end
  end
  return false
end

--- That table, as the JSON object it is meant to go out as.
--- @param value table
--- @return table
function Runtime.document(value)
  return Json.object(value)
end

--- A value that travels as it stands.
--- @param value any
--- @return any
function Runtime.itself(value)
  return value
end

--- An emitted value, written back the way the API reads it.
--- @param value table|nil
--- @return table|nil
function Runtime.written(value)
  if value == nil then
    return nil
  end
  return value:to_table()
end

--- Every item of a list, written back and marked as the array it is.
---
--- A table a caller assembled by hand says nothing about whether it is a list or a map, so it is
--- rebuilt rather than passed on: what goes out is what the API reads, whichever way it was built.
---
--- @param value table|nil
--- @param write function
--- @return table|nil
function Runtime.written_list(value, write)
  if value == nil then
    return nil
  end
  local out = {}
  for index = 1, #value do
    out[index] = write(value[index])
  end
  return Json.array(out)
end

--- Every value of a map, written back and marked as the object it is.
--- @param value table|nil
--- @param write function
--- @return table|nil
function Runtime.written_map(value, write)
  if value == nil then
    return nil
  end
  local out = {}
  for key, carried in pairs(value) do
    out[key] = write(carried)
  end
  return Json.object(out)
end

--- Whether two documents carry the same members, however deeply they nest.
--- @param left any
--- @param right any
--- @param depth integer|nil
--- @return boolean
function Runtime.same(left, right, depth)
  local at = depth or 1
  if at > Runtime.MAX_PAYLOAD_NESTING then
    return false
  end
  if left == right then
    return true
  end
  if type(left) ~= "table" or type(right) ~= "table" then
    return false
  end

  for key, carried in pairs(left) do
    if not Runtime.same(carried, right[key], at + 1) then
      return false
    end
  end
  for key in pairs(right) do
    if left[key] == nil then
      return false
    end
  end
  return true
end

--- Whether two emitted values carry the same members, which is what `==` answers on one.
--- @param left any
--- @param right any
--- @return boolean
function Runtime.equality(left, right)
  local function documented(value)
    if type(value) == "table" and type(value.to_table) == "function" then
      return value:to_table()
    end
    return value
  end
  return Runtime.same(documented(left), documented(right))
end

--- The characters a path segment carries as themselves; everything else travels percent-encoded.
local UNRESERVED = "[^A-Za-z0-9%-%._~]"

--- How a value travels in a request line, which is not always how Lua prints it.
--- @param value any
--- @return string
function Runtime.text_of(value)
  if type(value) == "boolean" then
    return tostring(value)
  end
  if math.type(value) == "float" and value == math.floor(value) and math.abs(value) < 2 ^ 53 then
    return string.format("%d", value)
  end
  return tostring(value)
end

--- A value as one segment of a path, with nothing left in it that could name another one.
--- @param value any
--- @return string
function Runtime.path_segment(value)
  return (Runtime.text_of(value):gsub(UNRESERVED, function(character)
    return string.format("%%%02X", character:byte())
  end))
end

--- Where a request lands, with each placeholder of the template filled in.
--- @param template string the path as the document writes it, placeholders included
--- @param filled table the value each placeholder carries
--- @return string
function Runtime.path(template, filled)
  local written = template
  for name, value in pairs(filled or {}) do
    if value == nil then
      refuse("`" .. name .. "` fills a segment of the path and carries nothing")
    end
    written = written:gsub("{" .. name:gsub("%W", "%%%0") .. "}", (Runtime.path_segment(value):gsub("%%", "%%%%")))
  end
  return written
end

--- What travels in the query string: every pair the operation declares that carries a value.
---
--- The pairs are what the emitted method lists, and this is what leaves out the ones the caller
--- passed nothing for — so an optional parameter costs nothing at the call site and nothing on the
--- wire.
---
--- @param pairs_of table|nil name and value pairs, in the order the operation declares them
--- @return table
function Runtime.query(pairs_of)
  local asked = {}
  if pairs_of == nil then
    return asked
  end
  if #pairs_of > Runtime.MAX_QUERY_PAIRS then
    refuse("a request carries more than the " .. Runtime.MAX_QUERY_PAIRS .. " query values accepted")
  end

  for index = 1, #pairs_of do
    local name, value = pairs_of[index][1], pairs_of[index][2]
    if value ~= nil then
      asked[#asked + 1] = { name, Runtime.text_of(value) }
    end
  end
  return asked
end

return Runtime
