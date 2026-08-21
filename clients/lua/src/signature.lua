--- Verifying that a webhook came from Hook0, and that nothing in it changed on the way.
---
--- A signature names the moment it was signed and one or two message authentication codes over the
--- body. The `v1` scheme also covers a list of request headers, so a receiver can tell apart two
--- deliveries that carry the same body but not the same context; `v0` covers the body alone and is
--- what an older sender still produces. When both are offered, `v1` is the one verified: accepting
--- the weaker of two schemes on the strength of the sender offering it is how a downgrade works.
---
--- Two things are refused before any code is computed. A header the signature says it covers but the
--- request did not carry is refused outright, because signing over an absent value would let a
--- sender drop a header and keep the signature valid. And a signature whose codes are not whole
--- hexadecimal is refused rather than decoded as far as it goes: a decoder that stops at the first
--- bad character compares a prefix, and a prefix of the right code is not the right code.

local Errors = require("hook0.errors")
local Sha256 = require("hook0.sha256")

local Signature = {}

--- Longest signature header read. The header is written by whoever reached the endpoint, so its size
--- is bounded before any of it is split, decoded or compared.
Signature.MAX_SIGNATURE_BYTES = 8 * 1024

--- Most `key=value` parts one signature header is split into.
Signature.MAX_SIGNATURE_PARTS = 32

--- Most header names one signature covers.
Signature.MAX_COVERED_HEADERS = 64

--- Most headers of a delivery read back, which bounds what a caller can hand this.
Signature.MAX_DELIVERED_HEADERS = 512

--- Furthest from the epoch, in either direction, a signature's moment may sit. A header carrying a
--- long run of digits would otherwise reach the arithmetic that holds it against the current time
--- and cost more than reading it did.
Signature.MAX_TIMESTAMP = 1000000000000

--- What separates one part of the signature header from the next.
Signature.PART_SEPARATOR = ","

--- What separates the name of a part from its value. Only the first one counts: a value may hold
--- further ones, and splitting on all of them would silently drop everything past the second.
Signature.PART_ASSIGNATOR = "="

--- What separates two header names inside the `h` part, and what they are joined back with.
Signature.HEADER_NAME_SEPARATOR = " "

--- What separates the pieces of the message a code is computed over.
Signature.MESSAGE_SEPARATOR = "."

--- Part naming the moment the delivery was signed, in whole seconds since the Unix epoch.
Signature.TIMESTAMP_PART = "t"

--- Part carrying the code covering the body alone.
Signature.BODY_SCHEME_PART = "v0"

--- Part carrying the code covering the covered headers and the body.
Signature.HEADERS_SCHEME_PART = "v1"

--- Part listing the headers the `v1` code covers, in the order it covers them.
Signature.COVERED_HEADERS_PART = "h"

local function refuse(message)
  Errors.throw(Errors.ClientError, message)
end

--- The text either side of the first assignator, and whether there was one at all.
local function partitioned(part)
  local at = part:find(Signature.PART_ASSIGNATOR, 1, true)
  if at == nil then
    return nil, nil
  end
  return part:sub(1, at - 1), part:sub(at + 1)
end

local function trimmed(text)
  return (text:gsub("^%s+", ""):gsub("%s+$", ""))
end

--- The pieces of a header, split on the separator and never on anything else.
local function split(text, separator)
  local pieces = {}
  local at = 1
  while true do
    local found = text:find(separator, at, true)
    if found == nil then
      pieces[#pieces + 1] = text:sub(at)
      return pieces
    end
    pieces[#pieces + 1] = text:sub(at, found - 1)
    at = found + #separator
  end
end

--- The `key=value` parts of a header, split on the first assignator of each and trimmed.
local function parts_of(signature)
  local pieces = split(signature, Signature.PART_SEPARATOR)
  if #pieces > Signature.MAX_SIGNATURE_PARTS then
    refuse("the signature carries more than the " .. Signature.MAX_SIGNATURE_PARTS .. " parts accepted")
  end

  -- What is counted is the parts a signature names rather than the parts it carries: a header
  -- naming one of them twice says one thing, not two, and the second value is the one kept.
  local read = {}
  local held = 0
  for index = 1, #pieces do
    local name, value = partitioned(pieces[index])
    if name ~= nil then
      local key = trimmed(name)
      if read[key] == nil then
        held = held + 1
      end
      read[key] = trimmed(value)
    end
  end
  return read, held
end

--- The moment the signature names, which it is not a signature without.
local function timestamp_of(read)
  local written = read[Signature.TIMESTAMP_PART]
  if written == nil then
    refuse("the signature carries no `" .. Signature.TIMESTAMP_PART .. "` part")
  end
  if not written:match("^%-?%d+$") then
    refuse("`" .. written .. "` is not a number of seconds")
  end

  local seconds = math.tointeger(tonumber(written))
  if seconds == nil or math.abs(seconds) > Signature.MAX_TIMESTAMP then
    refuse("the signature's moment is further than " .. string.format("%d", Signature.MAX_TIMESTAMP) ..
      " seconds from the epoch")
  end
  return seconds
end

--- One of the codes a signature offers, decoded whole or not at all.
local function code_of(read, part)
  local written = read[part]
  if written == nil then
    return nil
  end
  if #written == 0 or #written % 2 ~= 0 or written:match("^%x+$") == nil then
    refuse("the `" .. part .. "` code is not hexadecimal")
  end

  return (written:gsub("%x%x", function(pair)
    return string.char(tonumber(pair, 16))
  end))
end

--- What a header name is written with, as RFC 9110 spells a token.
local HEADER_NAME = "^[A-Za-z0-9!#%$%%&'%*%+%-%.%^_`|~]+$"

--- The headers the stronger scheme covers, in the order it covers them.
local function covered_headers_of(read)
  local written = read[Signature.COVERED_HEADERS_PART]
  if written == nil or written == "" then
    return {}
  end

  local names = split(written, Signature.HEADER_NAME_SEPARATOR)
  if #names > Signature.MAX_COVERED_HEADERS then
    refuse("the signature covers more than the " .. Signature.MAX_COVERED_HEADERS .. " headers accepted")
  end

  local covered = {}
  for index = 1, #names do
    if not names[index]:match(HEADER_NAME) then
      refuse("`" .. names[index] .. "` is not a header name")
    end
    covered[index] = names[index]:lower()
  end
  return covered
end

--- Reads a signature header, refusing anything it cannot read whole.
---
--- @param signature string the value of the `X-Hook0-Signature` header
--- @return table
function Signature.parse(signature)
  if type(signature) ~= "string" then
    refuse("the signature is " .. type(signature) .. ", not a header value")
  end
  if #signature > Signature.MAX_SIGNATURE_BYTES then
    refuse("the signature is " .. #signature .. " characters long, above the " ..
      Signature.MAX_SIGNATURE_BYTES .. " accepted")
  end

  local read, held = parts_of(signature)
  if held < 2 then
    refuse("the signature carries neither a timestamp nor a code")
  end

  local body_code = code_of(read, Signature.BODY_SCHEME_PART)
  local headers_code = code_of(read, Signature.HEADERS_SCHEME_PART)
  if body_code == nil and headers_code == nil then
    refuse("the signature carries neither a `" .. Signature.BODY_SCHEME_PART .. "` nor a `" ..
      Signature.HEADERS_SCHEME_PART .. "` code")
  end

  return {
    timestamp = timestamp_of(read),
    covered_headers = covered_headers_of(read),
    body_code = body_code,
    headers_code = headers_code,
  }
end

--- Whether two codes are the same, without saying by how long it took how much of one was right.
---
--- Every byte of both is looked at whatever the first one says, so the time this takes is the length
--- of what it was handed and nothing about its contents.
---
--- @param left string
--- @param right string
--- @return boolean
function Signature.same_code(left, right)
  if #left ~= #right then
    return false
  end

  local differing = 0
  for index = 1, #left do
    differing = differing | (left:byte(index) ~ right:byte(index))
  end
  return differing == 0
end

--- Whether the code this signature carries is the one the secret produces.
---
--- The stronger scheme wins when both are offered.
---
--- @param parsed table what `Signature.parse` answered
--- @param payload string the raw body of the webhook request
--- @param covered_values table the values of the covered headers, in order
--- @param subscription_secret string
--- @return boolean
function Signature.matches(parsed, payload, covered_values, subscription_secret)
  local separator = Signature.MESSAGE_SEPARATOR
  local moment = string.format("%d", parsed.timestamp)

  if parsed.headers_code ~= nil then
    local message = moment .. separator ..
      table.concat(parsed.covered_headers, Signature.HEADER_NAME_SEPARATOR) .. separator ..
      table.concat(covered_values, separator) .. separator ..
      payload
    return Signature.same_code(Sha256.hmac(subscription_secret, message), parsed.headers_code)
  end

  -- A signature carrying neither code is refused while it is being read, so what is left here is
  -- the body-only scheme.
  return Signature.same_code(Sha256.hmac(subscription_secret, moment .. separator .. payload), parsed.body_code)
end

--- The headers of the request, under the names a signature refers to them by.
---
--- A later value wins over an earlier one under the same name, which is what a table built by the
--- caller would have done.
local function delivered_headers(headers)
  local delivered = {}
  local held = 0

  local function record(name, value)
    held = held + 1
    if held > Signature.MAX_DELIVERED_HEADERS then
      refuse("a delivery carries more than the " .. Signature.MAX_DELIVERED_HEADERS .. " headers accepted")
    end
    if type(name) ~= "string" or type(value) ~= "string" then
      refuse("a header is not a header value")
    end
    if utf8.len(name) == nil or utf8.len(value) == nil then
      refuse("a header is not UTF-8")
    end
    delivered[name:lower()] = value
  end

  if headers == nil then
    return delivered
  end
  if type(headers) ~= "table" then
    refuse("the headers are " .. type(headers) .. ", not a table")
  end

  -- Both shapes a caller holds headers in: a list of pairs, in the order they arrived, and a table
  -- keyed by name. A list is read first, since a table with an entry at `1` is one.
  if headers[1] ~= nil then
    for index = 1, #headers do
      record(headers[index][1], headers[index][2])
    end
    return delivered
  end

  for name, value in pairs(headers) do
    record(name, value)
  end
  return delivered
end

--- Verifies a webhook against a moment the caller names.
---
--- The clock window is bilateral. A moment too far in the future is refused exactly like one too far
--- in the past, so the window a given delivery is accepted in stays the width the caller asked for,
--- whichever way a clock drifted.
---
--- @param signature string the value of the `X-Hook0-Signature` header
--- @param payload string the raw body of the webhook request
--- @param headers table the headers of the webhook request, as pairs or keyed by name
--- @param subscription_secret string the signing secret of the subscription it was delivered for
--- @param tolerance number how far, in seconds and in either direction, the moment the signature
---   names may sit from `current_time`. Five minutes is a reasonable trade-off between tolerating
---   clock drift and bounding how long a captured delivery can be replayed.
--- @param current_time number what to hold the signature's moment against, in seconds since the
---   epoch
--- @return nil
function Signature.verify_with_current_time(
  signature, payload, headers, subscription_secret, tolerance, current_time
)
  local parsed = Signature.parse(signature)

  local delivered = delivered_headers(headers)
  local covered_values = {}
  for index = 1, #parsed.covered_headers do
    local name = parsed.covered_headers[index]
    if delivered[name] == nil then
      refuse("the `" .. name .. "` header the signature covers was not delivered")
    end
    covered_values[index] = delivered[name]
  end

  if not Signature.matches(parsed, tostring(payload or ""), covered_values, tostring(subscription_secret or "")) then
    refuse("the signature does not match what the subscription secret produces")
  end

  local drift = current_time - parsed.timestamp
  if math.abs(drift) > tolerance then
    refuse(string.format("the signature was made %.0f seconds from now, outside the %s accepted",
      drift, tostring(tolerance)))
  end

  return nil
end

--- Verifies a webhook against the current moment.
---
--- See `Signature.verify_with_current_time` for what each argument is.
---
--- @param signature string
--- @param payload string
--- @param headers table
--- @param subscription_secret string
--- @param tolerance number
--- @return nil
function Signature.verify(signature, payload, headers, subscription_secret, tolerance)
  return Signature.verify_with_current_time(
    signature, payload, headers, subscription_secret, tolerance, os.time()
  )
end

return Signature
