# Overview

`lxagents-security` is a **dual-purpose** Rust crate: the same implementation is
reachable as a terminal command and as a Skill MCP server. It serves one thing — the global
security instruction set — and it serves it read-only.

## The two surfaces

| Surface | Bin | Who uses it |
|---|---|---|
| CLI | `lxagents-security` | A person running it by hand or from a script |
| Skill MCP server | `lxagents-security-server` | An MCP client, an editor, an agent, or a connector |

Both share one implementation and one tool list, so a result produced through one is
identical to the same result produced through the other. `cargo test` pins that agreement.

## The tools

One tool per markdown file in `content/`, so 12 tools. Each ID is `lxagents_skills_security_`
plus the name of the file it serves — the folder is dropped, the `.md` goes, the name is
lowercased and `-` becomes `_`. `SKILL.md` is therefore `lxagents_skills_security_skill`, and
`references/python-flask-web-server-security.md` is
`lxagents_skills_security_python_flask_web_server_security`. Four JavaScript guides shorten
`javascript` to `js` and `javascript_typescript` to `js_ts`, so every ID fits in 64 characters.

**No tool takes an argument.** The names are long because they name the guide they serve
rather than the folder holding it; that is the cost of a caller being able to read the
tool list and know what each tool is for.

`lxagents_skills_security_automation` is the hub: it lists every other tool with the one condition that activates
it, and it is the only tool read unprompted. `lxagents_skills_security_skill` names the language-and-framework workflow and points at the
ten reference files under `references/`, one per language and framework, named
`<language>-<framework>-<stack>-security.md`. The general file for a language comes
before the framework-specific one, and the pair is written to be read in that order.

`content/LICENSE.txt` is **not** served. A licence is not an instruction, and it has no
frontmatter for a caller to route on.

## Read-only, structurally

The server has no write path. No tool accepts a verb, no tool takes a credential, and no
tool opens a socket. The code that would write is absent rather than disabled, which is
what makes "pointing a repository at this server cannot mutate the set" a property of the
code rather than a configuration someone can change.

The read-only claim is also structural on the way in. The set is compiled into the binary
and every tool is a lookup over that text, so there is no path for a caller to traverse with
and no filesystem I/O on the read path at all.

## What ships

* A Skill MCP server over **stdio** and **streamable HTTP**, with a `/healthz` endpoint on the
  HTTP transport.
* A CLI with `help`, `version`, `tools`, and `serve`.
* A tool layer **derived from `content/` at build time and checked at startup** — one tool per file, no hand-written
  tool module, no argument anywhere.
* A `Host` allow-list for the HTTP transport that is **off unless it is set**, and says
  so on startup when it is off.
* A test suite covering the bijection between files and tools in both directions, the
  derivation, that no tool declares an input schema, that the served text totals the text
  on disk, that the licence is not served, the structural claim that no tool takes a verb
  or a credential, and — over a real socket — the route table, tool parity, concurrent
  requests, the `Host` allow-list, and what a shutdown does.

## Requirements

Rust 1.88 or newer, to build. The server is the official MCP Rust SDK (`rmcp`) with `axum`
for HTTP, on `tokio`. `cargo build --release` produces both binaries in `target/release/`,
which is never committed. The built binary needs nothing beside it: no runtime, no
`content/` folder.

## Related pages

* [`architecture.md`](architecture.md) — how the pieces fit together.
* [`../environments/setup.md`](../environments/setup.md) — installing and running it.
* [`../environments/env.md`](../environments/env.md) — environment variables.
