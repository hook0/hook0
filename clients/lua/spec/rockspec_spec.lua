--- What installing this rock puts where, and what it is allowed to drag in.
---
--- The rockspec maps a module name onto a source path, and that map is what this suite loads the
--- library through — so a case that passes here is a case that ran against what an install would
--- produce. What is left is to hold the map against the tree: a file the generator wrote and nobody
--- added to the rockspec would be missing from every install while every case still passed, since
--- nothing here would ever have asked for it.
---
--- Both directions are walked, and neither is a list written down: the tree is read off disk and the
--- map is read off the rockspec.

local Helper = require("spec.helper")

local Hook0 = Helper.Hook0

--- Deepest under `src` a source file may sit before this walk gives up.
local MAX_DEPTH = 4

--- Most files walked under `src`.
local MAX_FILES = 512

--- Every `.lua` file under a directory, as a path relative to the client.
local function sources_under(directory, held, depth)
  local lfs = require("lfs")
  assert(depth <= MAX_DEPTH, directory .. " sits deeper than the " .. MAX_DEPTH .. " directories walked")

  for entry in lfs.dir(Helper.CLIENT_ROOT .. "/" .. directory) do
    if entry ~= "." and entry ~= ".." then
      local path = directory .. "/" .. entry
      local kind = lfs.attributes(Helper.CLIENT_ROOT .. "/" .. path, "mode")
      if kind == "directory" then
        sources_under(path, held, depth + 1)
      elseif kind == "file" and entry:match("%.lua$") then
        assert(#held < MAX_FILES, "more than the " .. MAX_FILES .. " files walked sit under src")
        held[#held + 1] = path
      end
    end
  end
  return held
end

describe("the rockspec", function()
  local declared, filename = Helper.rockspec()

  it("is named after the version it declares, and the version the library names", function()
    assert.are.equal(Hook0.VERSION .. "-1", declared.version,
      "the rockspec declares a version the library does not name")
    assert.are.equal(declared.package .. "-" .. declared.version .. ".rockspec", filename,
      "the rockspec is not named after what it declares")
  end)

  it("fetches the release it is itself part of", function()
    -- A rock carries the source it is built from rather than containing it, so the tag here is what
    -- an install actually downloads. A bump that writes the version and leaves the tag behind
    -- produces a rock that installs the release before the one it announces, and nothing downstream
    -- of the install would show the difference. The prefix is the per-package tag convention that
    -- adr/0004-monorepo-tag-convention.md fixes for this train.
    assert.are.equal("sdk-v" .. Hook0.VERSION, declared.source.tag,
      "the rockspec fetches a release other than the one it declares")
  end)

  it("is published under the name this SDK is published under everywhere else", function()
    assert.are.equal("hook0-client", declared.package)
  end)

  it("maps every source of this client, and nothing that is not one", function()
    local walked = {}
    for _, path in ipairs(sources_under("src", {}, 1)) do
      walked[path] = true
    end

    local mapped = {}
    for module, path in pairs(declared.build.modules) do
      mapped[path] = module
      assert.is_true(walked[path] == true,
        "the rockspec maps `" .. module .. "` onto " .. path .. ", which this client does not carry")
      assert.is_truthy(module == "hook0" or module:match("^hook0%."),
        "the rockspec installs `" .. module .. "`, which is not under this rock's own name")
    end

    for path in pairs(walked) do
      assert.is_truthy(mapped[path],
        path .. " is a source of this client that the rockspec does not install, so nothing that " ..
        "requires it would find it")
    end
  end)

  it("declares only the dependencies Lua's standard library cannot answer for", function()
    -- Sockets are the one thing this SDK cannot write out: the standard library has no way to open
    -- one at all. Everything else — JSON, SHA-256, the keyed hash a signature is verified with — is
    -- pure Lua under `src`, which is why nothing else may appear here.
    local allowed = { lua = true, luasocket = true, luasec = true }

    for _, dependency in ipairs(declared.dependencies) do
      local name = dependency:match("^%s*([%w%-_]+)")
      assert.is_truthy(allowed[name],
        "this rock has grown a runtime dependency on `" .. tostring(name) ..
        "`; it is meant to reach for the standard library and a socket, and nothing else")
    end
  end)

  it("ships both halves of what this rock is", function()
    local shipped = {}
    for module in pairs(declared.build.modules) do
      shipped[module] = true
    end

    assert.is_true(shipped["hook0"], "the rockspec installs nothing a caller can require")
    assert.is_true(shipped["hook0.signature"], "the rockspec installs no signature verification")
    assert.is_true(shipped["hook0.generated.all"], "the rockspec installs none of what the generator wrote")
  end)
end)
