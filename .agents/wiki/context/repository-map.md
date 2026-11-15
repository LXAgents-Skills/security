---
name: agent-wiki-context-repository-map
description: Orientation for lxagents-security - what lives where, how the tool surface is derived from content/, how to build and test it, and the gotchas that bite first.
---

# Repository Map

Read this before touching anything in `lxagents-security`.

## What this repository is

A Skill MCP server and a CLI over one implementation, serving the **global security
instruction set** read-only — the language and framework guides for Python,
JavaScript/TypeScript, and Go. Rust 2024 edition (1.88+), on the official MCP Rust SDK
(`rmcp`) with `axum` for HTTP. `cargo build --release` puts both binaries in
`target/release/`; `target/` is never committed. `content/` is compiled into the binary.

* Remote: `LXAgents-Skills/security`, default branch `master`.
* Crate: `lxagents-security` (`publish = false`), library `lxagents_security`.
* Bins: `lxagents-security` (CLI) and `lxagents-security-server` (Skill MCP server).

## Layout

```
AGENTS.md                     entry point, connector bootstrap, trigger table
Cargo.toml, Cargo.lock        the crate, its two bins, the version
build.rs                      embeds every content/**/*.md as an include_str! table
rust-toolchain.toml           stable, with rustfmt and clippy
Dockerfile                    two stages: cargo build, then the one binary; MCP_TRANSPORT=http
.dockerignore                 what the build context must not carry
content/                      the published security set - the product
  SKILL.md                    the workflow, and the router into references/
  LICENSE.txt                 Apache-2.0, from the upstream package
  automation.md               the hub: every other tool and the condition that activates it
  references/                 ten guides, <language>-<framework>-<stack>-security.md
src/
  lib.rs                      module wiring only
  identity.rs                 SERVER_ID, SERVER_TITLE, VERSION, ID_PREFIX, the server instructions
  config.rs                   every environment variable but the token, read once into Config
  instructions/
    mod.rs                    the surface, built once from the embedded table; refuses an unroutable set
    naming.rs                 path to tool ID, NAME_OVERRIDES, the name shape, the 64-char cap
    frontmatter.rs            the single-line name/description reader
  server.rs                   the rmcp ServerHandler (list_tools, call_tool)
  run.rs                      picks stdio or HTTP, builds the runtime, checks the token first
  stdio.rs                    the stdio transport
  http/
    mod.rs                    preflight, bind, startup lines, signal and drain
    app.rs                    the axum Router as a pure builder - builds, never listens
    host.rs                   the Host allow-list
    auth.rs                   token_problem, configured_token, require_token
    body.rs                   the 4 MB limit and the JSON check
    rpc_error.rs              the one JSON-RPC error envelope every refusal uses
  bin/
    lxagents-security-server.rs
    lxagents-security.rs      the CLI: help, version, tools, serve
tests/
  common/mod.rs               starts the built binaries; raw HTTP; rmcp clients
  surface.rs                  bijection, derivation, no-argument surface, byte-identical text, CLI parity
  http.rs                     the Streamable HTTP transport, the token, threads and signals, over a real socket
wiki/                         human documentation
.agents/                      this set - rules, agent wiki, memory, indexes
```

Unit tests live beside their code in `#[cfg(test)]` modules.

## Commands

| Command | What it does |
|---|---|
| `cargo build --release` | Both binaries, in `target/release/`. A change to `content/` needs a rebuild. |
| `cargo test` | Unit tests, then the integration tests, which spawn the built binaries. |
| `cargo clippy --all-targets -- -D warnings` | Lints; must be clean. |
| `cargo fmt --check` | Formatting; must be clean. |
| `./target/release/lxagents-security tools` | Lists the tools through the CLI surface. |
| `./target/release/lxagents-security-server` | Serves over stdio. |
| `lxagents-security serve --http --host 127.0.0.1` | Serves Streamable HTTP on `127.0.0.1:PORT` (default 3000). Loopback, so no token is needed. Off loopback, **no token, no start.** |
| `npx @modelcontextprotocol/inspector ./target/release/lxagents-security-server` | MCP Inspector against the stdio server. |

## Environment variables

| Variable | Read by | Effect |
|---|---|---|
| `MCP_TRANSPORT` | `src/config.rs` | `stdio` (default) or `http` (Streamable HTTP on `/mcp`). |
| `PORT` | `src/config.rs` | HTTP port, default `3000`. An unusable value stops the server. |
| `HOST` | `src/config.rs` | Interface the HTTP transport binds, default `0.0.0.0` — every IPv4 interface. |
| `MCP_ALLOWED_HOSTS` | `src/config.rs` | Comma-separated `Host` allow-list for the HTTP transport. **Unset means none is applied**; the server says so on startup. |
| `MCP_CLUSTER_WORKERS` | `src/config.rs` | HTTP worker threads, default one per CPU. **`0` or `1` is a single thread.** One process either way. |
| `MCP_AUTH_TOKEN` | `src/http/auth.rs`, checked in `src/run.rs` | **A secret.** The bearer token every HTTP request except `GET /healthz` must carry; at least 32 characters; HTTP off loopback will not start without it; optional on loopback, enforced if set; stdio never reads it. |

There is no `API_KEY`: nothing here reaches an external service, and no tool reads a credential.
`MCP_ALLOWED_HOSTS` is a filter, not a credential. **HTTP tests need the token too:**
`tests/common/mod.rs` starts every server with `TEST_TOKEN` and its helpers send the matching
header, so a test that spawns the server any other way must do the same or it will read a `401`
as a failure of the thing it is testing. Pass `("MCP_AUTH_TOKEN", "")` with
`("HOST", "0.0.0.0")` to test the refusal to start; blank on the suite's loopback bind is the
local, token-free case.

## The two surfaces

The tool surface is derived, not declared. `build.rs` embeds every `.md` under `content/`, and
`instructions::surface()` builds one tool per file once, naming each after its own filename.
The CLI's `tools`, stdio and HTTP all read that one surface, and `tests/surface.rs` asserts that
what the CLI prints matches what an MCP client receives from `tools/list`. Adding a guide to
`content/` therefore adds a tool to both, after a rebuild.

Twelve tools today. None of them takes an argument.

## Gotchas

* **stdout is the protocol.** On stdio, a `println!` anywhere on the server path corrupts the
  JSON-RPC stream. Log to stderr with `eprintln!`. Only CLI commands print.
* **A change to `content/` needs a rebuild.** The set is compiled in; `build.rs` reruns when
  `content/` changes, but a binary built before the change still serves the old text.
* **Do not hand-write a tool.** Adding a `.md` file to `content/` *is* adding a tool. See
  [`../../rules/tool-authoring.md`](../../rules/tool-authoring.md).
* **A guide without a `description:` in its frontmatter will not start.** The startup error
  names the file. Frontmatter is read by a hand-rolled single-line parser, not a YAML crate.
* **Nothing is unserved except by not being `.md`.** `content/LICENSE.txt` has no tool because
  it is not markdown.
* **The checks wrap the whole route table.** `http/app.rs` puts the routes behind
  `fallback_service` and layers the `Host`, token and body checks on that. Layering them per
  route would run them inside axum's method dispatch, which stamps `Allow:` on a 401.
* **`http/app.rs` must not listen.** It is a builder; `http/mod.rs` owns the port.
* **An unset `MCP_ALLOWED_HOSTS` is the guard being off.** Not "allow nothing". Do not "fix"
  this by refusing everything when the variable is empty — a paste that lost its value would
  take the server down with a 403 on every request.
* **rmcp's own `Host` check is disabled** in `http/app.rs`. The router's layer is the one list.
* **`content/` is the product, not a source folder.** A change to it changes what every
  consuming repository reads.
* **Do not add a write path, and do not add an argument.**
* **The version is `Cargo.toml`'s**, read through `CARGO_PKG_VERSION` in `identity.rs`.

## Where the conventions come from

Branching, commits, pull requests, the task workflow and the creators are **not** in
this repository. They are served by the `lxagents-agents-base` connector and read as
`agents://` resources. This repository carries only what is its own.
