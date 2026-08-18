--- What the API declares, read out of the document the generator was run against.
---
--- Nothing here names an operation. The document is the one place that says which requests exist, so
--- it is what the suite holds the generated half to: an operation the API grows is one more entry
--- here the moment the snapshot carries it, and one it drops takes its entry with it.

local Helper = require("spec.helper")
local Surface = require("spec.generated_surface")

local Json = Helper.Json

local Document = {}

--- The tag that marks an operation as part of the surface an SDK exposes, which is the rule the
--- generator applies — see `PUBLIC_TAG` in `clients/sdkgen/src/snapshot.rs`.
Document.PUBLIC_TAG = "public"

--- The methods a request line can carry, which is what tells an operation from the rest.
local VERBS = {
  get = true,
  put = true,
  post = true,
  delete = true,
  options = true,
  head = true,
  patch = true,
  trace = true,
}

--- The path of a request line, without the query it carries.
--- @param target string
--- @return string
local function path_of(target)
  return (target:gsub("%?.*$", ""))
end

--- Every segment of a path, the empty ones included.
--- @param written string
--- @return table
local function segments_of(written)
  local found = {}
  for segment in (written .. "/"):gmatch("([^/]*)/") do
    found[#found + 1] = segment
  end
  return found
end

--- One operation the document declares, as a request has to look to be it.
local Declared = {}
Declared.__index = Declared

--- How this operation reads in a message, which is what names it when one fails.
--- @return string
function Declared:named()
  return self.verb .. " " .. self.template
end

--- Every name the query may carry, whether the operation requires it or not.
--- @param with_optional boolean
--- @return table
function Declared:query_names(with_optional)
  local names = {}
  for _, name in ipairs(self.required_query) do
    names[#names + 1] = name
  end
  if with_optional then
    for _, name in ipairs(self.optional_query) do
      names[#names + 1] = name
    end
  end
  table.sort(names)
  return names
end

--- The segments a request landed on, in the order the template writes them.
--- @param target string
--- @return table
function Declared.segments_of(target)
  return segments_of(path_of(target))
end

--- Whether a request landed on this operation.
--- @param verb string
--- @param target string
--- @return boolean
function Declared:matches(verb, target)
  if verb ~= self.verb then
    return false
  end

  local wanted = segments_of(self.template)
  local sent = segments_of(path_of(target))
  if #wanted ~= #sent then
    return false
  end

  for index = 1, #wanted do
    local declared = wanted[index]
    if declared:match("^{.*}$") then
      -- A parameter stands for a segment that is there; an empty one is the trailing slash of
      -- another path rather than a value.
      if sent[index] == "" then
        return false
      end
    elseif declared ~= sent[index] then
      return false
    end
  end
  return true
end

Document.Declared = Declared

local operations = nil

--- Every operation an SDK is built out of.
---
--- A document that marks nothing public exposes all of itself, and one that marks anything exposes
--- what it marked. Both are what the generator does with the tag.
--- @return table
function Document.operations()
  if operations ~= nil then
    return operations
  end

  local read = Json.decode(Helper.read_file(Surface.DOCUMENT, Surface.MAX_DOCUMENT_BYTES))
  local found = {}
  local public = {}
  for template, item in pairs(read.paths) do
    for verb, operation in pairs(item) do
      if VERBS[verb] then
        local required, optional = {}, {}
        for _, parameter in ipairs(operation.parameters or {}) do
          if parameter["in"] == "query" then
            local into = parameter.required == true and required or optional
            into[#into + 1] = parameter.name
          end
        end
        table.sort(required)
        table.sort(optional)

        local declared = setmetatable({
          verb = verb:upper(),
          template = template,
          required_query = required,
          optional_query = optional,
        }, Declared)
        found[#found + 1] = declared
        for _, tag in ipairs(operation.tags or {}) do
          if tag == Document.PUBLIC_TAG then
            public[#public + 1] = declared
          end
        end
      end
    end
  end
  assert(#found > 0, "the API document declares no operation at all")

  operations = #public > 0 and public or found
  table.sort(operations, function(one, other)
    return one:named() < other:named()
  end)
  return operations
end

--- Which operation of the document a request landed on, refusing one that is none or many.
--- @param verb string
--- @param target string
--- @return table
function Document.reached(verb, target)
  local matched = nil
  local held = 0
  for _, operation in ipairs(Document.operations()) do
    if operation:matches(verb, target) then
      matched = operation
      held = held + 1
    end
  end
  assert(held == 1, "`" .. verb .. " " .. target .. "` is " .. held .. " of the operations the document declares")
  return matched
end

--- What the query of a request carried, by name.
--- @param target string
--- @return table
function Document.query_of(target)
  local written = target:match("%?(.*)$")
  local carried = {}
  if written == nil or written == "" then
    return carried
  end

  for pair in (written .. "&"):gmatch("([^&]*)&") do
    if pair ~= "" then
      local name, value = pair:match("^([^=]*)=?(.*)$")
      carried[Document.unescaped(name)] = Document.unescaped(value)
    end
  end
  return carried
end

--- The inverse of what the transport writes a request line with.
--- @param written string
--- @return string
function Document.unescaped(written)
  return (written:gsub("%%(%x%x)", function(escaped)
    return string.char(tonumber(escaped, 16))
  end))
end

return Document
