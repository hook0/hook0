-- What the emitted source and the hand-written source are both held to.
--
-- The generator writes source that already satisfies this file: nothing reformats what it wrote, so
-- a warning reported here is a defect in it or a hand edit beside it.

std = "lua54"

-- The same margin the generator folds its comments and its argument lists to.
max_line_length = 120

-- Nothing sets a global. Both halves of this rock answer with a table and reach everything else
-- through `require`, so a global appearing anywhere is a `local` somebody forgot.
allow_defined_top = false

files["spec"] = {
  -- What busted puts in scope for a spec file, which nothing else in this rock may read.
  read_globals = { "describe", "it", "before_each", "after_each", "setup", "teardown", "pending" },
}
