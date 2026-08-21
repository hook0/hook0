--- The Hook0 API a case runs against, in a process of its own.
---
--- Lua has no threads, so a server sharing the suite's process would never be scheduled while the
--- client sat blocked on a socket nobody had answered. This runs beside it instead: it announces the
--- port it is listening on, answers what the case scripted, records what it was sent, and hands all
--- of it back on standard output when the case stops it.
---
--- It is a plain socket speaking as much HTTP/1.1 as one exchange needs. Nothing here is a stand-in
--- for a part of the client: what the client writes on the wire is what this reads.
---
--- Started as: <interpreter> fake_api.lua <client root> <plan file>

local socket = require("socket")

local root = assert(arg[1], "the client root is the first argument")
local plan_file = assert(arg[2], "the plan is the second argument")

local Json = assert(loadfile(root .. "/src/json.lua"))()

--- Longest this API waits for a case to reach it before it gives up on its own.
local MAX_IDLE_SECONDS = 30

--- Most connections one case opens, which bounds what this holds at once.
local MAX_CONNECTIONS = 64

--- Longest request line or header line read.
local MAX_LINE_BYTES = 8 * 1024

--- Most header lines read out of one request.
local MAX_HEADERS = 64

--- Largest request body read.
local MAX_REQUEST_BODY_BYTES = 64 * 1024

--- Where a case says it wants this to stop, and where it is handed what arrived.
local STOP_TARGET = "/__stop"

local function read_plan()
  local handle = assert(io.open(plan_file, "rb"))
  local written = handle:read("a")
  handle:close()
  return Json.decode(written)
end

local plan = read_plan()
local scripted = plan.responses or {}
local answered = 0
local received = {}

local server = assert(socket.bind("127.0.0.1", 0))
local _, port = server:getsockname()
io.stdout:setvbuf("line")
io.stdout:write("port " .. port .. "\n")
io.stdout:flush()

local function read_line(connection)
  local line, failed = connection:receive("*l")
  if line == nil then
    error("the connection closed mid-request: " .. tostring(failed), 0)
  end
  if #line > MAX_LINE_BYTES then
    error("a case sent a line longer than the " .. MAX_LINE_BYTES .. " read", 0)
  end
  return line
end

local function read_request(connection)
  local verb, target = read_line(connection):match("^(%S+)%s+(%S+)")
  if verb == nil then
    error("a case sent something that is not a request line", 0)
  end

  local headers = {}
  for _ = 1, MAX_HEADERS do
    local line = read_line(connection)
    if line == "" then
      break
    end
    local name, value = line:match("^([^:]+):%s*(.*)$")
    if name ~= nil then
      headers[name:lower()] = (value:gsub("%s+$", ""))
    end
  end

  local length = math.tointeger(tonumber(headers["content-length"] or "0")) or 0
  if length > MAX_REQUEST_BODY_BYTES then
    error("a case sent more than the " .. MAX_REQUEST_BODY_BYTES .. " bytes read", 0)
  end

  local body = ""
  if length > 0 then
    body = connection:receive(length) or ""
  end
  return { verb = verb, target = target, headers = Json.object(headers), body = body }
end

--- What this answers one request, which is the next thing the case scripted.
local function next_response()
  answered = answered + 1
  local chosen = scripted[answered]
  if chosen == nil then
    return { status = 500, body = Json.object({ error = "the case scripted no answer for this request" }) }
  end
  return chosen
end

local function written_body(body)
  if body == nil then
    return ""
  end
  if type(body) == "string" then
    return body
  end
  return Json.encode(body)
end

local function answer(connection, response)
  if type(response.held_for) == "number" and response.held_for > 0 then
    socket.sleep(response.held_for)
  end
  -- A socket that hangs up mid-exchange, which is one of the ways the corpus names for a request
  -- that got no answer at all.
  if response.close then
    return
  end

  local body = written_body(response.body)
  local head = {
    "HTTP/1.1 " .. math.tointeger(response.status) .. " Answer",
    "Content-Type: application/json",
    "Content-Length: " .. #body,
  }
  for index = 1, #(response.headers or {}) do
    local pair = response.headers[index]
    head[#head + 1] = tostring(pair[1]) .. ": " .. tostring(pair[2])
  end
  head[#head + 1] = "Connection: close"

  connection:send(table.concat(head, "\r\n") .. "\r\n\r\n" .. body)
end

--- Hands the case everything that arrived, one JSON value per line, and stops.
local function stop()
  for index = 1, #received do
    io.stdout:write(Json.encode(received[index]) .. "\n")
  end
  io.stdout:flush()
  server:close()
  os.exit(0)
end

server:settimeout(MAX_IDLE_SECONDS)
for _ = 1, MAX_CONNECTIONS do
  local connection, failed = server:accept()
  if connection == nil then
    error("no case reached this API within " .. MAX_IDLE_SECONDS .. " seconds: " .. tostring(failed), 0)
  end
  connection:settimeout(MAX_IDLE_SECONDS)

  local ok, request = pcall(read_request, connection)
  if ok then
    if request.target:sub(1, #STOP_TARGET) == STOP_TARGET then
      connection:send("HTTP/1.1 200 Answer\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
      connection:close()
      stop()
    end

    received[#received + 1] = request
    -- A case that scripted an answer this connection never waits for is the very thing a client
    -- giving up on a held answer walks into, so a failure to write it is not one.
    pcall(answer, connection, next_response())
  end
  connection:close()
end

error("a case opened more than the " .. MAX_CONNECTIONS .. " connections this API answers", 0)
