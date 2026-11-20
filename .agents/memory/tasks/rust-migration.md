---
name: memory-tasks-rust-migration
description: Rewriting the server from TypeScript to Rust on the official MCP Rust SDK (rmcp) with axum for HTTP. Tool IDs, served text, CLI, env vars and HTTP refusals unchanged.
---

# Task: migrate to Rust

The owner asked for every repository in the organization to move from TypeScript to Rust, one
repository per thread, using the official MCP Rust SDK (`rmcp`) and `axum` as the web framework.
This record is this repository's part.

**Status:** done.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | The record | `plan/rust-migration` | #22 |
| 2 | Cargo scaffold: `Cargo.toml`, `Cargo.lock`, `build.rs` embedding `content/`, `rust-toolchain.toml`; the identity and the instruction surface (naming, frontmatter) with unit tests | `build/cargo` | #23 |
| 3 | The Rust server: rmcp handler, config, stdio, the axum HTTP app (Host allow-list, token, body limit, routes), both binaries, integration tests porting the TypeScript suite; the TypeScript tree, `package.json` and lockfile removed; a Rust `Dockerfile` | `refactor/rust` | #24 |
| 4 | README, `wiki/`, `AGENTS.md` and `.agents/` describe the Rust build; an axum decision record supersedes the express one | `docs/rust` | #25 |
| 5 | Release `2.0.5`, a patch by the owner's standing choice | `release/2.0.5` | #26 |

**What stays:** every tool ID and description, the served text byte for byte, the server ID,
title and instructions, the CLI commands and exit codes, every environment variable, the
`MCP_AUTH_TOKEN` rule (required for HTTP unless the bind is loopback), and every HTTP route and
refusal: `POST /mcp` stateless, `GET /healthz` open, the Host allow-list before the token check
before the 4 MB body read, and the one JSON-RPC error envelope.

**What changes for a consumer:** the server is built with `cargo build --release` and run as
`target/release/lxagents-security-server`; the CLI is `target/release/lxagents-security`. A
local MCP registration moves from `node dist/index.js` to that binary. The npm package is no
longer built from this repository.

**Decision - `content/` is embedded at compile time.** `build.rs` generates an `include_str!`
table of every `.md` under `content/`, so the binary serves the set without a path to it and
the request path does no filesystem I/O. The derivation and its checks (valid name, no
collision, a description) still run once at startup, so a malformed set still fails the process.

**Decision - `MCP_CLUSTER_WORKERS` sets tokio worker threads.** Node needed `cluster` to use
more than one core; one tokio multi-thread runtime already does. Unset is one thread per CPU,
`0` or `1` is a single-threaded runtime, `N` is `N` threads. There are no child processes, so
the respawn loop and its crash limit are gone.

**Decision - stateless Streamable HTTP with JSON responses.** rmcp's `StreamableHttpService`
with session mode off and `json_response` on, since every tool answers in one message.

## Progress

### Task 1 - plan/rust-migration

Landed: this record and its row in `memory-index.md`.

### Task 2 - build/cargo

Landed: `Cargo.toml` (package `lxagents-security`, edition 2024, `rust-version` 1.88, version
still `2.0.4` to match `package.json` until the release task), `Cargo.lock`,
`rust-toolchain.toml` (stable, with rustfmt and clippy), and `build.rs`, which writes an
`include_str!` table of every `.md` under `content/` into `OUT_DIR`. The library has
`identity.rs` (server ID, title, version from `CARGO_PKG_VERSION`, the tool prefix and the server
instructions) and `instructions/` (`naming.rs`: derivation, overrides, the name shape and the
64-character cap; `frontmatter.rs`: the single-line field reader; `mod.rs`: the surface, built
once, refusing an invalid or long name, a collision, a missing description or an empty set).
The surface sorts paths case-insensitively, so the tool order matches the TypeScript server's
`localeCompare` order. `/target/` is ignored by git and Docker.

Verified: 14 unit tests pass; clippy with `-D warnings` and `cargo fmt --check` are clean; the
TypeScript suite still passes 69 of 69, since nothing it uses changed.

### Task 3 - refactor/rust

Landed. Dependencies: `rmcp` 3.10 (`server`, `transport-io`, `transport-streamable-http-server`),
`axum` 0.8 (`http1`, `tokio`, `json`), `tokio`, `tokio-util`, `serde_json`, `sha2`, `subtle`,
`http-body-util`; dev: `rmcp` client transports, `reqwest`, `tokio` full. Modules, one
responsibility each: `config.rs` (every variable, read once at the entry point), `server.rs`
(the rmcp `ServerHandler`: `list_tools` with an empty-object input schema, `call_tool` as a
lookup), `stdio.rs`, `run.rs` (transport and runtime choice), `http/app.rs` (the router),
`http/auth.rs`, `http/host.rs`, `http/body.rs`, `http/rpc_error.rs`, `http/mod.rs` (preflight,
bind, startup lines, drain), and `src/bin/lxagents-security-server.rs` and
`src/bin/lxagents-security.rs`. The TypeScript tree, `test/`, `package.json`, the lockfile and
`tsconfig.json` are deleted. The `Dockerfile` builds in `rust:1-slim-bookworm` and runs the one
binary as uid 10001 on `debian:bookworm-slim`.

Found while porting: axum's per-route layers run inside method dispatch, which stamped
`Allow: POST` on a 401 for `GET /mcp` and told a caller without the token which methods the path
takes. The checks now wrap the route table as one service (`fallback_service`), and a test pins
that no 401 carries `Allow`. rmcp's own `Host` check is disabled so the router's layer is the
one place that decision is made. A stdio pipe closed before the handshake now exits 0, as the
Node server did.

Verified: stdio output against 2.0.4 captured before the change: the same 12 tools in the same
order, identical descriptions, input schemas, server name, title and instructions, and every
tool's text byte-identical. `tools` output is byte-identical; `--help` differs only in the
`MCP_CLUSTER_WORKERS` line. 78 tests pass (24 unit, 14 surface, 40 HTTP); clippy with
`-D warnings` and `cargo fmt --check` are clean. The release binary is 4.2 MB and serves from a
directory with no `content/`. `docker build` could not run: the sandbox has no Docker daemon.

Not carried over, because there is no process to test: the cluster tests for two PIDs on one
port, respawning a killed worker, and orphaned workers after the primary is killed. Their thread
counterparts (`MCP_CLUSTER_WORKERS=1` and `=N`, one drain line, a second signal) are in
`tests/http.rs`.

### Task 4 - docs/rust

Landed: `README.md`, `wiki/information/overview.md`, `wiki/information/architecture.md`
(rewritten for the crate's modules, the check order and the thread model),
`wiki/environments/setup.md`, `env.md` and `docker.md`, `AGENTS.md` (stack line and version
carrier), `.agents/wiki/context/repository-map.md`, the facts in `.agents/rules/repository.md`
and `.agents/rules/tool-authoring.md` (paths, commands, build output; no rule added), and
`.agents/memory/state/repository-state.md`. New decision record
`decisions/axum-for-http-transport.md`, which supersedes the express record; both rows are in
`memory-index.md`.

Removed from `.agents/rules/repository.md` because it can no longer apply: the TypeScript-only
"no `any`" bullet. A Rust replacement is offered as a discovery finding, not written.
Left as it is: `.agents/rules/secrets.md`, whose example is a TypeScript handler for a
hypothetical credentialed tool; also offered as a finding.

### Task 5 - release/2.0.5

Landed: `Cargo.toml` and `Cargo.lock` at `2.0.5`, `wiki/logs/2/0/5/CHANGELOG.md`, its row in
`logs-index.md`, the image tags in the Dockerfile comments, `docker.md` and `env.md`, and
`repository-state.md`. The run's plan file under `.agents/plans/` is removed. PRs #22-#26,
rebase-merged in order.
