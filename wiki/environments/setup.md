# Local Setup

`lxagents-security` is **dual-purpose**. The same code is reachable two ways:

| Mode | What it is | Who uses it |
|---|---|---|
| **CLI mode** | A terminal command | A person running it by hand or from a script |
| **Server mode** | A Skill MCP server over stdio or streamable HTTP | An MCP client, an editor, an agent, or a connector |

Both modes share one implementation, so a result produced in one is identical to the
same result produced in the other.

## Requirements

Rust 1.88 or newer, with Cargo. `rust-toolchain.toml` selects the stable toolchain with
`rustfmt` and `clippy`.

```bash
git clone https://github.com/LXAgents-Skills/security
cd security
cargo build --release   # both binaries, in target/release/
cargo test              # unit tests, then integration tests against the built binaries
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

`content/` is compiled into the binary by `build.rs`, so a change to the set takes a rebuild,
and the built binary runs from any directory with nothing beside it. `target/` is never
committed.

An existing local registration that runs `node dist/index.js` must be changed to run
`target/release/lxagents-security-server`: there is no Node.js code left in this repository.

There is also a container image — see [`docker.md`](docker.md). It is not built by
`cargo test`.

## Authentication: HTTP only

Nothing here reaches an external service, so no tool needs a key, and **stdio needs no token
and no configuration**: the server starts, lists its twelve tools, and answers.

**A local install needs no token.** stdio never reads one, and HTTP bound to loopback
(`127.0.0.1`, `localhost`, `::1`) runs without one too, limited to a `localhost` `Host` header so a
web page cannot reach it by DNS rebinding. **A server install needs one:** HTTP bound to any other
address — the `0.0.0.0` default, a container, a public host — will not start without it. A token
that is set is always enforced, on any bind. The token is at least 32
characters, and every request except `GET /healthz` must carry it as
`Authorization: Bearer <token>`. The other environment variables — `MCP_TRANSPORT`, `PORT`,
`HOST`, `MCP_ALLOWED_HOSTS` and `MCP_CLUSTER_WORKERS` — are optional. `MCP_ALLOWED_HOSTS` is a
`Host` allow-list for the HTTP transport and is **off unless you set it**; the server says so
on startup. Set it before exposing the port anywhere but your own machine. Full list:
[`env.md`](env.md).

## CLI mode

### Install

```bash
# From a checkout: build, then use the binary in place
cargo build --release
./target/release/lxagents-security --help

# Or put both binaries on PATH (~/.cargo/bin)
cargo install --path .
```

### Use

```bash
lxagents-security --help
lxagents-security --version
lxagents-security tools
```

`tools` prints every registered tool with its description:

```text
lxagents_skills_security_automation                               Read this tool every session, first — …
lxagents_skills_security_golang_general_backend_security          Go (Golang) Security Spec (Go 1.25.x, Standard Library, net/http) — …
…
lxagents_skills_security_skill                                    Perform language and framework specific security best-practice reviews …
```

One line per file in `content/`, named after that file: twelve lines today.

The list comes from `instructions::surface()`, the same list the Skill MCP server serves, so
the two surfaces cannot disagree.

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | The request was understood but could not be satisfied |
| `2` | The command line itself was wrong |

## Server mode

### Install

An MCP client spawns the server as a subprocess, so installing it means pointing the
client at it. Either bin works: `lxagents-security-server` is the server directly, and
`lxagents-security serve` reaches the same server through the CLI.

```json
{
  "mcpServers": {
    "lxagents-security": {
      "command": "/path/to/security/target/release/lxagents-security-server"
    }
  }
}
```

After `cargo install --path .`, the binary can be named directly instead:

```json
{
  "mcpServers": {
    "lxagents-security": {
      "command": "lxagents-security-server"
    }
  }
}
```

For a remote connector, point the client at `https://<host>/mcp`, including the
`/mcp` path.

### Run

```bash
# stdio
lxagents-security-server
lxagents-security serve --stdio

# streamable HTTP, local - binds 127.0.0.1, needs no token
lxagents-security serve --http --host 127.0.0.1 --port 3000

# streamable HTTP, server - needs a token, or it exits 1 and says why
export MCP_AUTH_TOKEN="$(openssl rand -hex 32)"
lxagents-security serve --http --host 0.0.0.0 --port 3000
```

`--host` overrides `HOST` for one run. Without either, the server binds `0.0.0.0` and needs a
token.

Check it is up:

```bash
curl -s http://localhost:3000/healthz
```

### Inspect it

```bash
npx @modelcontextprotocol/inspector ./target/release/lxagents-security-server
```

This runs the MCP Inspector (a Node.js tool, fetched on demand) against the stdio server,
listing every tool and letting you call them.

### stdout belongs to the protocol

On the stdio transport, stdout **is** the JSON-RPC channel. Logging goes to stderr,
and `serve` prints nothing of its own. Only CLI commands write to stdout.

A `println!` on the server path is a bug that corrupts the protocol stream.

## Related pages

- [`env.md`](env.md) — every environment variable this project reads
- [`docker.md`](docker.md) — building and running the container image
- [`../information/overview.md`](../information/overview.md) — what this project is
- [`../information/architecture.md`](../information/architecture.md) — how the pieces fit
- [`README.md`](../../README.md)
