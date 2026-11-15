# Environment Variables

Six variables. **Five are optional, and stdio needs none of them.** The sixth,
`MCP_AUTH_TOKEN`, is a secret and is **required by the HTTP transport on any bind but
loopback**: without it a server install does not start, and a local one bound to `127.0.0.1`
runs without it.

| Variable | Default | Read by | Effect |
|---|---|---|---|
| `MCP_TRANSPORT` | `stdio` | `src/config.rs` | `stdio` or `http` (`streamable-http` is accepted too). `http` is **Streamable HTTP on `/mcp`** — it is the only HTTP transport this server has. |
| `PORT` | `3000` | `src/config.rs` | The port the HTTP transport listens on. Ignored on stdio. |
| `HOST` | `0.0.0.0` | `src/config.rs` | The interface the HTTP transport binds. Ignored on stdio. A loopback value (`127.0.0.1`, `localhost`, `::1`) makes the install local: no token required. `--host` overrides it. |
| `MCP_ALLOWED_HOSTS` | *unset* | `src/config.rs` | Comma-separated `Host` allow-list. **Unset means no allow-list is applied.** Read on the HTTP transport only. |
| `MCP_AUTH_TOKEN` | **none - required for HTTP off loopback** | `src/http/auth.rs` | The bearer token every HTTP request except `GET /healthz` must carry. At least 32 characters. **HTTP bound to anything but loopback will not start without it.** Optional on loopback, where it is still enforced if set. stdio never reads it. |
| `MCP_CLUSTER_WORKERS` | *the CPU count* | `src/config.rs` | How many tokio worker threads serve HTTP. **`0` or `1` means a single thread.** Always one process and one listener. Read on the HTTP transport only. |

`0.0.0.0` is every IPv4 interface, not the dual-stack `::`, so a client reaching the server
over IPv6 needs `HOST=::`. An unusable `PORT`, such as `http` or `70000`, stops the server
with a line naming the variable.

## `MCP_ALLOWED_HOSTS`

A comma-separated list of hostnames the `Host` header may carry. Matching is
**port-agnostic** — `security.example.test` allows `security.example.test:8080` — and an
IPv6 literal is written in brackets. A request outside the list is refused with a 403
and a JSON-RPC error naming the host that was rejected.

**An empty value means the guard is off.** Not "allow nothing". Set to nothing, or to
nothing but commas and spaces, the variable reads as unset: no list is applied and every
request is served. The server says so on startup, and says it only when it is true.

```text
lxagents-security 2.0.4 serving over http on :3000/mcp (all interfaces)
lxagents-security 2.0.4 bearer token required on every route except GET /healthz.
lxagents-security 2.0.4 MCP_ALLOWED_HOSTS is unset, so no Host header allow-list is applied.
```

Set it before exposing the port anywhere but your own machine. The server applies the
localhost list on its own only when it runs on loopback without a token; `HOST` defaults to
`0.0.0.0`, so without an explicit list nothing is filtering the `Host` header in exactly the
deployment — a container, a shared host — where it would matter.

Unset is a skipped check rather than a guessed one, because a wrong allow-list silently
refusing every request is a worse failure than an absent one, and the set is public
markdown either way.

## `MCP_AUTH_TOKEN`

Where it listens decides. A server a client spawns is a pipe on the client's own machine, and a
server bound to loopback can only be reached from that machine too, so neither needs a token. A
loopback server without one answers only a `localhost`, `127.0.0.1` or `[::1]` `Host` header
(unless `MCP_ALLOWED_HOSTS` names others), so a web page that rebinds its own name to
`127.0.0.1` cannot read it through the browser. A server on any other address is reachable by
anyone who can open a socket, so every request must prove the caller holds the token. An optional token would mean
a deployment that forgot to set it ran open and said nothing, so instead the process refuses
to start: one line on stderr naming the variable, and exit code `1`. `http::app::build`
refuses to build without it too, so the check cannot be skipped by calling the app directly.

```bash
export MCP_AUTH_TOKEN="$(openssl rand -hex 32)"    # 64 hex characters; keep it, clients need it
lxagents-security serve --http --host 0.0.0.0
```

* **Length.** Under 32 characters is refused at startup, so a value like `test` is found by
  you and not by a scanner. Surrounding whitespace is dropped, because a token read out of a
  file or an `env_file` often ends in a newline and an HTTP header cannot.
* **Sending it.** `Authorization: Bearer <token>` on every request. A missing header and a
  wrong token are both a `401` with `WWW-Authenticate: Bearer`; the second also says
  `error="invalid_token"`. There is no query-string form, because a URL is logged.
* **One token, shared.** Every client holds the same value, so there is no per-client
  identity and no revoking one client alone. Rotate it by changing the variable and
  restarting; the server is stateless, so a restart drops nothing.
* **Never logged.** The startup line says a token is required and never what it is, and a
  rejected token is not echoed. Keep it out of the image, the repository and the command
  line: pass it from the environment, an `env_file`, or the host's secret store.
* **`GET /healthz` is the one open route**, so an orchestrator's probe needs no token. It
  returns `{ status, server, version }` and nothing else.
* **Put TLS in front.** A bearer token sent over plain `http` can be read by anyone on the
  path. This server does not terminate TLS.

The token gates the *service*, not the *text*: the set is public, so it does not become
confidential. No tool reads a credential, and none takes one as an argument.

If a future *tool* needs a credential, the contract for that is
[`../../../.agents/rules/secrets.md`](../../../.agents/rules/secrets.md): read the
variable **inside the handler**, never at startup, and never as a condition on
whether the tool is registered.

## `MCP_TRANSPORT`, `PORT` and `HOST`

```bash
# stdio (default)
lxagents-security-server

# streamable HTTP on 3000, local (no token needed on loopback)
lxagents-security serve --http --host 127.0.0.1

# streamable HTTP on another port, as a server (MCP_AUTH_TOKEN must be set)
lxagents-security serve --http --host 0.0.0.0 --port 8080

# the same, from the environment
MCP_TRANSPORT=http HOST=0.0.0.0 PORT=8080 lxagents-security-server
MCP_ALLOWED_HOSTS=security.example.com MCP_TRANSPORT=http lxagents-security-server
```

The CLI's `serve` command sets the variables from its flags for that run only, so `--http`,
`--stdio`, `--port` and `--host` are equivalent to exporting `MCP_TRANSPORT`, `PORT` and
`HOST`.

## `MCP_CLUSTER_WORKERS`

How many tokio worker threads serve the HTTP listener. There is one process and one listener
whatever the value; the runtime spreads connections across its threads.

Unset, the count is the CPUs this process was actually given, not a constant, so a two-CPU
container gets two threads and a laptop does not get eight.

```bash
# one thread per CPU (the default)
lxagents-security serve --http --host 127.0.0.1

# a single-threaded runtime
MCP_CLUSTER_WORKERS=1 lxagents-security serve --http --host 127.0.0.1

# four threads on a machine that reports two CPUs
MCP_CLUSTER_WORKERS=4 lxagents-security serve --http --host 127.0.0.1
```

The startup log names the choice: `MCP_CLUSTER_WORKERS is 1, so a single thread serves every
request.` or `serving HTTP from 4 worker threads`, followed by one `serving over http` line.

The name is kept from the Node.js server, where the value was a count of forked `cluster`
processes. Those processes, their respawning and their crash limit are gone: tokio uses every
core from one process, so there is nothing to fork or to orphan.

## Related pages

* [`setup.md`](setup.md) — installing and running both modes.
* [`../information/overview.md`](../information/overview.md) — what the project is.
