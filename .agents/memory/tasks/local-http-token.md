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

### Task 2 — feat/local-http-token

Landed. `src/config.js` exports `isLoopback` and sets `config.local`. `tokenProblem` takes
`{ required }`: no token is fine when it is false, but a short token is still refused.
`createApp({ local })` builds without the token check only when `local` is set and the token is
empty, and it then mounts the SDK's `localhostHostValidation` unless `MCP_ALLOWED_HOSTS` names a
list. The primary requires the token only off loopback. The startup line says which control is
on. `serve --host` was added, and `npm run start:http` binds `127.0.0.1`, so it runs from
PowerShell with nothing set. The refusal tests now bind `0.0.0.0` (they exit before binding).
New tests cover local without a token on 1 and 2 workers, the localhost-only `Host` guard, a
token still enforced locally, a short token refused locally, and `isLoopback`. 69 tests pass.
The README, setup, env, architecture and repository-map docs say local versus server.

### Task 3 — release/2.0.3

Landed. Version `2.0.3`. The changelog says clients need do nothing, and notes that anyone who
used `start:http` to serve other machines must now pass `--host 0.0.0.0` with the token set.
