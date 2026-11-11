---
name: memory-tasks-local-http-token
description: Requiring MCP_AUTH_TOKEN only when HTTP is reachable from a network - loopback binds run without one, behind a localhost Host allow-list.
---

# Task: no token for a local install

The owner said that a local install should not need `MCP_AUTH_TOKEN`, and a server install
should. Checked first: stdio, the local install every client spawns, already needed no token.
The local case that did need one was HTTP on `npm run start:http`, which bound `0.0.0.0` exactly
as a server does.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | The record | `plan/local-http-token` | |
| 2 | A token is optional when HTTP binds loopback, and is then replaced by the localhost `Host` allow-list. Any other bind still refuses to start without a token. `--host` flag; `start:http` binds `127.0.0.1` | `feat/local-http-token` | |
| 3 | Release `2.0.3`, a patch by the owner's standing choice | `release/2.0.3` | |

What stays: the Docker image defaults to HTTP on `0.0.0.0`, so it still needs the token. A token
that is set but too short is still refused, even on loopback.

## Progress

### Task 1 — plan/local-http-token

Landed: this record and its row in `memory-index.md`.
