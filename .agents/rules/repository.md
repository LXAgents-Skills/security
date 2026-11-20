---
name: repository-rules
description: Rules specific to lxagents-security - the dual-surface contract, the stdout ban, the derived read-only surface, and what must not be introduced.
---

# Repository Rules

`lxagents-security` is a dual-purpose Skill MCP server and CLI over one implementation, and
it serves the global security set from `content/` read-only. Both facts constrain what
may be changed here.

## Mode and shared set

This repository is a **Mode B consumer**. The shared instruction set is resolved
through the `lxagents-agents-base` Skill MCP connector and is never copied into this tree.
See the bootstrap block in [`../../AGENTS.md`](../../AGENTS.md).

## The two surfaces stay in step

The CLI (`lxagents-security`) and the Skill MCP server (`lxagents-security-server`) are two
doors onto one implementation. A tool reachable from one is reachable from the other,
with the same name and the same description.

* The tool surface is built in exactly one place: `src/instructions/mod.rs`, from the
  files in `content/` that `build.rs` embeds.
* `src/server.rs` serves that surface; it does not declare tools of its own.
* `src/bin/lxagents-security.rs` never maintains its own list - it reads
  `instructions::surface()`.
* `tests/surface.rs` pins the agreement. A change that makes the two surfaces
  disagree fails the suite, and that failure is the point.

## Nothing writes to stdout except the CLI

On the stdio transport, stdout **is** the JSON-RPC channel. A stray `println!` on
the server path corrupts the protocol stream and the client reports a parse error
that names nothing useful.

* Server-side logging goes to stderr.
* `serve` prints nothing of its own.
* Only CLI commands write to stdout.

## Where things go

| Thing | Path |
|---|---|
| The served security set | `content/` |
| Embedding the set into the binary | `build.rs` |
| The tool surface, derived from the set | `src/instructions/` |
| The MCP handler | `src/server.rs` |
| The server's identity and the version | `src/identity.rs` (version from `Cargo.toml`) |
| CLI commands | `src/bin/lxagents-security.rs` |
| Build output, never committed | `target/` |
| Transport and entry point | `src/bin/lxagents-security-server.rs` → `src/run.rs` → `src/stdio.rs` or `src/http/` |
| Tests | unit tests in `#[cfg(test)]` beside the code; integration tests in `tests/{subject}.rs` |

## Commands

```bash
cargo build --release                       # Rust 1.88+; both binaries in target/release/
cargo test                                  # unit, then integration tests against the binaries
cargo clippy --all-targets -- -D warnings
cargo fmt --check
./target/release/lxagents-security tools
./target/release/lxagents-security-server   # stdio
./target/release/lxagents-security serve --http --host 127.0.0.1
```

## What must not be introduced

* Committed build output. `target/` is built by Cargo and is ignored by git; a checked-in
  copy would drift from its source.
* A second build tool. Cargo is the build, and `build.rs` does one thing: list `content/`
  into the embedded table.
* A second source of truth for the tool list, and a hand-written tool module beside the
  generator - see [`tool-authoring.md`](tool-authoring.md).
* Shared instruction content. If it can be read from `agents://`, it must not exist
  here as a file.
* A write path, and an argument. No tool may take a verb, a credential, or reach a
  network; no tool takes any argument at all. The read-only property is structural - the
  code that would write is absent - and it is the property a consuming repository depends
  on when it points at this server.
* `unsafe` code, anywhere in the crate - `src/`, `build.rs`, `tests/`. `Cargo.toml` sets
  `unsafe_code = "forbid"`, so the compiler refuses it and no `#[allow]` can lift it. That
  includes `std::env::set_var` in a test: pass the value to the spawned binary instead.
* A clippy warning. `cargo clippy --all-targets -- -D warnings` is clean before every
  commit; a lint is fixed, never silenced with `#[allow]` unless the line says why.
* An edit to a guide under `content/`. Those files are copied from the upstream
  workspace set; the change belongs there, and this repository follows it.
