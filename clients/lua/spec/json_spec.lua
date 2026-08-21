--- The one JSON codec this rock carries, and everything it refuses.
---
--- Lua's standard library reads no JSON at all, so this is written out under `src` rather than
--- installed — which makes what it accepts this rock's problem rather than a dependency's. Both
--- directions are bounded: what a caller hands it to write, and what a server this client does not
--- control hands it to read. A document past one of those bounds is refused rather than parsed, and
--- that refusal is what these cases are about.

local Helper = require("spec.helper")

local Hook0 = Helper.Hook0
local Json = Helper.Json

--- Why writing or reading refused what it was handed.
---
--- The codec raises what it raises rather than a string, so what a case reads is the reason it
--- carries — which is the same thing a caller reaches for.
local function refusal_of(run)
  local raised = Helper.refused(run)
  return Json.reason(raised) or Hook0.message(raised)
end

--- A document nesting that many levels deep, written out.
local function nested(depth)
  return string.rep("[", depth) .. string.rep("]", depth)
end

describe("writing a document", function()
  it("refuses an object carrying a key that is not a name", function()
    -- A key that is not a string has no spelling on the wire, and guessing one would put a member
    -- the caller never wrote into what goes out.
    assert.is_truthy(refusal_of(function() return Json.encode(Json.object({ [true] = "carried" })) end)
      :find("not a string", 1, true))
  end)

  it("refuses an array carrying a key that is not an index", function()
    assert.is_truthy(refusal_of(function() return Json.encode(Json.array({ named = "carried" })) end)
      :find("not an index", 1, true))
  end)

  it("refuses a number no document can carry", function()
    for _, unusable in ipairs({ 0 / 0, math.huge, -math.huge }) do
      assert.is_truthy(refusal_of(function() return Json.encode(Json.object({ held = unusable })) end)
        :find("cannot carry", 1, true))
    end
  end)

  it("refuses a value that is not something a document carries at all", function()
    assert.is_truthy(refusal_of(function() return Json.encode(Json.object({ held = print })) end)
      :find("is not something a document carries", 1, true))
  end)

  it("refuses a document nesting deeper than it writes", function()
    local held = Json.array({})
    for _ = 1, Json.MAX_DEPTH + 1 do
      held = Json.array({ held })
    end

    assert.is_truthy(refusal_of(function() return Json.encode(held) end):find("nests deeper", 1, true))
  end)

  it("writes nothing at all as the null a document spells", function()
    assert.are.equal("null", Json.encode(nil))
  end)

  it("writes what it read, and reads what it wrote, for every shape it carries", function()
    local written = Json.encode(Json.object({
      text = "a value",
      number = 12,
      truth = true,
      nothing = Json.null,
      list = Json.array({ 1, 2 }),
      nested = Json.object({ held = "deeper" }),
    }))
    local read = Json.decode(written)

    assert.are.equal("a value", read.text)
    assert.are.equal(12, read.number)
    assert.is_true(read.truth)
    assert.are.equal(Json.null, read.nothing)
    assert.are.same({ 1, 2 }, read.list)
    assert.are.equal("deeper", read.nested.held)
  end)
end)

describe("reading a document", function()
  it("refuses something that is not text at all", function()
    assert.is_truthy(refusal_of(function() return Json.decode(42) end):find("not text", 1, true))
  end)

  it("refuses one longer than it reads", function()
    -- The bytes come off a socket a server this client does not control is on the other end of, so
    -- how much of them is parsed at all is bounded here rather than there.
    local oversized = "\"" .. string.rep("e", Json.MAX_TEXT_BYTES) .. "\""

    assert.is_truthy(refusal_of(function() return Json.decode(oversized) end):find("bytes long", 1, true))
  end)

  it("refuses one nesting deeper than it reads", function()
    -- A document that is nothing but brackets would otherwise be walked for as deep as it goes.
    assert.has_no.errors(function() return Json.decode(nested(Json.MAX_DEPTH)) end)
    assert.is_truthy(refusal_of(function() return Json.decode(nested(Json.MAX_DEPTH + 1)) end)
      :find("nests deeper", 1, true))
  end)

  it("refuses a string carrying a character no string may carry unescaped", function()
    assert.is_truthy(refusal_of(function() return Json.decode('{"held":"a\tvalue"}') end)
      :find("control character", 1, true))
  end)

  it("refuses an object whose members are not separated the way one is", function()
    assert.is_truthy(refusal_of(function() return Json.decode('{"one":1 "other":2}') end)
      :find("expected `,` or `}`", 1, true))
  end)

  it("refuses an escape that is not four hexadecimal digits", function()
    assert.is_truthy(refusal_of(function() return Json.decode('{"held":"\\uZZZZ"}') end)
      :find("not four hexadecimal digits", 1, true))
  end)

  it("reads a lone leading surrogate as the code point it spells rather than joining what follows", function()
    -- A pair is two escapes that both name their half. A leading half followed by anything else —
    -- nothing, another escape that is not four digits, or one outside the trailing range — is the
    -- code point it spells and no more, rather than an offset into whatever came next.
    for _, written in ipairs({ '"\\ud83d"', '"\\ud83dtail"', '"\\ud83d\\u0041"' }) do
      local read = Json.decode('{"held":' .. written .. '}')
      assert.are.equal(utf8.char(0xD83D), read.held:sub(1, #utf8.char(0xD83D)),
        "a lone leading surrogate was joined to what followed it")
    end
  end)

  it("refuses the escape that follows a lone leading surrogate rather than folding it in", function()
    -- The leading half is read as the code point it spells and what follows is read as an escape of
    -- its own, so a malformed one there is refused rather than swallowed by the pair that was not.
    assert.is_truthy(refusal_of(function() return Json.decode('{"held":"\\ud83d\\uZZZZ"}') end)
      :find("not four hexadecimal digits", 1, true))
  end)

  it("reads an escape as the character it names, surrogate pairs included", function()
    -- A code point past the basic plane travels as two escapes, and reading them one at a time
    -- would yield two characters that are neither of them the one that was written.
    local read = Json.decode('{"held":"\\u00e9\\ud83d\\ude00\\n"}')

    assert.are.equal("é😀\n", read.held)
  end)
end)

describe("what a failure of the codec says", function()
  it("is nothing at all for a failure the codec did not raise", function()
    -- The codec's own refusals carry what they were, and anything else a `pcall` hands back is not
    -- one: reading it as one would report another module's failure as a malformed document.
    assert.is_nil(Json.reason("a string another module raised"))
    assert.is_truthy(Json.reason(Helper.refused(function() return Json.decode("{") end)))
  end)
end)
