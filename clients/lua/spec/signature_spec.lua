--- Verifying a webhook, beyond the vectors the shared corpus pins.
---
--- Every accepted and refused delivery the corpus declares is run in `conformance_spec.lua`. What is
--- here is what a vector cannot say: that the pieces this is built out of are the right ones, that
--- the window looks both ways, and that a header nothing can be read out of is refused rather than
--- guessed at.

local Helper = require("spec.helper")

local Hook0 = Helper.Hook0
local Sha256 = Hook0.Sha256
local Signature = Hook0.Signature

local SECRET = "a-subscription-secret"
local PAYLOAD = '{"event":"user.created"}'
local MOMENT = 1800000000
local TOLERANCE = 300

local HEADERS = {
  { "x-event-id", "evt-1" },
  { "x-delivery-id", "dlv-1" },
  { "content-type", "application/json" },
}

--- A signature over the body alone, at that moment.
local function body_scheme(moment)
  local code = Sha256.hexhmac(SECRET, moment .. "." .. PAYLOAD)
  return "t=" .. moment .. ",v0=" .. code
end

local function verified(signature, headers, current_time)
  return Signature.verify_with_current_time(
    signature, PAYLOAD, headers or HEADERS, SECRET, TOLERANCE + 0.0, current_time or MOMENT)
end

describe("SHA-256", function()
  it("answers the digests the standard publishes", function()
    -- Held against values computed outside this repository: a suite that hashed with the module it
    -- is testing and compared against the same module would pass whatever the two agreed on.
    assert.are.equal("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      Sha256.hexdigest(""))
    assert.are.equal("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
      Sha256.hexdigest("abc"))
    assert.are.equal("248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
      Sha256.hexdigest("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"))
    assert.are.equal("cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
      Sha256.hexdigest(string.rep("a", 1000000)))
  end)

  it("answers the keyed hashes RFC 4231 publishes", function()
    assert.are.equal("b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7",
      Sha256.hexhmac(string.rep("\11", 20), "Hi There"))
    assert.are.equal("5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843",
      Sha256.hexhmac("Jefe", "what do ya want for nothing?"))
    -- A key longer than a block, which the standard says is replaced by its own digest.
    assert.are.equal("60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54",
      Sha256.hexhmac(string.rep("\170", 131), "Test Using Larger Than Block-Size Key - Hash Key First"))
  end)
end)

describe("a signature", function()
  it("verifies at the edge of the window, on either side of it", function()
    -- The window is the width a delivery is accepted within, so its own edges are inside it.
    assert.has_no.errors(function() verified(body_scheme(MOMENT - TOLERANCE)) end)
    assert.has_no.errors(function() verified(body_scheme(MOMENT + TOLERANCE)) end)
  end)

  it("is refused as far ahead of the window as behind it", function()
    -- A window that only looked backwards is one a sender widens by dating its own delivery in the
    -- future, which is the same replay the window exists to bound.
    local behind = Helper.refused(function() verified(body_scheme(MOMENT - TOLERANCE - 1)) end)
    local ahead = Helper.refused(function() verified(body_scheme(MOMENT + TOLERANCE + 1)) end)

    assert.is_truthy(Hook0.message(behind):find("outside the", 1, true), Hook0.message(behind))
    assert.is_truthy(Hook0.message(ahead):find("outside the", 1, true), Hook0.message(ahead))
  end)

  it("is read on the first assignator of each part, and never on the rest", function()
    -- A value may hold further assignators, and splitting on all of them would drop everything past
    -- the second.
    local parsed = Signature.parse("t=" .. MOMENT .. ",note=a=b=c,v0=" .. string.rep("ab", 32))
    assert.are.equal(MOMENT, parsed.timestamp)
  end)

  it("refuses a code that is not whole hexadecimal, rather than reading it as far as it goes", function()
    for _, code in ipairs({ "abc", "zz", "", "abcg" }) do
      local raised = Helper.refused(function()
        return Signature.parse("t=" .. MOMENT .. ",v0=" .. code)
      end)
      assert.is_truthy(Hook0.message(raised):find("hexadecimal", 1, true) or
        Hook0.message(raised):find("neither", 1, true),
        "`" .. code .. "` was refused as `" .. Hook0.message(raised) .. "`")
    end
  end)

  it("refuses a header carrying no moment, no code, or nothing at all", function()
    for _, header in ipairs({ "", "t=" .. MOMENT, "v0=" .. string.rep("ab", 32), "nonsense", "t=later,v0=ab" }) do
      assert.is_true(Hook0.is(Helper.refused(function() return Signature.parse(header) end), Hook0.ClientError),
        "`" .. header .. "` was read as a signature")
    end
  end)

  it("settles a covered header that was not delivered before any code is computed", function()
    -- Signing over an absent value would let a sender drop a header and keep the signature valid, so
    -- the refusal comes first — and it names the header rather than reporting a mismatch.
    local raised = Helper.refused(function()
      return verified("t=" .. MOMENT .. ",h=x-event-id x-missing,v1=" .. string.rep("ab", 32))
    end)

    assert.is_truthy(Hook0.message(raised):find("was not delivered", 1, true), Hook0.message(raised))
    assert.is_truthy(Hook0.message(raised):find("x-missing", 1, true),
      "the refusal does not name the header the signature covered: " .. Hook0.message(raised))
  end)

  it("reads the headers of a delivery held either way round", function()
    -- A caller holds headers as the pairs they arrived in or as a table keyed by name; both are the
    -- same delivery.
    local keyed = {}
    for _, pair in ipairs(HEADERS) do
      keyed[pair[1]] = pair[2]
    end

    assert.has_no.errors(function() verified(body_scheme(MOMENT), HEADERS) end)
    assert.has_no.errors(function() verified(body_scheme(MOMENT), keyed) end)
  end)

  it("compares two codes without saying how much of one was right", function()
    assert.is_true(Signature.same_code("abcd", "abcd"))
    assert.is_false(Signature.same_code("abcd", "abce"))
    assert.is_false(Signature.same_code("abcd", "abcde"))
    assert.is_false(Signature.same_code("", "a"))
  end)

  it("refuses a header longer than it reads, before any of it is split", function()
    local raised = Helper.refused(function()
      return Signature.parse(string.rep("t=1,", Signature.MAX_SIGNATURE_BYTES))
    end)
    assert.is_truthy(Hook0.message(raised):find(tostring(Signature.MAX_SIGNATURE_BYTES), 1, true),
      "the refusal does not name the ceiling it crossed: " .. Hook0.message(raised))
  end)
end)
