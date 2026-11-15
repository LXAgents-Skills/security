---
name: tool-authoring
description: How a tool comes to exist - the surface is derived from content/, so adding a markdown file to the set is the whole procedure.
---

# Tool Authoring

**Adding a markdown file to `content/` is adding a tool.** There is no tool file to
write, no import to add, and no entry to put in an array.

`build.rs` embeds every `.md` file under `content/`, and `src/instructions/mod.rs` makes one
tool per embedded file, once, at startup. A contributor who hand-writes a tool beside the
generator would put two tools for one file on the surface, or one the derivation cannot see.
A change to `content/` reaches the binary on the next build.

## The derivation

A tool's ID is `lxagents_skills_security_` followed by a name from its own filename:

1. take the basename, dropping the folder;
2. drop `.md`;
3. lowercase;
4. `-` → `_`.

| File | Tool |
|---|---|
| `SKILL.md` | `lxagents_skills_security_skill` |
| `references/python-flask-web-server-security.md` | `lxagents_skills_security_python_flask_web_server_security` |
| `references/javascript-typescript-react-web-frontend-security.md` | `lxagents_skills_security_js_ts_react_web_frontend_security` (override) |

The prefix is `{org_name}_{repo_name}` with underscores, so two servers connected to one client
cannot collide. So the folder is dropped, and the ten reference guides get long names. That is the honest
consequence of naming a tool after the guide it serves rather than after its folder — a
caller reading the tool list can tell what each one is without opening anything.

`NAME_OVERRIDES` in `src/instructions/naming.rs` is the escape hatch, for a filename that
derives a name saying nothing useful or too long. **Every published ID is held to 64
characters**, because several clients cap tool names there; the owner chose on 2026-11-11 to
shorten only the base name, never the prefix, so four JavaScript guides use `js` and `js_ts`.
An override carries the base name; the prefix is added to it. Adding one is a naming decision, and it
belongs to the owner, not to whoever happened to add the file.

## The surface is read-only

Every tool is a read, and every tool takes **no argument at all**. Do not add a tool that
takes an argument, a verb, a credential, or a network call.

The property is **structural**: the code that would write is absent, not disabled behind
a check. That is stronger than a permission check on a general-purpose tool, and it is
what a consuming repository depends on when it points at this server — it cannot mutate
the set, because there is nothing here that mutates anything.

Not taking an argument is not a weaker version of the old check; it is the check. There
is no `path` to traverse with, so there is no traversal defence to keep correct.

## What the generator requires of a file

Four things fail the process at startup rather than the first caller, because a set that
cannot be routed on should not start:

* **A `description:` in the frontmatter.** It is the tool description, and it is what a
  caller routes on. A file without one publishes a tool nobody can choose.
* **A filename that derives a valid MCP tool name** — `^[a-z][a-z0-9_]{0,127}$`. A name
  starting with a digit or holding a character the client will not accept is a startup
  error naming the file and the fix.
* **An ID of at most 64 characters.** A longer one is a startup error asking for a
  shortened base in `NAME_OVERRIDES`.
* **A filename no other file derives.** Two files deriving one name would silently shadow
  each other, so the second is a startup error naming both.

Frontmatter is read by a small hand-rolled parser, not a YAML dependency: single-line
scalar fields only. A folded or multi-line value is not read, and a `description:` it
misses shows up as the missing-description error rather than an empty tool description.
Write `description:` on one line.

## Editing the set

`content/` is a copy of an upstream workspace set. It is served verbatim, with its
frontmatter intact, from the next build — there is no draft space inside it. A change to a
guide belongs upstream and is copied here; see [`repository.md`](repository.md).

A file that must **not** be served still has to be a `.md` file to be skipped, so nothing
under `content/` can opt out by other means. `LICENSE.txt` is unserved because it is not
markdown. That is the whole mechanism, and it is why the licence is a `.txt`.

## Errors

A malformed set is a `SurfaceError` naming the file and what to do about it, printed at
startup, with exit code 1. There is no runtime error path to design for: by the time a call
arrives, the set has already been validated.

## Tests

`tests/surface.rs` and the unit tests in `src/instructions/` own the surface, and they are
written to fail if the derivation is broken:

* every `.md` under `content/` has exactly one tool, and every tool maps back to one
  file — the bijection, in both directions, so a guide added and never surfaced is caught
  as loudly as a tool serving a file that is gone;
* every name matches the derivation, and is a valid MCP tool name;
* every input schema is an object with no properties, so no tool takes an argument;
* every tool returns its own file byte for byte, frontmatter included;
* `LICENSE.txt` is not served.

**If a change to `src/server.rs` or `src/instructions/` makes any of those true
of a new tool, the test that should catch it does not exist yet — write it.**
