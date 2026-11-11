# Local Setup

`@lxagents-skills/security` is **dual-purpose**. The same code is reachable two ways:

| Mode | What it is | Who uses it |
|---|---|---|
| **CLI mode** | A terminal command | A person running it by hand or from a script |
| **Server mode** | A Skill MCP server over stdio or streamable HTTP | An MCP client, an editor, an agent, or a connector |

Both modes share one implementation, so a result produced in one is identical to the
same result produced in the other.

## Requirements

Node.js 20 or newer.

```bash
npm install       # installs, then compiles src/ to dist/ (the prepare script)
npm test          # builds, then runs the suite through tsx
```

`npm run build` recompiles after a change to `src/`; the server always runs from `dist/`.
`npm run typecheck` checks `src/` and `test/` together without writing anything.

Three runtime dependencies: `@modelcontextprotocol/sdk`, `express` and `zod`. TypeScript,
`tsx` and the type packages are devDependencies, needed to build and test only.

An existing local registration that points at `src/index.js` must be changed to
`dist/index.js`: the source is TypeScript now, and Node runs the compiled output.

There is also a container image, for a host that cannot run Node at all — see
[`docker.md`](docker.md). It is not built by `npm test` and has never been built.

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
# From a checkout, for development
npm install
npm link

# Or globally, from the registry
npm install -g @lxagents-skills/security
```

Without installing anything:

```bash
node dist/cli.js --help
npm run cli -- --help
```

### Use

```bash
lxagents-security --help
lxagents-security --version
lxagents-security tools
```

`tools` prints every registered tool with its description:

```text
golang_general_backend_security                    Go (Golang) Security Spec (Go 1.25.x, Standard Library, net/http) — …
javascript_express_web_server_security             Express (Node.js) Web Security Spec (Express 5.x / 4.19.2+, Node.js LTS) — …
…
skill                                              Perform language and framework specific security best-practice reviews …
```

One line per file in `content/`, named after that file. Eleven lines today; one more
whichever guide is added to the set next.

The list comes from `listTools()` in `src/server.ts` — the same list the Skill MCP server
registers — so the two surfaces cannot disagree.

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
      "command": "node",
      "args": ["dist/index.js"],
      "cwd": "/path/to/security"
    }
  }
}
```

Once the package is installed globally, the bin can be named directly instead:

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
npm start
lxagents-security serve --stdio

# streamable HTTP, local - binds 127.0.0.1, needs no token
npm run start:http
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
npm run inspect
```

This runs the MCP Inspector against the stdio server, listing every tool and letting
you call them.

### stdout belongs to the protocol

On the stdio transport, stdout **is** the JSON-RPC channel. Logging goes to stderr,
and `serve` prints nothing of its own. Only CLI commands write to stdout.

A `console.log` on the server path is a bug that corrupts the protocol stream.

## Related pages

- [`env.md`](env.md) — every environment variable this project reads
- [`docker.md`](docker.md) — building and running the container image
- [`../information/overview.md`](../information/overview.md) — what this project is
- [`../information/architecture.md`](../information/architecture.md) — how the pieces fit
- [`README.md`](../../README.md)
