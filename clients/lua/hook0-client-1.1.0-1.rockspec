-- What this rock is, and what installing it puts where.
--
-- The name is the one this SDK is published under everywhere else: `hook0-client` on PyPI, on
-- RubyGems, on crates.io, on Packagist and on npm, and now on LuaRocks. A rock published as `hook0`
-- would be the odd one out of a set that is already six deep.
--
-- `modules` maps a module name onto a source path, which is what makes the layout of this repository
-- and the layout of the installed rock two separate things: the generator owns `src/generated` and
-- nothing else, the hand-written half sits beside it in `src`, and a caller reaches both as
-- `hook0.*`. The map is not a list somebody keeps up to date by hand — `spec/rockspec_spec.lua`
-- walks `src` and fails when a file here is missing from it, or when an entry here names a file that
-- is not there.

rockspec_format = "3.0"
package = "hook0-client"
version = "1.1.0-1"

source = {
  url = "git+https://gitlab.com/hook0/hook0.git",
  tag = "sdk-v1.1.0",
  dir = "hook0/clients/lua",
}

description = {
  summary = "Lua SDK for Hook0, open-source Webhooks as a service for SaaS",
  detailed = [[
Send events to Hook0, upsert the event types your application uses, verify the signature of an
incoming webhook, and call every operation the API declares through generated, documented tables.
Sending is idempotent and retried under bounds the caller sets.
]],
  homepage = "https://www.hook0.com/",
  issues_url = "https://gitlab.com/hook0/hook0/-/issues",
  labels = { "webhooks", "http", "api", "saas" },
  license = "MIT",
}

-- The two things Lua's standard library does not carry that this SDK cannot do without.
--
-- Everything else is written out beside this file: the JSON reader and writer, SHA-256, and the
-- keyed hash a signature is verified with are all pure Lua under `src`, and they are held to codes
-- computed outside this repository by the shared conformance corpus. Sockets are the one thing that
-- cannot be written out — the standard library has no way to open one at all, and the only escape is
-- `io.popen`, which is a shell rather than a socket. `luasec` is what turns that socket into an
-- `https` one, which is the only scheme a hosted Hook0 answers on.
dependencies = {
  "lua >= 5.3, < 5.5",
  "luasocket >= 3.0",
  "luasec >= 1.0",
}

build = {
  type = "builtin",
  modules = {
    ["hook0"] = "src/hook0.lua",
    ["hook0.client"] = "src/client.lua",
    ["hook0.errors"] = "src/errors.lua",
    ["hook0.json"] = "src/json.lua",
    ["hook0.runtime"] = "src/runtime.lua",
    ["hook0.sha256"] = "src/sha256.lua",
    ["hook0.signature"] = "src/signature.lua",
    ["hook0.transport"] = "src/transport.lua",
    ["hook0.generated.all"] = "src/generated/all.lua",
    ["hook0.generated.api"] = "src/generated/api.lua",
    ["hook0.generated.errors"] = "src/generated/errors.lua",
    ["hook0.generated.models"] = "src/generated/models.lua",
  },
  copy_directories = {},
}

test_dependencies = {
  "busted >= 2.0",
  "luacheck >= 1.1",
}

test = {
  type = "busted",
}
