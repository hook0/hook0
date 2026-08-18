--- Where a request lands, which is settled before a socket is opened.
---
--- What a base URL and a path add up to is the one thing about a request this client settles on its
--- own: everything else is either what the caller asked for or what the API answered. A base that
--- names nowhere is therefore refused rather than turned into a message accusing the network, and
--- the refusal is the one of the three causes the shared corpus classifies that says so.

local Helper = require("spec.helper")

local Hook0 = Helper.Hook0
local Json = Helper.Json

--- What a request against that base URL raised.
local function refused_by(base_url, path)
  return Helper.refused(function()
    return Hook0.Transport.new(base_url, "token-xyz"):request("GET", path or "/event")
  end)
end

describe("a base URL nothing can be sent to", function()
  it("is refused rather than sent to, when it is not a URL at all", function()
    local raised = refused_by(42)

    assert.is_true(Hook0.is(raised, Hook0.TransportError))
    assert.are.equal("unusable_api_url", raised.cause)
    assert.is_false(raised.retryable, "building the same unusable request again cannot end differently")
  end)

  it("is refused when it names no scheme", function()
    local raised = refused_by("nowhere.invalid/api/v1")

    assert.are.equal("unusable_api_url", raised.cause)
    assert.is_truthy(Hook0.message(raised):find("names no scheme", 1, true), Hook0.message(raised))
  end)

  it("is refused when it names a scheme this transport does not speak", function()
    local raised = refused_by("gopher://nowhere.invalid")

    assert.are.equal("unusable_api_url", raised.cause)
    assert.is_truthy(Hook0.message(raised):find("gopher", 1, true), Hook0.message(raised))
  end)

  it("is refused when it names no host", function()
    local raised = refused_by("http://")

    assert.are.equal("unusable_api_url", raised.cause)
    assert.is_truthy(Hook0.message(raised):find("names no host", 1, true), Hook0.message(raised))
  end)
end)

describe("a base URL carrying a credential of its own", function()
  it("is reached at the host beyond it rather than at the whole of it", function()
    -- Text before an `@` is userinfo rather than a host; sending to it would open a socket to
    -- something that is not a machine at all.
    local api = Helper.FakeApi.new({ { status = 200, body = Json.array({}) } })
    local base = api:base_url():gsub("^http://", "http://someone:secret@")

    local status = Hook0.Transport.new(base, "token-xyz"):request("GET", "/event_types")
    local received = api:stop()

    assert.are.equal(200, status)
    assert.are.equal(1, #received)
    assert.are.equal("/event_types", received[1].target)
  end)
end)
