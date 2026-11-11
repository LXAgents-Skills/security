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
