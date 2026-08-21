--- A Hook0 API on a loopback port, and what every case is written against.
---
--- Every case below goes over a real socket: the request the client builds, the headers it sets, the
--- way it reads an answer and the way it gives up on one are all the real ones. Nothing here stands
--- in for a part of the client, so a case that passes says the client works rather than that it was
--- called.
---
--- The API runs in a second process rather than in a thread, because Lua has no threads: a server
--- sharing this one would never be scheduled while the client sat blocked on a socket it had not
--- answered yet. The child speaks as much HTTP/1.1 as one exchange needs, records what it was sent,
--- and hands it all back when it is stopped.
---
--- The module map this suite loads the library through is read out of the rockspec rather than
--- written down again, so what the cases exercise is what installing the rock would put in place.

local Helper = {}

--- Where this client sits, worked out from where this file sits rather than from a working
--- directory, so the suite runs the same wherever it is started from.
local here = debug.getinfo(1, "S").source:match("^@(.*)[/\\][^/\\]*$")
Helper.CLIENT_ROOT = here .. "/.."

--- Where the shared contract every SDK is held to sits, which is beside the clients that read it.
Helper.CORPUS = Helper.CLIENT_ROOT .. "/../conformance"

--- Largest document of the corpus read back. The corpus is committed, so one above this is one that
--- grew out of shape rather than one somebody meant.
Helper.MAX_CORPUS_BYTES = 512 * 1024

--- Longest a case waits for the API in the second process to say where it is listening.
Helper.MAX_STARTUP_SECONDS = 20

--- Reads a whole file, refusing one larger than this suite reads back.
--- @param path string
--- @param limit integer
--- @return string
function Helper.read_file(path, limit)
  local handle = assert(io.open(path, "rb"), path .. " is not readable")
  local read = handle:read(limit + 1)
  handle:close()
  assert(read ~= nil, path .. " is empty")
  assert(#read <= limit, path .. " is longer than the " .. limit .. " bytes read back")
  return read
end

--- The rockspec of this client, found rather than named: its file carries the version, so naming it
--- here would be a second place a release has to be remembered.
--- @return table, string what the rockspec declares, and what it is called
function Helper.rockspec()
  local lfs = require("lfs")

  local found = nil
  for entry in lfs.dir(Helper.CLIENT_ROOT) do
    if entry:match("%.rockspec$") then
      assert(found == nil, "this client carries more than one rockspec: " .. tostring(found) .. " and " .. entry)
      found = entry
    end
  end
  assert(found ~= nil, "this client carries no rockspec")

  local declared = {}
  local chunk = assert(loadfile(Helper.CLIENT_ROOT .. "/" .. found, "t", declared))
  chunk()
  return declared, found
end

--- Loads this library the way installing the rock would, out of the map the rockspec declares.
---
--- A searcher rather than a path: the rockspec maps a module name onto a source path, and the two do
--- not follow one another closely enough for `package.path` to express. Putting it ahead of the
--- filesystem searchers is what keeps a rock installed on the machine from being the one tested.
local function install_searcher()
  local declared = Helper.rockspec()
  local modules = declared.build.modules

  table.insert(package.searchers, 2, function(name)
    local path = modules[name]
    if path == nil then
      return "\n\tno module `" .. name .. "` in the rockspec of this client"
    end
    local chunk, refused = loadfile(Helper.CLIENT_ROOT .. "/" .. path)
    if chunk == nil then
      return "\n\t" .. tostring(refused)
    end
    return chunk, path
  end)
end

install_searcher()

local Json = require("hook0.json")
local Hook0 = require("hook0")

Helper.Json = Json
Helper.Hook0 = Hook0

--- One document of the shared contract, bounded before it is parsed.
--- @param name string
--- @return table
function Helper.contract(name)
  return Json.decode(Helper.read_file(Helper.CORPUS .. "/" .. name, Helper.MAX_CORPUS_BYTES))
end

--- The counter-examples worth keeping, committed beside the properties they broke.
---
--- One JSON value per line, so that a header carrying a comma, a newline or nothing at all is read
--- back exactly as it was written down.
--- @param name string
--- @return table
function Helper.regressions(name)
  local written = Helper.read_file(here .. "/regressions/" .. name .. ".jsonl", Helper.MAX_CORPUS_BYTES)
  local read = {}
  for line in written:gmatch("[^\n]+") do
    if line:match("%S") then
      read[#read + 1] = Json.decode(line)
    end
  end
  return read
end

--- The interpreter running this suite, which is what the second process is started with.
---
--- Taken from the argument vector rather than guessed at: whatever launched this — `lua`, `lua5.4`,
--- a path inside a container image — is what the API in the other process has to be launched with
--- too, and a name written down here would be right on one machine.
--- @return string
function Helper.interpreter()
  local lowest = 0
  while arg[lowest - 1] ~= nil do
    lowest = lowest - 1
  end
  assert(type(arg[lowest]) == "string", "nothing in the argument vector names the interpreter")
  return arg[lowest]
end

--- What a shell reads as one argument, whatever the text carries.
local function quoted(text)
  return "'" .. tostring(text):gsub("'", "'\\''") .. "'"
end

local FakeApi = {}
FakeApi.__index = FakeApi
Helper.FakeApi = FakeApi

--- A Hook0 API listening on a loopback port for the lifetime of one case.
---
--- @param responses table what it answers, in the order it answers them: `status`, `body`,
---   `held_for` and `headers`
--- @return table
function FakeApi.new(responses)
  local plan = os.tmpname()
  local handle = assert(io.open(plan, "wb"))
  handle:write(Json.encode(Json.object({ responses = Json.array(responses or {}) })))
  handle:close()

  local command = table.concat({
    "LUA_PATH=" .. quoted(package.path),
    "LUA_CPATH=" .. quoted(package.cpath),
    quoted(Helper.interpreter()),
    quoted(here .. "/fake_api.lua"),
    quoted(Helper.CLIENT_ROOT),
    quoted(plan),
  }, " ")

  local child = assert(io.popen(command, "r"))
  local announced = child:read("l")
  assert(announced ~= nil, "the API in the other process said nothing about where it is listening")

  local port = math.tointeger(tonumber(announced:match("^port (%d+)$") or ""))
  assert(port ~= nil, "the API in the other process announced `" .. announced .. "`")

  return setmetatable({ child = child, plan = plan, port = port, stopped = false }, FakeApi)
end

--- Where the client reaches this API.
--- @return string
function FakeApi:base_url()
  return "http://127.0.0.1:" .. self.port
end

--- Stops the API and answers what it received, in the order it received it.
--- @return table
function FakeApi:stop()
  if self.stopped then
    return self.received
  end
  self.stopped = true

  local socket = require("socket")
  local connection = socket.tcp()
  connection:settimeout(Helper.MAX_STARTUP_SECONDS)
  if connection:connect("127.0.0.1", self.port) then
    connection:send("GET /__stop HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
    connection:receive("*a")
  end
  connection:close()

  local written = self.child:read("a") or ""
  self.child:close()
  os.remove(self.plan)

  local received = {}
  for line in written:gmatch("[^\n]+") do
    if line:match("%S") then
      received[#received + 1] = Json.decode(line)
    end
  end
  self.received = received
  return received
end

--- A schedule short enough that a case spends its time on requests rather than on waiting.
---
--- Its budget sits far above what its delays add up to, so the number of attempts a case observes is
--- the one its policy asked for rather than the one its budget allowed.
--- @param max_attempts integer|nil
--- @return table
function Helper.retries(max_attempts)
  return Hook0.RetryPolicy.new({
    max_attempts = max_attempts or 4,
    initial_backoff = 0.005,
    max_backoff = 0.005,
    max_total_delay = 1.0,
  })
end

--- The bounds a case holds one send to.
--- @param chosen table|nil
--- @return table
function Helper.options(chosen)
  local asked = chosen or {}
  local built = {
    retry_policy = asked.retry_policy or Helper.retries(asked.max_attempts),
    request_timeout = asked.request_timeout or 5.0,
  }
  for _, name in ipairs({
    "max_payload_bytes", "max_response_bytes", "max_response_headers", "max_header_bytes", "max_head_bytes",
  }) do
    built[name] = asked[name]
  end
  return Hook0.Options.new(built)
end

--- A client pointed at that API.
--- @param api table
--- @param chosen table|nil
--- @return table
function Helper.client(api, chosen)
  return Hook0.Client.new(api:base_url(), "app-123", "token-xyz", chosen or Helper.options())
end

--- An event a case sends.
--- @param overrides table|nil
--- @return table
function Helper.an_event(overrides)
  local event = {
    event_type = "auth.user.create",
    payload = '{"email": "test@example.com"}',
    payload_content_type = "application/json",
    labels = { environment = "production" },
  }
  for name, value in pairs(overrides or {}) do
    event[name] = value
  end
  return event
end

--- What the API answers when it takes the event.
--- @param event_id string
--- @return table
function Helper.ingested(event_id)
  return {
    status = 201,
    body = Json.object({ application_id = "app-123", event_id = event_id, received_at = "2026-01-01" }),
  }
end

--- What the API says when it refuses a request, in the shape every Hook0 failure takes.
--- @param status integer
--- @param problem string
--- @param headers table|nil
--- @return table
function Helper.refusal(status, problem, headers)
  return {
    status = status,
    headers = Json.array(headers or {}),
    body = Json.object({
      id = problem,
      status = status,
      title = "refused",
      detail = "what the corpus scripted",
      type = "https://hook0.com/documentation/errors/" .. problem,
    }),
  }
end

--- Runs something that is expected to raise, and answers what it raised.
--- @param run function
--- @return any
function Helper.refused(run)
  local ok, raised = pcall(run)
  assert(not ok, "nothing was raised where a failure was expected")
  return raised
end

return Helper
