---
name: memory-tasks-code-optimization
description: A targeted structural cleanup - the CLI help gap, one JSON-RPC error envelope, one config reader, and index.js split by responsibility.
---

# Task: optimize the code structure

The full analysis is in the owner's project report, `2026-11-11-security-code-optimization-plan.md`.
The verdict: the code is not spaghetti, and only four steps are needed. No behaviour change, so no
version change.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | The record | `plan/code-optimization` | |
| 2 | List `MCP_AUTH_TOKEN` in the CLI help; drop a dead memory reference | `fix/cli-help-token` | |
| 3 | One JSON-RPC error envelope | `refactor/rpc-error` | |
| 4 | Read configuration in one place | `refactor/config` | |
| 5 | Split `src/index.js` into `http/primary.js`, `http/worker.js`, `stdio.js` | `refactor/http-entry` | |

Not in scope: splitting `test/http.test.js`, caching `McpServer`, renaming the protocol surface.

## Progress

### Task 1 — plan/code-optimization

Landed: this record and its row in `memory-index.md`.

### Task 2 — fix/cli-help-token

Landed. `--help` lists `MCP_AUTH_TOKEN`. A new test scans `src/` for every `env.NAME` read and
fails if `--help` leaves one out; it failed before the fix. The dead
`activation-security.md` reference is gone from `src/app.js` and the express decision.

### Task 3 — refactor/rpc-error

Landed. `src/http/rpc-error.js` is the one JSON-RPC error envelope; `app.js` and the 401 in
`auth.js` both use it. The response bodies are unchanged and the token tests pin them.
