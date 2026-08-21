--- SHA-256 and the keyed hash built on it, written out because Lua carries neither.
---
--- Lua's standard library has no cryptography at all, so this is the one thing a client that
--- verifies webhook signatures cannot borrow. It is written against FIPS 180-4 and against RFC 2104,
--- and it is held to codes computed outside it: the vectors of the shared conformance corpus were
--- produced with a general-purpose HMAC tool, so a suite that passes them is not one that agreed
--- with itself.
---
--- Lua 5.3 gave the language 64-bit integers and the bitwise operators, which is what makes this
--- readable rather than a table of arithmetic tricks. Every intermediate is masked back to 32 bits
--- after every step, since the words of SHA-256 are 32 bits wide and Lua's are not.

local Sha256 = {}

--- How many bytes one block of the compression function carries.
local BLOCK_BYTES = 64

--- How many bytes the digest is.
Sha256.DIGEST_BYTES = 32

--- Longest message this module hashes, in bytes. A signature is computed over a body the caller was
--- handed by whoever reached its endpoint, so what is hashed is bounded like anything else that
--- arrives from there.
Sha256.MAX_MESSAGE_BYTES = 64 * 1024 * 1024

--- What the standard mixes in at each of the sixty-four rounds: the fractional parts of the cube
--- roots of the first sixty-four primes.
local ROUND_CONSTANTS = {
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
}

--- Where the state starts: the fractional parts of the square roots of the first eight primes.
local INITIAL_STATE = {
  0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
}

local MASK = 0xFFFFFFFF

local function rotate(word, places)
  return ((word >> places) | (word << (32 - places))) & MASK
end

--- The message, with the padding the standard puts on the end of it: a one bit, as many zero bits
--- as it takes, and the length in bits as eight bytes.
local function padded(message)
  local bits = #message * 8
  local zeros = (55 - #message) % BLOCK_BYTES
  return message .. "\128" .. string.rep("\0", zeros) .. string.pack(">I8", bits)
end

--- The digest of a message, as the thirty-two bytes it is.
--- @param message string
--- @return string
function Sha256.digest(message)
  if type(message) ~= "string" then
    error("a message is " .. type(message) .. ", not text", 2)
  end
  if #message > Sha256.MAX_MESSAGE_BYTES then
    error("a message of " .. #message .. " bytes is above the " .. Sha256.MAX_MESSAGE_BYTES .. " accepted", 2)
  end

  local state = { table.unpack(INITIAL_STATE) }
  local written = padded(message)
  local schedule = {}

  for start = 1, #written, BLOCK_BYTES do
    for index = 1, 16 do
      schedule[index] = string.unpack(">I4", written, start + (index - 1) * 4)
    end
    for index = 17, 64 do
      local left, right = schedule[index - 15], schedule[index - 2]
      local mixed_left = rotate(left, 7) ~ rotate(left, 18) ~ (left >> 3)
      local mixed_right = rotate(right, 17) ~ rotate(right, 19) ~ (right >> 10)
      schedule[index] = (schedule[index - 16] + mixed_left + schedule[index - 7] + mixed_right) & MASK
    end

    local a, b, c, d = state[1], state[2], state[3], state[4]
    local e, f, g, h = state[5], state[6], state[7], state[8]

    for index = 1, 64 do
      local sum_e = rotate(e, 6) ~ rotate(e, 11) ~ rotate(e, 25)
      local choice = (e & f) ~ ((~e & MASK) & g)
      local first = (h + sum_e + choice + ROUND_CONSTANTS[index] + schedule[index]) & MASK
      local sum_a = rotate(a, 2) ~ rotate(a, 13) ~ rotate(a, 22)
      local majority = (a & b) ~ (a & c) ~ (b & c)
      local second = (sum_a + majority) & MASK

      h, g, f = g, f, e
      e = (d + first) & MASK
      d, c, b = c, b, a
      a = (first + second) & MASK
    end

    local mixed = { a, b, c, d, e, f, g, h }
    for index = 1, 8 do
      state[index] = (state[index] + mixed[index]) & MASK
    end
  end

  return string.pack(">I4I4I4I4I4I4I4I4", table.unpack(state))
end

--- The digest of a message, written out as the sixty-four hexadecimal characters it is.
--- @param message string
--- @return string
function Sha256.hexdigest(message)
  return (Sha256.digest(message):gsub(".", function(byte)
    return string.format("%02x", byte:byte())
  end))
end

--- The keyed hash of a message, as RFC 2104 defines one.
---
--- A key longer than a block is replaced by its own digest, and a shorter one is padded out with
--- zeros — both of which the standard says, and neither of which a caller should have to do.
---
--- @param key string the secret the code is computed under
--- @param message string what the code covers
--- @return string the thirty-two bytes of the code
function Sha256.hmac(key, message)
  if type(key) ~= "string" then
    error("a key is " .. type(key) .. ", not text", 2)
  end

  local block = key
  if #block > BLOCK_BYTES then
    block = Sha256.digest(block)
  end
  block = block .. string.rep("\0", BLOCK_BYTES - #block)

  local inner, outer = {}, {}
  for index = 1, BLOCK_BYTES do
    local byte = block:byte(index)
    inner[index] = string.char(byte ~ 0x36)
    outer[index] = string.char(byte ~ 0x5c)
  end

  return Sha256.digest(table.concat(outer) .. Sha256.digest(table.concat(inner) .. message))
end

--- The keyed hash, written out as hexadecimal.
--- @param key string
--- @param message string
--- @return string
function Sha256.hexhmac(key, message)
  return (Sha256.hmac(key, message):gsub(".", function(byte)
    return string.format("%02x", byte:byte())
  end))
end

return Sha256
