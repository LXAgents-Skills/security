# 2.0.5

**Released:** 2026-11-20

The server is now written in Rust, on the official MCP Rust SDK (`rmcp`) with `axum` for HTTP.
`content/` is embedded in the binary at compile time, so the binary runs from any directory.
Every tool ID, every description, the served text, both transports, the HTTP routes and
refusals, the `MCP_AUTH_TOKEN` rule and every environment variable behave as they did in 2.0.4.

**Clients must:** build with `cargo build --release` and point a local stdio registration at
`target/release/lxagents-security-server` instead of `node dist/index.js`. The npm package is no
longer built from this repository (nothing was unpublished).

```json
{ "command": "./target/release/lxagents-security-server", "cwd": "/path/to/security" }
```

## Added

- **Crate `lxagents-security`**: Rust 2024 edition, `rust-version` 1.88, `rmcp`, `axum`,
  `tokio`, `sha2` and `subtle`; `build.rs` embeds `content/**/*.md`.
- **Binaries**: `lxagents-security-server` (the server) and `lxagents-security` (the CLI, same
  commands and flags as before).
- **Tests**: `cargo test` runs 78 tests (24 unit, 14 surface, 40 HTTP).
- **Decision record** `axum-for-http-transport`, superseding the express record.

## Changed

- **`MCP_CLUSTER_WORKERS`** sets tokio worker threads in one process instead of forked Node
  processes. `0` or `1` runs a current-thread runtime. Process respawn no longer exists.
- **Dockerfile**: a `rust` build stage and a `debian:bookworm-slim` runtime running only the
  binary as uid 10001.
- **Docs**: README, `wiki/information/*`, `wiki/environments/*`, `AGENTS.md` and the agent
  wiki describe the Rust build.

## Removed

- **The TypeScript tree**: `src/*.ts`, `test/`, `package.json`, `package-lock.json` and
  `tsconfig.json`.
- **The cluster process tests** (two PIDs on one port, respawn), replaced by thread tests.
