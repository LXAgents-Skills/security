# Architecture

One Rust crate: a library holding every responsibility in its own module, and two thin
binaries over it. `content/` is compiled into the binary by `build.rs`, so the server needs no
path to the set at run time.

```
build.rs              lists content/**/*.md into an include_str! table in OUT_DIR
src/
  lib.rs              module wiring only
  identity.rs         server ID, title, version (from Cargo.toml), tool prefix, server instructions
  config.rs           every environment variable but the token, read once into a Config value
  instructions/
    mod.rs            the surface: built once from the embedded table, refused if unroutable
    naming.rs         path to tool ID: folder dropped, .md off, kebab to snake, overrides, 64-char cap
    frontmatter.rs    the single-line name/description reader
  server.rs           the rmcp ServerHandler: list_tools and call_tool over the surface
  run.rs              picks stdio or HTTP and builds the tokio runtime it needs
  stdio.rs            the stdio transport
  http/
    mod.rs            token preflight, bind, startup lines, signal and drain
    app.rs            the axum Router, as a pure builder: builds, never listens
    host.rs           the Host allow-list middleware
    auth.rs           the bearer token: the strength check and the middleware that enforces it
    body.rs           the 4 MB limit and the JSON check
    rpc_error.rs      the one JSON-RPC error envelope every refusal uses
  bin/
    lxagents-security-server.rs   reads the process environment and calls run
    lxagents-security.rs          the CLI: help, version, tools, serve
tests/                integration tests that drive the built binaries
content/              the published security set
```

```
bin/lxagents-security ─┐
bin/lxagents-security-server ─▶ config ─▶ run ─┬─▶ stdio ──────────────┐
                                               └─▶ http ─▶ http/app ─────┼─▶ server ─▶ instructions ─▶ identity
                                                    host · auth · body · rpc_error
```

**Nothing in the library starts anything on its own.** `http::app::build` returns a `Router`
and does not listen or read the environment; `http::serve` owns the port. The binaries are the
only code that reads the process environment, and `MCP_AUTH_TOKEN` is read once, in `run`, when
the HTTP transport is chosen. The stdio path never reads it.

## Entry point and transports

`config::from_process_env` reads the configuration, and `run::run` serves either way:

* **stdio** (default): one `SecurityServer` on rmcp's stdio transport, on a single-threaded
  runtime. stdout is the JSON-RPC channel, so nothing else on this path writes to it. A pipe
  closed before the handshake exits 0.
* **streamable HTTP**: an axum application exposing `GET /healthz` and `POST /mcp`, served by
  one listener on a tokio runtime.

The HTTP transport is **stateless**: rmcp's `StreamableHttpService` runs with session mode off,
JSON responses on, and `NeverSessionManager`, building a fresh `SecurityServer` per request. The
handler holds only a reference to the immutable surface, so a fresh one costs nothing and shares
nothing. A bare `tools/list` with no handshake is answered.

The interface is named rather than defaulted. `HOST` names it, and its `0.0.0.0` default says on
the startup line that the port is open on every interface.

## The order a request is checked in

`http::app::build` wraps the whole route table as one service and layers three checks around it,
in the order a request meets them:

1. **`Host` allow-list** (`host.rs`), on every route, `/healthz` included. `MCP_ALLOWED_HOSTS`
   when set; the localhost names when the server runs on loopback without a token; otherwise
   none, and the startup line says so. A refusal is a 403 naming the host. The port is not part
   of the match.
2. **Bearer token** (`auth.rs`), on everything except exactly `GET /healthz`. A refusal is a 401
   with `WWW-Authenticate`; it gives no route information.
3. **Body** (`body.rs`), on `POST /mcp` only: at most 4 MB, and it must parse as JSON. An
   oversized body and a malformed one get the same 400 / `-32700`, as they always have.

The checks wrap the route table, not each route, on purpose. A layer added per route runs inside
axum's method dispatch, which adds `Allow:` to a refused response and would tell a caller without
the token which methods `/mcp` takes. rmcp's own `Host` check is turned off, so there is one list,
not two that could disagree.

Then the routes: `GET /healthz` returns `{ status, server, version }`; `POST /mcp` is the MCP
endpoint; any other method on `/mcp` is a 405; any other path is a JSON-RPC 404. No response
carries `X-Powered-By` or `Server`.

## Worker threads

On HTTP, `MCP_CLUSTER_WORKERS` sets how many tokio worker threads serve the one listener
(default: the CPUs the process was given). `0` or `1` runs a single-threaded runtime. There are no
child processes, so nothing is forked, respawned, or left holding the port if the process is
killed. The Node server needed `cluster` processes to use more than one core; a tokio runtime
does not.

### Shutdown drains before it closes

`SIGINT` and `SIGTERM` stop the listener accepting, let in-flight requests finish, and cut off
whatever is still running after five seconds. The process prints one `{signal}, draining` line
and exits 0. A second signal during the drain exits at once: the operator has stopped waiting.

## The tool layer

There is no hand-written tool. `build.rs` embeds every `.md` under `content/`, and
`instructions::build_surface` turns the table into one `Instruction` per file, once:

```rust
for (path, text) in ordered {
    let name = naming::tool_id_for(path);          // folder dropped, .md off, kebab → snake
    // refuse an invalid name, one over 64 characters, or a collision
    let description = frontmatter::parse(text)…;   // refuse a missing description
    surface.push(Instruction { name, description, path, text });
}
```

Paths are ordered case-insensitively, so the tool list is in the order the Node server listed it.
Four things fail **at startup** rather than at the first call, because a set that cannot be
routed on should stop the process and not mislead the caller: two files deriving one name, a name
that is not a valid MCP tool name, a name over 64 characters, and a file with no frontmatter
`description:`.

The file is served whole, frontmatter included, byte-identical to the set. Every tool declares
the input schema `{ "type": "object", "properties": {} }`, and `call_tool` is a lookup by name, so
there is no argument for a caller to aim with and no filesystem I/O on the request path.

## Authentication

**HTTP on any bind but loopback requires a bearer token; stdio and loopback HTTP do not.** A
loopback server without a token gets the localhost `Host` list instead. No tool reads a
credential, and none takes one as an argument; the token belongs to the transport.

`run` checks the token before a runtime or a socket exists, so a missing or short one is a single
line naming the fix and exit code `1`, whatever `MCP_CLUSTER_WORKERS` says. `http::app::build`
refuses too, so an open server cannot come from forgetting an option. The comparison is
constant-time over SHA-256 digests (`subtle`), the token is trimmed of surrounding whitespace,
never logged or echoed, and there is no query-string form and no flag that turns the check off.

The token gates who may use a deployed instance, not the text: the set is public. It is one shared
value, so there is no per-client identity, and it is only as private as the connection it travels
on, so TLS belongs in front. The reasoning is in
[`.agents/memory/decisions/http-bearer-token.md`](../../.agents/memory/decisions/http-bearer-token.md).

## The parity guarantee

`instructions::surface()` is the only tool list. The CLI's `tools` command, stdio and HTTP all
read it. `tests/surface.rs` asserts that what the CLI prints matches what an MCP client receives
from `tools/list`, pins the bijection between the files in `content/` and the tools, and checks
every tool's text against the file on disk. `tests/http.rs` asserts that HTTP and stdio serve the
same tools with the same text.

## Related pages

* [`overview.md`](overview.md): what this project is.
* [`../environments/setup.md`](../environments/setup.md): running it.
