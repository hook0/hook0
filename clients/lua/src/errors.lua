--- What this client raises, and how a caller tells one failure from another.
---
--- Lua's `error` takes any value, and most libraries hand it a string — which leaves a caller
--- matching on wording. Nothing here does that. A failure is a table carrying the kind it is, the
--- text a human reads, and whatever the failure knows about itself: the status the API answered
--- under, the problem document it carried, the cause a request that got no answer had. A caller
--- matches on the kind, and the wording stays free to improve.
---
--- Kinds form a chain rather than a set: every problem the API can report is a kind of
--- `ProblemError`, which is a kind of `ClientError`, so `Errors.is(raised, Hook0.ClientError)` is one
--- test that covers everything this client raises. The chain is what the generated half declares its
--- problems under, and it is the only thing the two halves agree on here.

local Errors = {}

--- Longest text a raised failure carries. What a message is built from includes bodies a server this
--- client does not control answered, so the message itself is bounded rather than the pieces alone.
Errors.MAX_MESSAGE_BYTES = 4096

--- Deepest a chain of kinds may go, which bounds every walk over one.
Errors.MAX_KIND_DEPTH = 32

local Kind = {}
Kind.__index = Kind

function Kind.__tostring(kind)
  return kind.name
end

--- One kind of failure, under the kind it derives from.
---
--- @param name string what the kind is called, which is what a message prints
--- @param parent table|nil the kind this one is a kind of
--- @return table
function Errors.kind(name, parent)
  if type(name) ~= "string" or name == "" then
    error("a kind is named by a string", 2)
  end
  return setmetatable({ name = name, parent = parent }, Kind)
end

--- Every failure this client raises is one of these.
Errors.ClientError = Errors.kind("ClientError")

local Raised = {}
Raised.__index = Raised

function Raised.__tostring(raised)
  return raised.message
end

--- How much text a raised failure keeps.
local function bounded(message)
  local written = tostring(message)
  if #written <= Errors.MAX_MESSAGE_BYTES then
    return written
  end
  return written:sub(1, Errors.MAX_MESSAGE_BYTES) .. "…"
end

--- A failure of that kind, built rather than raised.
---
--- @param kind table
--- @param message string what to say about it
--- @param extra table|nil what the failure knows about itself
--- @return table
function Errors.new(kind, message, extra)
  if getmetatable(kind) ~= Kind then
    error("a failure is raised under a kind", 2)
  end

  local raised = setmetatable({ kind = kind, message = bounded(message) }, Raised)
  if extra ~= nil then
    for name, value in pairs(extra) do
      if name ~= "kind" and name ~= "message" then
        raised[name] = value
      end
    end
  end
  return raised
end

--- Raises a failure of that kind, carrying what the API answered.
---
--- The shape the generated half calls: a status and, when this client could read one, the problem
--- document the API answered.
---
--- @param kind table
--- @param message string
--- @param status integer|nil
--- @param problem table|nil
function Errors.raise(kind, message, status, problem)
  error(Errors.new(kind, message, { status = status, problem = problem }), 0)
end

--- Raises a failure of that kind, carrying whatever it knows about itself.
---
--- @param kind table
--- @param message string
--- @param extra table|nil
function Errors.throw(kind, message, extra)
  error(Errors.new(kind, message, extra), 0)
end

--- Whether a raised value is one this client raised.
--- @param raised any
--- @return boolean
function Errors.raised(raised)
  return getmetatable(raised) == Raised
end

--- Whether a raised value is of that kind, or of a kind deriving from it.
---
--- @param raised any what a `pcall` handed back
--- @param kind table
--- @return boolean
function Errors.is(raised, kind)
  if not Errors.raised(raised) or getmetatable(kind) ~= Kind then
    return false
  end

  local walked = raised.kind
  for _ = 1, Errors.MAX_KIND_DEPTH do
    if walked == nil then
      return false
    end
    if walked == kind then
      return true
    end
    walked = walked.parent
  end
  return false
end

--- What to say about any raised value, whether or not this client raised it.
--- @param raised any
--- @return string
function Errors.message(raised)
  if Errors.raised(raised) then
    return raised.message
  end
  return bounded(raised)
end

return Errors
