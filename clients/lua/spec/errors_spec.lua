--- How a caller tells one failure of this client from another.
---
--- Lua's `error` takes any value, so a caller that reached for this library is holding whatever a
--- `pcall` handed back — a failure this client raised, or something a third-party module raised, or
--- a bare string. What is here is that the three are told apart without the caller having to know
--- which it is holding, and that a chain of kinds is walked rather than trusted.

local Helper = require("spec.helper")

local Hook0 = Helper.Hook0

describe("a kind of failure", function()
  it("is named by the name it was declared under", function()
    local declared = Hook0.kind("SomethingWentWrong", Hook0.ClientError)

    assert.are.equal("SomethingWentWrong", tostring(declared))
  end)

  it("is refused when it is named by anything but a name", function()
    -- A kind with no name is one no message can print and no caller can recognise.
    for _, unnamed in ipairs({ "", 42 }) do
      local raised = Helper.refused(function() return Hook0.kind(unnamed) end)
      assert.is_truthy(tostring(raised):find("a kind is named by a string", 1, true), tostring(raised))
    end
  end)
end)

describe("whether a raised value is of a kind", function()
  it("is false for anything this client did not raise", function()
    -- A caller rescues whatever `pcall` handed it, which may be a string a third-party module
    -- raised: asking what kind that is answers no rather than failing on it.
    for _, held in ipairs({ "a string another module raised", 42, {} }) do
      assert.is_false(Hook0.is(held, Hook0.ClientError))
    end
  end)

  it("is false when what it is asked about is not a kind", function()
    local raised = Helper.refused(function() return Hook0.EventType.parse("auth.user") end)

    assert.is_true(Hook0.is(raised, Hook0.ClientError))
    assert.is_false(Hook0.is(raised, "ClientError"))
  end)

  it("is false for a kind the chain never reaches", function()
    local unrelated = Hook0.kind("SomethingElseEntirely")
    local raised = Helper.refused(function() return Hook0.EventType.parse("auth.user") end)

    assert.is_false(Hook0.is(raised, unrelated))
  end)
end)

describe("what to say about a raised value", function()
  it("is what the failure carries, when this client raised it", function()
    local raised = Helper.refused(function() return Hook0.EventType.parse("auth.user") end)

    assert.are.equal(tostring(raised), Hook0.message(raised))
    assert.is_truthy(Hook0.message(raised):find("auth.user", 1, true))
  end)

  it("is what it reads as, when something else raised it", function()
    assert.are.equal("a string another module raised", Hook0.message("a string another module raised"))
  end)

  it("is cut to what a failure keeps, however long what it was built from was", function()
    -- A message is built out of bodies a server this client does not control answered, so the
    -- message itself is bounded rather than the pieces it was built from.
    local written = string.rep("e", 8192)
    local held = Hook0.message(written)

    assert.is_true(#held < #written, "a message longer than a failure keeps was carried whole")
    assert.is_truthy(held:find("…", 1, true), "a message that was cut does not say so")
  end)
end)
