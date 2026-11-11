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

### Task 4 — refactor/config

Landed. `src/config.js` reads `MCP_TRANSPORT`, `PORT`, `HOST`, `MCP_CLUSTER_WORKERS` and
`MCP_ALLOWED_HOSTS` into one frozen object; `index.js` and `createApp` take it instead of reading
the environment themselves. `MCP_AUTH_TOKEN` stays in `auth.js`, read at call time. The
exported `allowedHosts()` is gone; `createApp` takes `allowedHosts` as an option.
`test/config.test.js` pins the defaults and the parsing.

### Task 5 — refactor/http-entry

Landed. `src/index.js` is a three-line entry point over `src/run.js`, which imports
`src/stdio.js`, `src/http/primary.js` or `src/http/worker.js` once the transport is chosen.
`app.js` and `auth.js` moved into `src/http/`. `cli.js serve` passes its flags to `loadConfig`
instead of writing `process.env`; a forked worker re-runs it with the same arguments. Every log
line, both bins and `package.json` `main` are unchanged. Checked by hand as well as by
`npm test` (63 tests): `cli.js serve --http` with two workers, the 401, SIGTERM drain, the
refusal without a token, and stdio `tools/list`. The architecture page and repository map
show the new layout; the architecture tree still said `tools/` after the migration and now
says `instructions/`.

### Task 6 — release/2.0.1

Landed. The owner asked for a release branch and a patch bump only, so the version is `2.0.1`. It
also ships the earlier Skills migration, which had no release of its own. `package.json`, the
lockfile, the image tags in the docs and the Dockerfile comments, `wiki/logs/2/0/1/CHANGELOG.md`
and its `logs-index.md` row carry it. No git tag was created.
