#!/usr/bin/env python3
"""Check that every directory under `skills/` is a loadable Agent Skill.

A skill is loaded by its `SKILL.md`: a YAML front matter block whose `name` is
the directory name (lowercase letters, digits and hyphens, at most 64
characters) and whose `description` says what the skill does and when to use
it (at most 1024 characters). A skill that breaks either is silently skipped by
the agents that read it, so this refuses it here instead.

Only the two fields are read, without a YAML library, because the front matter
of these skills is plain `key: value` lines and folded `>-` blocks.
"""
import pathlib
import re
import sys

NAME = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)*$")


def front_matter(text):
    if not text.startswith("---\n"):
        raise ValueError("does not open with a `---` front matter block")
    end = text.find("\n---\n", 4)
    if end < 0:
        raise ValueError("front matter block is never closed")
    fields, current = {}, None
    for line in text[4:end].splitlines():
        match = re.match(r"^([a-z_-]+):\s*(.*)$", line)
        if match:
            current = match.group(1)
            value = match.group(2)
            fields[current] = "" if value in (">-", ">", "|", "|-") else value
        elif current and line.startswith("  "):
            fields[current] = (fields[current] + " " + line.strip()).strip()
    return fields


def main(root):
    skills = sorted(p for p in pathlib.Path(root).iterdir() if p.is_dir() and not p.name.startswith("."))
    if not skills:
        print(f"no skill directory under {root}")
        return 1
    failures = 0
    for skill in skills:
        problems = []
        manifest = skill / "SKILL.md"
        if not manifest.is_file():
            problems.append("no SKILL.md")
        else:
            try:
                fields = front_matter(manifest.read_text())
            except ValueError as error:
                problems.append(str(error))
                fields = {}
            name = fields.get("name", "")
            description = fields.get("description", "")
            if name != skill.name:
                problems.append(f"`name` is {name!r}, not the directory name {skill.name!r}")
            if not NAME.match(name) or len(name) > 64:
                problems.append("`name` must be lowercase letters, digits and hyphens, 64 characters at most")
            if not description:
                problems.append("`description` is empty")
            elif len(description) > 1024:
                problems.append(f"`description` is {len(description)} characters, above 1024")
        failures += bool(problems)
        print(f"{'FAIL' if problems else 'OK  '} {skill.name}" + "".join(f"\n     - {p}" for p in problems))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1] if len(sys.argv) > 1 else "skills"))
