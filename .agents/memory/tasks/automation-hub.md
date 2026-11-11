---
name: memory-tasks-automation-hub
description: Adding content/automation.md as the routing hub over every served file, and prefixing every tool ID with lxagents_skills_security_.
---

# Task: automation hub and prefixed tool IDs

Four branches, stacked. Version `2.0.2`, a patch, following the owner's earlier choice of patch bumps.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | The record | `plan/automation-hub` | |
| 2 | `content/automation.md`, listing every other file in `content/` with its ID and condition; a test that fails on an orphan | `feat/automation-hub` | |
| 3 | Tool IDs become `lxagents_skills_security_{base}`; `TOOL_NAME` allows 128 characters; overrides and tests follow | `feat/prefixed-tool-ids` | |
| 4 | Release | `release/2.0.2` | |

## Open question for the owner

Four prefixed IDs are longer than 64 characters (the longest is 74). Some clients cap tool names
at 64. The owner is asked whether to shorten those four through `NAME_OVERRIDES`; the PR for
task 3 waits on the answer.

## Progress

### Task 1 — plan/automation-hub

Landed: this record and its row in `memory-index.md`.

### Task 2 — feat/automation-hub

Landed. `content/automation.md` lists the other eleven tools, grouped by language, each with the
condition that activates it. Its frontmatter `name` and `description` match the shared set's
automation tool, with the em dash the owner's text had lost. A new test fails when the hub and the
served names disagree in either direction or a tool is listed twice. The server's `instructions`
text now says to start at `automation`. 64 tests pass.

### Task 3 — feat/prefixed-tool-ids

Landed. Every ID is `lxagents_skills_security_{base}`, built from the exported `ID_PREFIX`.
`TOOL_NAME` admits 128 characters as the owner asked. The owner then chose to keep every
published ID within 64 characters by shortening the base name only, never the prefix. Four
overrides do that: `javascript_general` becomes `js_general`, and `javascript_typescript`
becomes `js_ts` for Next.js, React and Vue. The longest ID is now 64 characters
(`..._javascript_jquery_web_frontend_security`). The `AGENTS.md` override is prefixed too.
The derivation test now checks the prefix, the underscores-only rule, the 64-character cap and
the four overrides. `automation.md`, the server instructions, the README, the overview and the
tool-authoring rule all use the new IDs. 64 tests pass.

### Task 4 — release/2.0.2

Landed. The version is `2.0.2`, a patch, by the owner's standing choice; the changelog says plainly
that callers using the old names break and gives the full rename table. No git tag.
