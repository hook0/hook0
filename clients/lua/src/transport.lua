--- How a request reaches the API, and what a server on the other end is not allowed to cost.
---
--- The transport answers the status and the bytes and knows nothing of what the API declares:
--- reading those bytes is the generated half's job, and deciding whether to send them again is the
--- client's. That is what lets one HTTP implementation serve both the hand-written event path and
--- every generated method — a generated group calls whatever object it is handed, and this is the
--- one this rock ships.
---
--- The exchange is written here rather than taken from `socket.http`, and that is the whole point:
--- `socket.http` reads a head of any length and a body of any length into memory before a caller is
--- consulted, so a server that is broken or hostile decides how much a client spends. Every ceiling
--- the shared conformance corpus names is applied here instead, on the line that crosses it: how
--- many header lines an answer may carry, how long one of them may be, how large the whole head may
--- come to, and how many bytes of body are read off the socket.

local Errors = require("hook0.errors")
local Json = require("hook0.json")
local Runtime = require("hook0.runtime")
local socket = require("socket")

local Transport = {}
Transport.__index = Transport

--- A request the API never answered, and what caused that.
---
--- The three causes are told apart because only one of them could end differently. A request that
--- got no answer — a connection refused or reset, an attempt out of time, a body that stopped
--- mid-way — says nothing about whether the API acted on it, which is exactly why a send carries an
--- identifier the client chose itself, and why repeating it is safe and worth doing. An answer that
--- crossed a ceiling this client set for itself draws the same answer the second time, and reading it
--- again four times over costs the caller four times as much for the same failure. A URL nothing can
--- be sent to was never sent at all, and a repetition builds the same unusable request, turning a
--- misconfiguration into a message that accuses the network.
---
--- The names are the ones the shared conformance corpus gives them, so the verdict a client applies
--- and the verdict that corpus writes down are the same words.
Transport.TransportError = Errors.kind("TransportError", Errors.ClientError)

--- Longest one attempt at reaching the API is given before it is abandoned, in seconds.
---
--- Ten seconds is far above what ingesting an event takes when the API is healthy, and short enough
--- that a stuck connection does not hold a caller for a noticeable time.
Transport.DEFAULT_REQUEST_TIMEOUT = 10.0

--- Largest response body read off a socket, in bytes.
Transport.DEFAULT_MAX_RESPONSE_BYTES = 8 * 1024 * 1024

--- How many header lines an answer may carry before it is refused. Sixty-four is well above what the
--- API sends.
Transport.DEFAULT_MAX_RESPONSE_HEADERS = 64

--- Longest one header line may be, name and value together, in bytes.
Transport.DEFAULT_MAX_HEADER_BYTES = 64 * 1024

--- Largest whole head an answer may carry, every line counted together, in bytes.
---
--- This is the one that bounds what a head costs. A line count and a size per line multiply:
--- sixty-four lines of sixty-four kilobytes each is four megabytes of head, and both of the bounds
--- above admit it. They earn their place by refusing early, on the line that crosses them rather
--- than at the end of the head; this one sets the ceiling.
---
--- Sixteen kilobytes is what Node enforces by default, and matching it is the point: a lower ceiling
--- would refuse heads another target accepts, and a higher one would not bind there at all, leaving
--- each language a different effective limit.
Transport.DEFAULT_MAX_HEAD_BYTES = 16 * 1024

--- How many bytes of body are asked for at a time. Bounded so that a `Content-Length` a server made
--- up cannot be turned into an allocation before a single byte has arrived.
Transport.CHUNK_BYTES = 32 * 1024

--- What a request body says it carries, and what an answer is asked for in.
Transport.JSON_MEDIA_TYPE = "application/json"

--- Longest each part this client composes its `User-Agent` out of may be, in characters.
---
--- The interpreter and the operating system are described by the platform rather than by this rock,
--- so their length is not this rock's to guarantee: they are cut here so that the header cannot grow
--- with whatever the platform feels like saying. Every part is also stripped of anything the grammar
--- of the header uses as punctuation, so a platform cannot forge a shape it does not have.
local MAX_USER_AGENT_PART_CHARS = 64

--- Longest a duration this client states its retry policy in may be, in milliseconds.
---
--- The three durations of a policy are numbers a caller set, and a header is no place for whatever
--- arithmetic they lead to: about twenty-five days is already past any schedule a send could hold,
--- and cutting to it is what keeps the value an integer whatever was configured.
local MAX_STATED_MILLISECONDS = (1 << 31) - 1

--- The schemes this transport reaches, and the port each one uses when the URL names none.
local PORTS = { http = 80, https = 443 }

--- The API was reached for and answered nothing this client could read to its end.
--- @param detail string
function Transport.no_answer(detail)
  Errors.throw(Transport.TransportError, detail, { cause = "no_answer", retryable = true })
end

--- The API answered, and what it answered crossed a ceiling this client set for itself.
--- @param detail string
function Transport.answer_above_a_bound(detail)
  Errors.throw(Transport.TransportError, detail, { cause = "answer_above_a_bound", retryable = false })
end

--- There is nowhere to send the request, so nothing was sent.
--- @param detail string
function Transport.unusable_api_url(detail)
  Errors.throw(Transport.TransportError, detail, { cause = "unusable_api_url", retryable = false })
end

--- What this transport reaches, read out of a URL.
---
--- Written here rather than taken from `socket.url`, which answers a table of pieces for any text at
--- all: what a transport needs is a refusal for the text it cannot send anything to, and that is one
--- of the three causes the corpus classifies.
---
--- @param url string
--- @return table
local function reachable(url)
  if type(url) ~= "string" then
    Transport.unusable_api_url("the API URL is " .. type(url) .. ", not a URL")
  end

  local scheme, rest = url:match("^(%a[%w+.-]*)://(.*)$")
  if scheme == nil then
    Transport.unusable_api_url("`" .. Runtime.preview(url) .. "` names no scheme this transport can send a request to")
  end

  scheme = scheme:lower()
  if PORTS[scheme] == nil then
    Transport.unusable_api_url("`" .. scheme .. "` is not a scheme this transport can send a request to")
  end

  local authority = rest:match("^([^/?#]*)") or ""
  local target = rest:sub(#authority + 1)
  if authority:find("@", 1, true) then
    authority = authority:sub(authority:find("@", 1, true) + 1)
  end

  local host, port = authority:match("^%[([^%]]*)%]:?(%d*)$")
  if host == nil then
    host, port = authority:match("^([^:]*):?(%d*)$")
  end
  if host == nil or host == "" then
    Transport.unusable_api_url("`" .. Runtime.preview(url) .. "` names no host to send a request to")
  end

  return {
    scheme = scheme,
    host = host,
    port = port ~= "" and math.tointeger(tonumber(port)) or PORTS[scheme],
    target = target ~= "" and target or "/",
  }
end

--- A value as it travels in a query string.
local function encoded(text)
  return (text:gsub("[^A-Za-z0-9%-%._~]", function(character)
    return string.format("%%%02X", character:byte())
  end))
end

--- Where the request lands: the path of the base URL, extended by the operation's own, and then the
--- query the operation asked for.
local function resolved(base_url, path, query)
  local reached = reachable(base_url)
  local written = tostring(path or "")

  if written:sub(1, 1) == "/" then
    reached.target = written
  elseif written ~= "" then
    reached.target = reached.target:gsub("/*$", "") .. "/" .. written
  end

  local asked = {}
  for index = 1, #(query or {}) do
    asked[#asked + 1] = encoded(tostring(query[index][1])) .. "=" .. encoded(tostring(query[index][2]))
  end
  if #asked > 0 then
    local separator = reached.target:find("?", 1, true) and "&" or "?"
    reached.target = reached.target .. separator .. table.concat(asked, "&")
  end

  return reached
end

--- What is left of the budget one attempt was given, set on the socket before every blocking call.
---
--- A timeout set once bounds each call rather than the exchange: a server answering one byte just
--- under it, over and over, would hold a caller for as long as it liked. What is bounded here is the
--- whole attempt, so the arithmetic is done again before every read and every write.
local function within(connection, deadline)
  local left = deadline - socket.gettime()
  if left <= 0 then
    Transport.no_answer("the attempt ran out of the time it was given before the API answered")
  end
  connection:settimeout(left)
  return connection
end

--- One connection to the API, wrapped in TLS when the URL asked for it.
local function connected(reached, deadline)
  local raw, opening = socket.tcp()
  if raw == nil then
    Transport.no_answer("no socket could be opened: " .. tostring(opening))
  end

  local opened, failed = within(raw, deadline):connect(reached.host, reached.port)
  if opened == nil then
    raw:close()
    Transport.no_answer("the API at " .. reached.host .. ":" .. reached.port ..
      " was not reached: " .. tostring(failed))
  end

  if reached.scheme ~= "https" then
    return raw
  end

  local ok, ssl = pcall(require, "ssl")
  if not ok then
    raw:close()
    Transport.unusable_api_url("`https` needs luasec, which is not installed beside this rock")
  end

  local secured, wrapping = ssl.wrap(raw, { mode = "client", protocol = "any", verify = "peer", options = "all" })
  if secured == nil then
    raw:close()
    Transport.no_answer("the connection could not be secured: " .. tostring(wrapping))
  end
  secured:sni(reached.host)

  local shaken, refusing = within(secured, deadline):dohandshake()
  if shaken == nil then
    secured:close()
    Transport.no_answer("the API refused the handshake: " .. tostring(refusing))
  end
  return secured
end

--- One line of the head, refused before it is held whole when it is longer than this client reads.
local function head_line(connection, deadline, max_header_bytes)
  local line, failed, partial = within(connection, deadline):receive("*l")
  if line == nil then
    if partial ~= nil and #partial > max_header_bytes then
      Transport.answer_above_a_bound(
        "the API answered a header above the " .. max_header_bytes .. " bytes read at most")
    end
    Transport.no_answer("the API answered nothing this client could read: " .. tostring(failed))
  end
  if #line > max_header_bytes then
    Transport.answer_above_a_bound("the API answered a header above the " .. max_header_bytes .. " bytes read at most")
  end
  return line
end

--- What an answer carried beside its body, under the names a caller looks them up by.
---
--- Read line by line and refused on the line that crosses a ceiling, rather than after the head has
--- been held whole: a head this client will not accept costs it the line it was refused on.
local function carried(connection, deadline, options)
  local headers = {}
  local held = 0
  local whole = 0

  while true do
    local line = head_line(connection, deadline, options.max_header_bytes)
    if line == "" then
      return headers
    end

    held = held + 1
    if held > options.max_response_headers then
      Transport.answer_above_a_bound(
        "the API answered more than the " .. options.max_response_headers .. " header lines read at most")
    end

    whole = whole + #line
    if whole > options.max_head_bytes then
      Transport.answer_above_a_bound(
        "the API answered a head above the " .. options.max_head_bytes .. " bytes read at most")
    end

    local name, value = line:match("^([^:]+):%s*(.*)$")
    if name ~= nil then
      headers[name:lower()] = (value:gsub("%s+$", ""))
    end
  end
end

--- The body of an answer, up to what this transport agrees to hold.
---
--- Read in bounded pieces whatever the answer says its length is, so a `Content-Length` a server made
--- up costs nothing until the bytes behind it actually arrive.
local function bounded(connection, deadline, headers, max_response_bytes)
  local announced = math.tointeger(tonumber(headers["content-length"] or ""))
  local held = {}
  local size = 0

  while true do
    local wanted = Transport.CHUNK_BYTES
    if announced ~= nil then
      wanted = math.min(wanted, announced - size)
      if wanted <= 0 then
        break
      end
    end

    local read, failed, partial = within(connection, deadline):receive(wanted)
    local piece = read or partial or ""
    size = size + #piece
    if size > max_response_bytes then
      Transport.answer_above_a_bound(
        "the API answered more than the " .. max_response_bytes .. " bytes read at most")
    end
    if #piece > 0 then
      held[#held + 1] = piece
    end

    if read == nil then
      if failed == "closed" then
        break
      end
      Transport.no_answer("the API stopped answering mid-body: " .. tostring(failed))
    end
  end

  return table.concat(held)
end

--- One part of the `User-Agent`, with everything the header's own grammar uses taken out of it and
--- cut to `MAX_USER_AGENT_PART_CHARS`.
--- @param part string
--- @return string
local function clipped(part)
  local kept = part:gsub("[^\32-\126]", ""):gsub("[();]", "")
  return kept:sub(1, MAX_USER_AGENT_PART_CHARS)
end

--- What this rock says it is, composed once: none of the three parts it is built out of can change
--- while a process runs.
local composed = nil

--- Which SDK, at which version, on which runtime and operating system, is talking to the API.
---
--- The version is the one the module this file is a half of declares, rather than a second copy of
--- it here, and it is read at first use rather than at load: that module is assembled out of this
--- one, so requiring it as this file loads would be a cycle. Lua says nothing about the operating
--- system beyond the separator it writes paths with, and neither luasocket nor luasec adds to it,
--- which leaves the two families it can tell apart.
--- @return string
local function user_agent()
  if composed == nil then
    local family = package.config:sub(1, 1) == "\\" and "windows" or "posix"
    composed = "hook0-client-lua/" .. clipped(require("hook0").VERSION) ..
      " (" .. clipped(_VERSION) .. "; " .. clipped(family) .. ")"
  end
  return composed
end

--- One duration of a retry policy, in the whole milliseconds it is stated as.
---
--- What arrives here is already the duration in force, so a value no schedule could be built on has
--- been read as its default before this sees it: what is stated and what is waited are the same
--- number by construction rather than by two rules that agree today. All that is left is the
--- ceiling, which keeps a finite duration nobody meant from deciding how long the header is.
--- @param seconds number
--- @return integer
local function stated_milliseconds(seconds)
  local milliseconds = (tonumber(seconds) or 0) * 1000
  return math.floor(math.min(math.max(milliseconds, 0), MAX_STATED_MILLISECONDS) + 0.5)
end

--- The retry policy a transport was built to serve, as every request states it.
---
--- What it states is what the policy holds rather than what one send went on to do: a policy
--- allowing a single attempt still names the delays it holds, and an instance reading `attempts=1`
--- already knows none of them will be waited. It is the one client setting the API can see the
--- consequences of without being told — a burst of identical requests is a client repeating one
--- send, and nothing else on the wire tells that apart from a client in a loop.
---
--- The grammar is the one `X-Hook0-Signature` already travels under, parts joined by `,` and each
--- cut at its first `=`, so nothing here is a second shape to get wrong.
--- @param policy table
--- @return string
local function client_options(policy)
  return string.format(
    "attempts=%d,backoff=%d,ceiling=%d,budget=%d",
    policy:attempts(),
    stated_milliseconds(policy:initial_backoff_in_force()),
    stated_milliseconds(policy:max_backoff_in_force()),
    stated_milliseconds(policy:max_total_delay_in_force()))
end

--- Builds a transport pointed at one API, under one credential.
---
--- The policy a transport states is the one the client it serves was built with, and the default one
--- where nothing handed it a client. It is asked for here rather than at load, for the reason
--- `user_agent` is: the module that declares a policy is assembled out of this one.
---
--- @param base_url string where the API lives, such as https://app.hook0.com/api/v1
--- @param token string an authentication token valid for that API
--- @param options table|nil the ceilings one exchange is held to
--- @return table
function Transport.new(base_url, token, options)
  local chosen = options or {}
  return setmetatable({
    base_url = base_url,
    token = token,
    timeout = chosen.request_timeout or Transport.DEFAULT_REQUEST_TIMEOUT,
    max_response_bytes = chosen.max_response_bytes or Transport.DEFAULT_MAX_RESPONSE_BYTES,
    max_response_headers = chosen.max_response_headers or Transport.DEFAULT_MAX_RESPONSE_HEADERS,
    max_header_bytes = chosen.max_header_bytes or Transport.DEFAULT_MAX_HEADER_BYTES,
    max_head_bytes = chosen.max_head_bytes or Transport.DEFAULT_MAX_HEAD_BYTES,
    retry_policy = chosen.retry_policy or require("hook0").RetryPolicy.new(),
  }, Transport)
end

--- What the API answered, headers included, whether or not it answered a success.
---
--- Header names are lowercased and a later value wins over an earlier one under the same name, so a
--- caller reads a header without knowing which case the server wrote it in.
---
--- @param method string the HTTP method the operation is issued under
--- @param path string where the request lands, absolute or under the base URL
--- @param query table|nil the name and value pairs of the query string
--- @param body any|nil what to send as a JSON document, or nothing at all
--- @return integer, table, string the status, the headers and the body
function Transport:deliver(method, path, query, body)
  local reached = resolved(self.base_url, path, query)
  local deadline = socket.gettime() + self.timeout

  local written = nil
  if body ~= nil then
    local ok, encoded_body = Json.try_encode(body)
    if not ok then
      -- A caller's mistake rather than a failure of the exchange: nothing about the network would
      -- make the same value encodable the second time, and a send has nothing to decide here.
      Errors.throw(Errors.ClientError, "the body of the request is not something a document can carry")
    end
    written = encoded_body
  end

  local head = {
    string.upper(tostring(method)) .. " " .. reached.target .. " HTTP/1.1",
    "Host: " .. reached.host .. (reached.port == PORTS[reached.scheme] and "" or ":" .. reached.port),
    "Authorization: Bearer " .. tostring(self.token),
    "Accept: " .. Transport.JSON_MEDIA_TYPE,
    "User-Agent: " .. user_agent(),
    "Hook0-Client-Options: " .. client_options(self.retry_policy),
    "Connection: close",
  }
  if written ~= nil then
    head[#head + 1] = "Content-Type: " .. Transport.JSON_MEDIA_TYPE
    head[#head + 1] = "Content-Length: " .. #written
  end

  local connection = connected(reached, deadline)
  local ok, answered = pcall(function()
    local sent, failed = within(connection, deadline):send(
      table.concat(head, "\r\n") .. "\r\n\r\n" .. (written or ""))
    if sent == nil then
      Transport.no_answer("the request could not be sent: " .. tostring(failed))
    end

    local status = head_line(connection, deadline, self.max_header_bytes):match("^HTTP/%d%.%d%s+(%d%d%d)")
    if status == nil then
      Transport.no_answer("the API answered something that is not an HTTP status line")
    end
    local headers = carried(connection, deadline, self)
    return {
      math.tointeger(tonumber(status)),
      headers,
      bounded(connection, deadline, headers, self.max_response_bytes),
    }
  end)

  connection:close()
  if not ok then
    error(answered, 0)
  end
  return answered[1], answered[2], answered[3]
end

--- What the API answered, whether or not it answered a success.
---
--- This is the shape the generated half of this rock reads, which is the status and the bytes. A
--- caller that also needs what the answer carried beside its body — the delay a paced instance names
--- is one — asks `deliver` for it.
---
--- @param method string the HTTP method the operation is issued under
--- @param path string where the request lands, absolute or under the base URL
--- @param query table|nil the name and value pairs of the query string
--- @param body any|nil what to send as a JSON document, or nothing at all
--- @return integer, string the status and the body
function Transport:request(method, path, query, body)
  local status, _, payload = self:deliver(method, path, query, body)
  return status, payload
end

return Transport
