--- The Lua client against a Hook0 that is really running.
---
--- Three things the loopback suite cannot ask: whether an application secret the API minted is
--- accepted, whether a second send under an identifier already ingested is reported as the conflict
--- it is, and whether a signature the output worker computed verifies. Everything else about this
--- client is settled by `clients/lua/spec`.

--- Where this smoke sits, so it runs the same wherever it is started from.
---
--- The directory is whatever the interpreter was handed, which is nothing at all when the file was
--- named without one — as it is when the harness starts it with this directory as the working one.
local source = debug.getinfo(1, "S").source:sub(2)
local here = source:match("^(.*)[/\\][^/\\]*$") or "."
local client_root = here .. "/../../../clients/lua"

--- Loads the rock the way installing it would.
---
--- A searcher rather than a `package.path`, because the rock's module names and its source layout
--- do not follow one another closely enough for a path to express: `hook0` is `src/hook0.lua`, and
--- everything under it drops the prefix. That correspondence is not assumed — `spec/rockspec_spec`
--- walks `src` and fails when the rockspec and the tree disagree — so applying it here loads what
--- installing the rock would put in place. Ahead of the filesystem searchers, so a rock installed
--- on the machine is never the one exercised.
table.insert(package.searchers, 2, function(name)
  local under = name == "hook0" and "hook0" or name:match("^hook0%.(.+)$")
  if under == nil then
    return "\n\t`" .. name .. "` is not a module of this client"
  end
  local path = client_root .. "/src/" .. under:gsub("%.", "/") .. ".lua"
  local chunk, refused = loadfile(path)
  if chunk == nil then
    return "\n\t" .. tostring(refused)
  end
  return chunk, path
end)

local Hook0 = require("hook0")

--- The conflict the API answers a duplicated ingestion with.
local ALREADY_INGESTED = "EventAlreadyIngested"

--- Reads a whole file, refusing one larger than this smoke reads back.
local MAX_PART_BYTES = 1024 * 1024

--- A setting the harness passes, or a refusal naming it.
local function setting(name)
  local value = os.getenv(name)
  if value == nil or value == "" then
    io.stderr:write(name .. " is not set\n")
    os.exit(1)
  end
  return value
end

--- One part of the delivery, as the harness wrote it down.
local function read(delivery, part)
  local handle = assert(io.open(delivery .. "/" .. part, "rb"), part .. " is not readable")
  local what = handle:read(MAX_PART_BYTES)
  handle:close()
  return what or ""
end

--- The event both sends carry, under the identifier the caller names.
local function event(event_type, event_id)
  return {
    event_id = event_id,
    event_type = event_type,
    payload = '{"from":"the lua smoke"}',
    payload_content_type = "application/json",
    labels = { language = "lua" },
  }
end

--- Verifies what the output worker really delivered, with this client's own verification.
local function verify(delivery)
  local headers = {}
  for line in (read(delivery, "headers") .. "\n"):gmatch("([^\n]*)\n") do
    local name, value = line:match("^([^:]+): (.*)$")
    if name ~= nil then
      headers[name] = value
    end
  end

  Hook0.verify_webhook_signature(
    (read(delivery, "signature"):gsub("%s+$", "")),
    read(delivery, "body"),
    headers,
    (read(delivery, "secret"):gsub("%s+$", "")),
    tonumber((read(delivery, "tolerance"):gsub("%s+$", "")))
  )
end

local client = Hook0.Client.new(
  setting("HOOK0_API_URL"),
  setting("HOOK0_APPLICATION_ID"),
  setting("HOOK0_TOKEN")
)
local event_type = setting("HOOK0_EVENT_TYPE")

local sent = client:send_event(event(event_type, nil))
print("ingested " .. sent)

local accepted, raised = pcall(function()
  return client:send_event(event(event_type, sent))
end)
if accepted then
  io.stderr:write("sending the same event twice was accepted twice\n")
  os.exit(1)
end
local said = Hook0.message(raised)
if not said:find(ALREADY_INGESTED, 1, true) then
  io.stderr:write("the second send failed without naming " .. ALREADY_INGESTED .. ": " .. said .. "\n")
  os.exit(1)
end
print("the second send reported " .. ALREADY_INGESTED)

verify(setting("HOOK0_DELIVERY"))
print("the signature the instance produced verifies")
