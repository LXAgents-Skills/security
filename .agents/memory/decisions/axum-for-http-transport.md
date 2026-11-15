---
name: memory-decisions-axum-for-http-transport
description: Why the HTTP transport is axum around rmcp's StreamableHttpService, why the checks wrap the whole route table, and why MCP_CLUSTER_WORKERS now counts threads. Supersedes the express decision.
---

# Decision - axum for the HTTP transport

## 2026-11-15

**Context.** The owner asked for every repository to move from TypeScript to Rust, on the
official MCP Rust SDK (`rmcp`) with `axum` as the web framework. This record replaces
[`express-for-http-transport.md`](express-for-http-transport.md) for the current code; that
record stays as the history of the Node.js server.

**Decision.**

* `POST /mcp` is rmcp's `StreamableHttpService`, stateless (`legacy_session_mode` off,
  `NeverSessionManager`) with JSON responses, a fresh handler per request. Every tool answers in
  one message, so there is nothing to stream.
* The `Host` allow-list, the bearer token and the 4 MB JSON body check are axum middleware,
  written here, in that order. They wrap the route table as a single `fallback_service`, not
  each route: axum's per-route layers run inside method dispatch, which added `Allow: POST` to a
  401 for `GET /mcp` and told a caller without the token which methods the path takes.
* rmcp's own `Host` check is disabled. The router's layer is the one list, so the two cannot
  disagree, and the layer also covers `/healthz`, which rmcp never sees.
* The body is read and parsed by our layer before rmcp, so an oversized body and a malformed
  one keep their single 400 / `-32700` answer, and an unauthenticated caller never makes the
  server read one.
* `MCP_CLUSTER_WORKERS` sets tokio worker threads. Node's `cluster` was there to use more than
  one core; a tokio runtime does that in one process, so forking, respawning and the orphaned
  worker problem are gone. The variable keeps its name so existing deployments keep working.

**Consequences.** The HTTP behaviour a client sees is unchanged, and `tests/http.rs` pins it.
The startup log changes where it described processes: one `serving over http` line, and a line
naming the thread count.
