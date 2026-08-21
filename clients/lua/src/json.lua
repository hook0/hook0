--- Reading and writing JSON, bounded, with nothing but the standard library.
---
--- Lua carries no JSON at all, and the two things a table cannot say on its own are exactly the two
--- JSON needs: whether an empty table is an array or an object, and whether a member is absent or
--- carries null. Both are answered here rather than guessed at. A decoded container is marked as one
--- or the other through its metatable, and `Json.null` is the one value standing for a null the
--- document actually carried — `nil` means absent, everywhere, and a table cannot hold it anyway.
---
--- Everything the parser reads is bounded before it is read: the text itself, how deep it nests, and
--- how many members one container may hold. A document that crosses one of those is refused naming
--- the ceiling it crossed rather than parsed as far as it fits.

local Json = {}

--- Longest document read, in bytes.
Json.MAX_TEXT_BYTES = 8 * 1024 * 1024

--- Deepest a document may nest, which is what keeps a text that is nothing but brackets from
--- growing the stack.
Json.MAX_DEPTH = 64

--- Most members one array or one object may carry.
Json.MAX_MEMBERS = 100000

--- What a document carried a null under.
---
--- A distinct value rather than `nil`: a table cannot hold `nil`, so a member answered as null would
--- otherwise be indistinguishable from one the API never sent.
Json.null = setmetatable({}, { __name = "json.null", __tostring = function() return "null" end })

--- What marks a table as the one or the other, so an empty one still says which it is.
local ARRAY = { __name = "json.array" }
local OBJECT = { __name = "json.object" }

--- The text of a JSON document that could not be read.
Json.Error = "json"

local function refuse(reason)
  error({ json = reason }, 0)
end

--- The reason a decode failed, when it failed for a reason this module raised.
--- @param raised any
--- @return string|nil
function Json.reason(raised)
  if type(raised) == "table" and type(raised.json) == "string" then
    return raised.json
  end
  return nil
end

--- That table, read as an array from here on.
--- @param value table
--- @return table
function Json.array(value)
  return setmetatable(value, ARRAY)
end

--- That table, read as an object from here on.
--- @param value table
--- @return table
function Json.object(value)
  return setmetatable(value, OBJECT)
end

--- Whether that table was decoded as, or marked as, an array.
--- @param value any
--- @return boolean
function Json.is_array(value)
  return getmetatable(value) == ARRAY
end

--- Whether that table was decoded as, or marked as, an object.
--- @param value any
--- @return boolean
function Json.is_object(value)
  return getmetatable(value) == OBJECT
end

local ESCAPED = {
  ['"'] = '\\"',
  ["\\"] = "\\\\",
  ["\b"] = "\\b",
  ["\f"] = "\\f",
  ["\n"] = "\\n",
  ["\r"] = "\\r",
  ["\t"] = "\\t",
}

local function escaped(text)
  return (text:gsub("[%z\1-\31\\\"]", function(character)
    return ESCAPED[character] or string.format("\\u%04x", character:byte())
  end))
end

--- Whether a table is a dense sequence, and how long it is.
local function sequence_length(value)
  local count = 0
  for _ in pairs(value) do
    count = count + 1
  end
  for index = 1, count do
    if value[index] == nil then
      return nil
    end
  end
  return count
end

--- The keys of an object, in one order whatever order they were built in.
---
--- Sorted, so that writing the same document twice writes the same bytes: two clients that disagree
--- on the order of a body disagree on the bytes a signature is computed over.
local function ordered_keys(value)
  local keys = {}
  for key in pairs(value) do
    if type(key) ~= "string" then
      refuse("an object carries a key that is not a string")
    end
    keys[#keys + 1] = key
  end
  table.sort(keys)
  return keys
end

local function write(value, out, depth)
  if depth > Json.MAX_DEPTH then
    refuse("the document nests deeper than the " .. Json.MAX_DEPTH .. " accepted")
  end

  if value == Json.null then
    out[#out + 1] = "null"
    return
  end

  local kind = type(value)
  if kind == "nil" then
    out[#out + 1] = "null"
  elseif kind == "boolean" then
    out[#out + 1] = tostring(value)
  elseif kind == "string" then
    out[#out + 1] = '"' .. escaped(value) .. '"'
  elseif kind == "number" then
    if value ~= value or value == math.huge or value == -math.huge then
      refuse("a number the document cannot carry: " .. tostring(value))
    end
    if math.type(value) == "integer" then
      out[#out + 1] = string.format("%d", value)
    else
      out[#out + 1] = string.format("%.14g", value)
    end
  elseif kind == "table" then
    local length = sequence_length(value)
    local as_array = Json.is_array(value) or (not Json.is_object(value) and length ~= nil and length > 0)
    if as_array then
      if length == nil then
        refuse("an array carries a key that is not an index")
      end
      out[#out + 1] = "["
      for index = 1, length do
        if index > 1 then
          out[#out + 1] = ","
        end
        write(value[index], out, depth + 1)
      end
      out[#out + 1] = "]"
    else
      out[#out + 1] = "{"
      local keys = ordered_keys(value)
      for index = 1, #keys do
        if index > 1 then
          out[#out + 1] = ","
        end
        out[#out + 1] = '"' .. escaped(keys[index]) .. '":'
        write(value[keys[index]], out, depth + 1)
      end
      out[#out + 1] = "}"
    end
  else
    refuse("a value of type " .. kind .. " is not something a document carries")
  end
end

--- That value as the text of a JSON document.
---
--- A table is written as an array when it was marked as one or is a non-empty dense sequence, and as
--- an object otherwise — which is why what this client sends is always marked rather than left to be
--- inferred.
---
--- @param value any
--- @return string
--- @raise a table carrying the reason, when the value is not something a document can carry
function Json.encode(value)
  local out = {}
  write(value, out, 1)
  return table.concat(out)
end

--- Whether the encode succeeded, and what it answered or why it did not.
--- @param value any
--- @return boolean, any
function Json.try_encode(value)
  return pcall(Json.encode, value)
end

local Reader = {}
Reader.__index = Reader

local WHITESPACE = { [" "] = true, ["\t"] = true, ["\n"] = true, ["\r"] = true }

local UNESCAPED = {
  ['"'] = '"',
  ["\\"] = "\\",
  ["/"] = "/",
  b = "\b",
  f = "\f",
  n = "\n",
  r = "\r",
  t = "\t",
}

function Reader.new(text)
  return setmetatable({ text = text, at = 1, length = #text }, Reader)
end

function Reader:skip()
  while self.at <= self.length and WHITESPACE[self.text:sub(self.at, self.at)] do
    self.at = self.at + 1
  end
end

function Reader:peek()
  return self.text:sub(self.at, self.at)
end

function Reader:expect(character)
  if self:peek() ~= character then
    refuse("expected `" .. character .. "` at byte " .. self.at)
  end
  self.at = self.at + 1
end

--- One `\u` escape, and the code point it names once a surrogate pair has been put back together.
function Reader:code_point()
  local written = self.text:sub(self.at, self.at + 3)
  if not written:match("^%x%x%x%x$") then
    refuse("an escape at byte " .. self.at .. " is not four hexadecimal digits")
  end
  self.at = self.at + 4

  local point = tonumber(written, 16)
  if point < 0xD800 or point > 0xDBFF then
    return point
  end
  if self.text:sub(self.at, self.at + 1) ~= "\\u" then
    return point
  end

  local trailing = self.text:sub(self.at + 2, self.at + 5)
  if not trailing:match("^%x%x%x%x$") then
    return point
  end
  local low = tonumber(trailing, 16)
  if low < 0xDC00 or low > 0xDFFF then
    return point
  end

  self.at = self.at + 6
  return 0x10000 + (point - 0xD800) * 0x400 + (low - 0xDC00)
end

function Reader:string()
  self:expect('"')

  local pieces = {}
  while true do
    if self.at > self.length then
      refuse("a string is not closed before the document ends")
    end

    local character = self.text:sub(self.at, self.at)
    if character == '"' then
      self.at = self.at + 1
      return table.concat(pieces)
    end

    if character == "\\" then
      self.at = self.at + 1
      local marker = self.text:sub(self.at, self.at)
      self.at = self.at + 1
      if marker == "u" then
        pieces[#pieces + 1] = utf8.char(self:code_point())
      elseif UNESCAPED[marker] then
        pieces[#pieces + 1] = UNESCAPED[marker]
      else
        refuse("`\\" .. marker .. "` at byte " .. (self.at - 1) .. " is not an escape")
      end
    elseif character:byte() < 0x20 then
      refuse("a string carries a control character at byte " .. self.at)
    else
      pieces[#pieces + 1] = character
      self.at = self.at + 1
    end
  end
end

function Reader:number()
  local written = self.text:match("^-?%d+%.?%d*[eE]?[-+]?%d*", self.at)
  if written == nil or written == "" then
    refuse("expected a number at byte " .. self.at)
  end
  local read = tonumber(written)
  if read == nil then
    refuse("`" .. written .. "` at byte " .. self.at .. " is not a number")
  end
  self.at = self.at + #written

  -- A count the document wrote without a fractional part is a whole number here too: writing it
  -- back out as `1.0` is a body the API refuses.
  if not written:find("[%.eE]") then
    return math.tointeger(read) or read
  end
  return read
end

function Reader:literal(written, value)
  if self.text:sub(self.at, self.at + #written - 1) ~= written then
    refuse("expected a value at byte " .. self.at)
  end
  self.at = self.at + #written
  return value
end

function Reader:value(depth)
  if depth > Json.MAX_DEPTH then
    refuse("the document nests deeper than the " .. Json.MAX_DEPTH .. " accepted")
  end
  self:skip()

  local character = self:peek()
  if character == "" then
    refuse("the document ends where a value was expected")
  elseif character == "{" then
    return self:object(depth)
  elseif character == "[" then
    return self:array(depth)
  elseif character == '"' then
    return self:string()
  elseif character == "t" then
    return self:literal("true", true)
  elseif character == "f" then
    return self:literal("false", false)
  elseif character == "n" then
    return self:literal("null", Json.null)
  end
  return self:number()
end

function Reader:array(depth)
  self:expect("[")
  local read = Json.array({})

  self:skip()
  if self:peek() == "]" then
    self.at = self.at + 1
    return read
  end

  local held = 0
  while true do
    held = held + 1
    if held > Json.MAX_MEMBERS then
      refuse("an array carries more than the " .. Json.MAX_MEMBERS .. " members accepted")
    end
    read[held] = self:value(depth + 1)

    self:skip()
    local character = self:peek()
    if character == "]" then
      self.at = self.at + 1
      return read
    end
    if character ~= "," then
      refuse("expected `,` or `]` at byte " .. self.at)
    end
    self.at = self.at + 1
  end
end

function Reader:object(depth)
  self:expect("{")
  local read = Json.object({})

  self:skip()
  if self:peek() == "}" then
    self.at = self.at + 1
    return read
  end

  local held = 0
  while true do
    held = held + 1
    if held > Json.MAX_MEMBERS then
      refuse("an object carries more than the " .. Json.MAX_MEMBERS .. " members accepted")
    end

    self:skip()
    local key = self:string()
    self:skip()
    self:expect(":")
    read[key] = self:value(depth + 1)

    self:skip()
    local character = self:peek()
    if character == "}" then
      self.at = self.at + 1
      return read
    end
    if character ~= "," then
      refuse("expected `,` or `}` at byte " .. self.at)
    end
    self.at = self.at + 1
  end
end

--- The value a JSON document carries.
---
--- @param text string
--- @return any
--- @raise a table carrying the reason, for every way a text fails to be a document
function Json.decode(text)
  if type(text) ~= "string" then
    refuse("a document is " .. type(text) .. ", not text")
  end
  if #text > Json.MAX_TEXT_BYTES then
    refuse("the document is " .. #text .. " bytes long, above the " .. Json.MAX_TEXT_BYTES .. " accepted")
  end

  local reader = Reader.new(text)
  local read = reader:value(1)
  reader:skip()
  if reader.at <= reader.length then
    refuse("the document carries something past its value, at byte " .. reader.at)
  end
  return read
end

--- Whether the decode succeeded, and what it answered or why it did not.
--- @param text string
--- @return boolean, any
function Json.try_decode(text)
  return pcall(Json.decode, text)
end

return Json
