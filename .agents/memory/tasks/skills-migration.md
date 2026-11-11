---
name: memory-tasks-skills-migration
description: Integrating the stale http-token-auth branches into master, then moving the organization to LXAgents-Skills and the internal vocabulary from tools to instructions.
---

# Task: integrate the token-auth branches and migrate to LXAgents-Skills

Three branches, stacked in this order. **No version change**: the package scope moves, which
a consumer installing by name will notice, so the version is left for the owner to approve.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | The record | `plan/skills-migration` | |
| 2 | Merge `feat/http-token-auth`, `chore/http-token-auth-plan`, `release/2.0.0` in that order | `chore/http-token-auth-integration` | |
| 3 | `LXAgents-MCP` to `LXAgents-Skills`, MCP to Skill MCP in prose, tools to instructions in internal code | `refactor/skills-terminology` | |

The owner named no release, so there is no `release/{version}` slot in this chain; the version
for the package-scope move is recorded as an open question below.

## What stays

* `@modelcontextprotocol/sdk`, `server.tool`, `listTools()`, `tools/list`, `tools/call`: the
  external protocol surface is not renamed.
* `MCP_AUTH_TOKEN`: an environment variable a deployment already sets.
* Released change logs under `wiki/logs/`: a published release note is history.
* Mentions of the protocol itself (an MCP client, the MCP Inspector, a valid MCP tool name),
  which would become false if renamed.

## Progress

### Task 1 — plan/skills-migration

Landed: this record and its row in `memory-index.md`.

### Task 2 — chore/http-token-auth-integration

Landed. The three branches were the pre-rebase copies of pull requests #8 to #10, whose
rebased commits master already carried, so the merges add history and no content: after the
third merge the tree equals `origin/release/2.0.0`. `feat/http-token-auth` conflicted in the
`Dockerfile`, `wiki/environments/docker.md` and the token-auth record; `chore/http-token-auth-plan`
conflicted in the record again. Every conflict was resolved to master's side, which is the
released `2.0.0` state the owner asked to finish on. `release/2.0.0` merged cleanly.

### Task 3 — refactor/skills-terminology

Landed. `src/tools/` is `src/instructions/`; `TOOL_MODULES`, `CONTENT_TOOLS` and `TOOL_FILES` are
`INSTRUCTION_MODULES`, `CONTENT_INSTRUCTIONS` and `INSTRUCTION_FILES`. The organization is
`LXAgents-Skills` and the package `@lxagents-skills/security` in `package.json`, the lockfile,
README, LICENSE, the wiki and the repository map. Self-descriptions say Skill MCP server; mentions
of the protocol, its clients and the Inspector keep MCP. Released logs and closed task records keep
the old names. `npm test` runs 58 tests, all passing.

Open for the owner: the package name changed, so the next publish is a new package. Its version
needs approval, and `npm deprecate` on `@lxagents-mcp/security` is worth doing once it ships.
