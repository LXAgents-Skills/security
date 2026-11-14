---
name: memory-tasks-rust-migration
description: Rewriting the server from TypeScript to Rust on the official MCP Rust SDK (rmcp) with axum for HTTP. Tool IDs, served text, CLI, env vars and HTTP refusals unchanged.
---

# Task: migrate to Rust

The owner asked for every repository in the organization to move from TypeScript to Rust, one
repository per thread, using the official MCP Rust SDK (`rmcp`) and `axum` as the web framework.
This record is this repository's part.

**Status:** in progress.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | The record | `plan/rust-migration` | |
| 2 | Cargo scaffold: `Cargo.toml`, `Cargo.lock`, `build.rs` embedding `content/`, `rust-toolchain.toml`; the identity and the instruction surface (naming, frontmatter) with unit tests | `build/cargo` | |
| 3 | The Rust server: rmcp handler, config, stdio, the axum HTTP app (Host allow-list, token, body limit, routes), both binaries, integration tests porting the TypeScript suite; the TypeScript tree, `package.json` and lockfile removed; a Rust `Dockerfile` | `refactor/rust` | |
| 4 | README, `wiki/`, `AGENTS.md` and `.agents/` describe the Rust build; an axum decision record supersedes the express one | `docs/rust` | |
| 5 | Release `2.0.5`, a patch by the owner's standing choice | `release/2.0.5` | |

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
