---
name: memory-state-repository-state
description: Current known state of lxagents-security - the derived per-file tool surface, the read-only set, the stack, and what is not built.
---

# Repository State

## What this repository is right now

`lxagents-security` is a working dual-purpose Skill MCP server and CLI at version `2.0.5`, written in Rust. It
serves the **global security set** read-only. It is no longer a template: `PROMPT.md` and
`template-mode.md` are gone, and the tool surface is derived from the set.

## Stack

Rust 2024 edition (1.88+), on the official MCP Rust SDK `rmcp` 3.10 with `axum` 0.8 and
`tokio`. `cargo build --release` produces both binaries in `target/` (never committed), with
`content/` embedded by `build.rs`. Tests are `cargo test`: unit tests beside the code and
integration tests in `tests/` against the built binaries. See
[`../tasks/rust-migration.md`](../tasks/rust-migration.md). There is no Node.js code left.

## What exists

* **The set.** `content/` holds `automation.md` (the routing hub), `SKILL.md`, the upstream
  Apache-2.0 `LICENSE.txt`, and ten reference guides under `references/`, named
  `<language>-<framework>-<stack>-security.md` across Python, JavaScript/TypeScript, and Go.
* **A derived tool surface.** `src/instructions/` builds one tool per embedded `.md` file,
  once, named after its own filename — twelve tools, each ID prefixed
  `lxagents_skills_security_`. There is no hand-written tool.
* **No argument anywhere.** Every tool returns one file verbatim with its frontmatter intact.
  A call is a lookup over text compiled into the binary: no filesystem I/O at all on the read
  path, and no path for a caller to traverse with.
* **Read-only, structurally.** No tool accepts a verb, takes a credential, or opens a socket.
* **An axum HTTP transport.** `POST /mcp` is stateless Streamable HTTP through rmcp's
  `StreamableHttpService`, selected by `MCP_TRANSPORT=http`, behind the `Host` allow-list, the
  bearer token and the 4 MB JSON body check, in that order. It names its interface with `HOST`
  (default `0.0.0.0`) and drains in-flight requests on `SIGINT`/`SIGTERM`.
* **Worker threads, not processes.** `MCP_CLUSTER_WORKERS` sets tokio worker threads, one per
  CPU by default; `0` or `1` is a single thread. One process, one listener.
* **Surface parity.** The CLI's `tools` and both transports read `instructions::surface()`;
  `tests/surface.rs` pins the CLI list against `tools/list` over stdio, and `tests/http.rs`
  pins HTTP against stdio.
* **Instruction system.** `AGENTS.md` plus `.agents/`, resolving the shared set through
  the `lxagents-agents-base` connector. Local rules: `repository`, `tool-authoring`,
  `secrets`. No overrides.
* **Documentation.** `wiki/information/` and `wiki/environments/`, updated in the same
  commit as the code they describe, plus changelogs at `wiki/logs/0/1/0/` and
  `wiki/logs/1/0/0/`.

## What is not built

* The HTTP transport is stateless and requires a bearer token: `MCP_AUTH_TOKEN`, at least 32
  characters, on every route except `GET /healthz`, and the process will not start without it.
  stdio never reads it. See `../tasks/http-token-auth.md`. `MCP_ALLOWED_HOSTS` filters which
  `Host` values are answered; it says nothing about who is asking. `HOST` defaults to
  `0.0.0.0`, so the port is open on every interface unless the operator narrows it, and the
  allow-list is off unless the operator sets it. There is one shared token, no TLS and no rate
  limiting in this process.
* No CI workflow. `cargo clippy -- -D warnings` and `cargo fmt --check` are run by hand.
* `content/` is a copy. A change to the set belongs upstream in the workspace set first;
  this repository is a delivery surface for it, not its editor. One consequence is on
  record: `SKILL.md` does not enumerate the ten reference filenames, only their two
  documented shapes, so nothing here can make it do so.
* `content/LICENSE.txt` is no longer served. It never was an instruction, but the old
  path-taking tool would return it on request and the new surface does not.

## Shared set

Resolved through the `lxagents-agents-base` Skill MCP connector. Nothing shared is vendored
here, and there are no overrides - see
[`../../index/root-index.md`](../../index/root-index.md).

## Sibling servers

`lxagents-agents-base` carries the org-wide conventions.
`RBAgents-MCP/shared-instruction` carries Roblox development and
`RBAgents-MCP/security` carries Roblox security. Roblox is a different threat model — a
hostile client against a server holding the authority — and is not covered by the
material in `content/`.

## Next obvious step

Add CI that runs `cargo test`, `cargo clippy --all-targets -- -D warnings` and
`cargo fmt --check` on push. The suite is the only thing holding the two surfaces and the
file-to-tool bijection together, and nothing runs it automatically. Build the Docker image
once on a machine with a daemon: it has not been built since the Rust migration.

`tests/http.rs` spawns a child server per test and waits up to 30 seconds for its startup
line, so a slow machine is a slower suite rather than a failure.
