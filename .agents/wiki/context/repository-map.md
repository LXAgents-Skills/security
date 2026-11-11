---
name: agent-wiki-context-repository-map
description: Orientation for lxagents-security - what lives where, how the tool surface is derived from content/, how to build and test it, and the gotchas that bite first.
---

# Repository Map

Read this before touching anything in `lxagents-security`.

## What this repository is

A Skill MCP server and a CLI over one implementation, serving the **global security
instruction set** read-only — the language and framework guides for Python,
JavaScript/TypeScript, and Go. Node.js 20+, ESM (`"type": "module"`), strict TypeScript.
`src/` is compiled by `tsc` to `dist/`, which is what Node runs and what the package ships.
`npm install` builds it through the `prepare` script; `dist/` is never committed.

* Remote: `LXAgents-Skills/security`, default branch `master`.
* Package: `@lxagents-skills/security`.
* Bins: `lxagents-security` (CLI) and `lxagents-security-server` (Skill MCP server).

## Layout

```
AGENTS.md                     entry point, connector bootstrap, trigger table
package.json                  both bins on dist/; build, prepare, typecheck, test scripts
tsconfig.json                 ES2022, NodeNext, strict; src/ to dist/
Dockerfile                    two stages: build with tsc, run dist/; MCP_TRANSPORT=http
.dockerignore                 what the build context must not carry
content/                      the published security set - the product
  SKILL.md                    the workflow, and the router into references/
  LICENSE.txt                 Apache-2.0, from the upstream package
  automation.md               the hub: every other tool and the condition that activates it
  references/                 ten guides, <language>-<framework>-<stack>-security.md
src/
  index.ts                    entry point; reads the config and calls run()
  run.ts                      picks stdio or streamable HTTP, primary or worker
  config.ts                   every environment variable but the token, read once into a frozen object
  stdio.ts                    the stdio transport
  http/
    primary.ts                forks the workers, respawns them, drains them; checks the token first
    worker.ts                 listens, logs the startup lines, drains on a signal
    app.ts                    the express application as a pure factory - builds, never listens
    auth.ts                   the bearer token: tokenProblem, configuredToken, requireBearerToken
    rpc-error.ts              the one JSON-RPC error envelope every refusal uses
  server.ts                   builds the McpServer and registers the generated surface; exports listTools()
  cli.ts                      the CLI: help, version, tools, serve
  version.ts                  ROOT, CONTENT_DIR, and the version out of package.json
  instructions/
    from-content.ts           builds the whole instruction surface from content/, once, at import
test/
  tsconfig.json               extends the root config with noEmit, so typecheck covers the tests
  server.test.ts              bijection, derivation, no-argument surface, served totals, parity
  http.test.ts                the streamable HTTP transport and the workers, over a real socket
  config.test.ts              the configuration defaults and parsing
wiki/                         human documentation
.agents/                      this set - rules, agent wiki, memory, indexes
```

## Commands

| Command | What it does |
|---|---|
| `npm install` | Installs the dependencies, then compiles `src/` to `dist/` (`prepare`). |
| `npm run build` | `tsc`: recompiles `dist/` after a change to `src/`. |
| `npm run typecheck` | `tsc -p test/tsconfig.json`: checks `src/` and `test/` together, writes nothing. |
| `npm test` | Builds first (`pretest`), then `node --import tsx --test test/*.test.ts`. The suite spawns the compiled `dist/` server; there is no watch mode. |
| `npm run cli -- tools` | Lists registered tools through the CLI surface. |
| `npm start` | Serves over stdio. |
| `npm run start:http` | Serves over streamable HTTP on `127.0.0.1:PORT` (default 3000). Loopback, so no token is needed. Off loopback, **no token, no start.** |
| `npm run inspect` | MCP Inspector against the stdio server. |

## Environment variables

| Variable | Read by | Effect |
|---|---|---|
| `MCP_TRANSPORT` | `src/config.ts` | `stdio` (default) or `http` (Streamable HTTP on `/mcp`). |
| `PORT` | `src/config.ts` | HTTP port, default `3000`. |
| `HOST` | `src/config.ts` | Interface the HTTP transport binds, default `0.0.0.0` — every IPv4 interface. |
| `MCP_ALLOWED_HOSTS` | `src/config.ts` | Comma-separated `Host` allow-list for the HTTP transport. **Unset means none is applied**; the server says so on startup. |
| `MCP_CLUSTER_WORKERS` | `src/config.ts` | HTTP worker count, default one per CPU. **`1` forks nothing.** stdio never forks. |
| `MCP_AUTH_TOKEN` | `src/http/auth.ts`, checked in `src/http/primary.ts` | **A secret.** The bearer token every HTTP request except `GET /healthz` must carry; at least 32 characters; HTTP off loopback will not start without it; optional on loopback, enforced if set; stdio never reads it. |

There is no `API_KEY`: nothing here reaches an external service, and no tool reads a credential.
`MCP_ALLOWED_HOSTS` is a filter, not a credential — it decides which `Host` values are answered,
not who is asking; `MCP_AUTH_TOKEN` is what decides that. **HTTP tests need the token too:** the
harness in `test/http.test.ts` hands every child the variable and the SDK client the matching
header, so a test that spawns the server any other way, or fetches `/mcp` raw, must do the same
or it will read a `401` as a failure of the thing it is testing. Blank it
(`MCP_AUTH_TOKEN: ""`) with `HOST: "0.0.0.0"` to test the refusal to start; blank on the
suite's loopback bind is the local, token-free case.

## The two surfaces

The tool surface is derived, not declared. `src/instructions/from-content.ts` walks `content/`
once at import and builds one tool per `.md` file, naming each after its own filename.
`src/server.ts` registers that array and exports `listTools()`; `src/cli.ts` imports
`listTools()` from it rather than keeping its own, and `test/server.test.ts` asserts that
what the CLI would print matches what an MCP client receives from `tools/list`. Adding a
guide to `content/` therefore adds a tool to both, and there is no way to add it to only
one without failing the suite.

Eleven tools today. None of them takes an argument.

## Gotchas

* **stdout is the protocol.** On stdio, a `console.log` anywhere on the server path
  corrupts the JSON-RPC stream. Log to stderr. Only CLI commands print.
* **Do not hand-write a tool module.** Adding a `.md` file to `content/` *is* adding a
  tool. A module under `src/instructions/` beside `from-content.js` puts two tools on the
  surface for one file and is invisible to the derivation. See
  [`../../rules/tool-authoring.md`](../../rules/tool-authoring.md).
* **A guide without a `description:` in its frontmatter will not boot.** That is the
  startup error you will see first, and it names the file. Frontmatter is read by a
  hand-rolled single-line parser, not a YAML dependency.
* **Nothing is unserved except by not being `.md`.** `content/LICENSE.txt` has no tool
  because it is not markdown. Nothing else in the set opts out.
* **A fresh `McpServer` per HTTP request.** `src/http/app.ts` builds and closes one per
  request because `McpServer` holds per-connection state. Do not hoist it to module
  scope, and note that with workers on, that request may be answered by any of N
  separate processes.
* **`src/http/app.ts` must not call `listen()`.** It is a factory; `src/http/worker.ts` owns the
  port and the worker count. A file that both builds and binds cannot be tested without
  opening one.
* **An unset `MCP_ALLOWED_HOSTS` is the guard being off.** Not "allow nothing". The
  startup line says `MCP_ALLOWED_HOSTS is unset` when there is no list, and `HOST`
  defaults to `0.0.0.0`, so the unguarded state is the default one. Do not "fix" this by
  refusing everything when the variable is empty — a paste that lost its value would
  take the server down with a 403 on every request.
* **`content/` is the product, not a source folder.** Every file in it is served
  verbatim on the next boot, with its frontmatter intact. `src/` is local; a change to
  `content/` changes what every consuming repository reads.
* **Do not add a write path, and do not add an argument.** The read-only surface is the
  property a consuming repository depends on, and no argument means there is no path to
  traverse with.
* **`version.ts` reads `package.json` at import** via a path one level above its own
  directory, which holds from `src/` under `tsx` and from `dist/` when compiled, and
  holds `CONTENT_DIR` for the generator. Moving it breaks the version and every served
  file without failing a test.

## Where the conventions come from

Branching, commits, pull requests, the task workflow and the creators are **not** in
this repository. They are served by the `lxagents-agents-base` connector and read as
`agents://` resources. This repository carries only what is its own.
