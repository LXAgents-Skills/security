# 2.0.1

**Released:** 2026-11-11

The repository moves to the LXAgents-Skills organization and the package to
`@lxagents-skills/security`. The code inside is split by responsibility, and `--help` now names
the token that HTTP needs. **The instruction surface, the HTTP routes, every environment variable,
both bins and every log line's wording are unchanged.**

**Clients must:**

1. **If you install from npm, install `@lxagents-skills/security`.** The old name,
   `@lxagents-mcp/security`, gets no further releases. A `.mcp.json` that runs
   `npx -y @lxagents-mcp/security` should name the new package.
2. **If you import from inside the package**, `src/app.js` and `src/auth.js` are now
   `src/http/app.js` and `src/http/auth.js`, and `allowedHosts()` is gone: pass `allowedHosts` to
   `createApp` or use `loadConfig()` from `src/config.js`. Running the bins or `src/index.js`
   needs no change.

**If you spawn `node src/index.js`, run the bins, or run the image, you need to do nothing**
apart from the new image tag.

## Added

- **`src/config.js`**: `loadConfig()` reads `MCP_TRANSPORT`, `PORT`, `HOST`,
  `MCP_CLUSTER_WORKERS` and `MCP_ALLOWED_HOSTS` once, into a frozen object.
- **`src/run.js`**, **`src/stdio.js`**, **`src/http/primary.js`**, **`src/http/worker.js`**: the
  transport choice, stdio, the cluster primary and the HTTP worker, each taken out of
  `src/index.js`. A stdio server no longer loads express.
- **`src/http/rpc-error.js`**: the one JSON-RPC error envelope. The 401 from the token check now
  goes through it too.
- **Tests**: `test/config.test.js`, plus a test that fails when `--help` leaves out any variable
  `src/` reads. 58 → 63, all passing.

## Changed

- **Organization and package**: `LXAgents-MCP` → `LXAgents-Skills`, `@lxagents-mcp/security` →
  `@lxagents-skills/security`, in `package.json`, the lockfile, README, LICENSE and the docs.
- **Vocabulary**: `src/tools/` → `src/instructions/`; `TOOL_MODULES`, `CONTENT_TOOLS`,
  `TOOL_FILES` → `INSTRUCTION_MODULES`, `CONTENT_INSTRUCTIONS`, `INSTRUCTION_FILES`. Prose calls
  this a Skill MCP server. The protocol surface (`server.tool`, `listTools()`, `tools/list`) is
  unchanged.
- **`src/app.js`, `src/auth.js`** → `src/http/app.js`, `src/http/auth.js`. `createApp` takes
  `allowedHosts` as an option instead of reading the environment.
- **`src/cli.js serve`** passes its flags to `loadConfig` instead of writing `process.env`.
- **Image tag** in the docs and Dockerfile comments: `2.0.1`.

## Removed

- **`allowedHosts()`** export from the app module. `loadConfig().allowedHosts` replaces it.

## Fixed

- **`--help`** now lists `MCP_AUTH_TOKEN`, the one variable HTTP cannot start without.
- A dead reference to `.agents/memory/tasks/activation-security.md`, in `src/http/app.js` and the
  express decision.
