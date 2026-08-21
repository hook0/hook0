-- What every assembled Lua example of the SDK reference is held to.
--
-- Nothing here sets a global on purpose: every harness region introduces what a snippet assumes
-- through a `local`, the way a reader's own file would, so a name luacheck cannot resolve is a typo
-- this project failed to catch, not a global the page's author meant to reach. `clients/lua` itself
-- is held to the same `std` and the same margin; see its own `.luacheckrc`.

std = "lua54"
max_line_length = 120
allow_defined_top = false

-- W542, empty if branch: the page reads a raised failure by matching its kind in an if/elseif
-- chain and explaining, in a comment, what each branch means -- the same style the Go reference
-- page shows its own error kinds in (an empty `case` per error, one comment each). The branch is
-- empty because the page is naming a kind, not writing a handler; a rule built for production code
-- would ask the example to invent a body it does not need.
ignore = { "542" }
