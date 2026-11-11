# 2.0.3

**Released:** 2026-11-11

You no longer need `MCP_AUTH_TOKEN` to run this on your own machine. HTTP bound to
loopback (`127.0.0.1`, `localhost` or `::1`) now starts without a token. Instead, it only
answers requests whose `Host` header is localhost, which stops a web page from reaching it
through DNS rebinding. If HTTP is bound to any other address, such as the default `0.0.0.0`,
a container, or a public host, it still refuses to start without a token. stdio still never
reads the token. A token you do set is enforced on every bind.

**Clients must:** nothing.

- **Server installs, including the Docker image:** nothing changes.
- **Local installs:** you can drop the token. `npm run start:http` now binds `127.0.0.1`, so if
  you used it to serve other machines, run `lxagents-security serve --http --host 0.0.0.0`
  with the token set instead.

## Added

- **`serve --host ADDR`** sets the bind address for one run.
- **`isLoopback()`** and **`config.local`** in `src/config.js`.
- **`createApp({ local })`**: with `local` set and no token, the app is built without the token
  check and with the SDK's `localhostHostValidation`.
- **Tests**: local HTTP with no token on 1 and 2 workers, the localhost-only `Host` guard, a
  token still enforced locally, a short token refused locally, and the loopback rule. The
  test count goes from 64 to 69.

## Changed

- **`tokenProblem(token, { required })`**: with `required: false`, no token is acceptable, but a
  token that is too short is still refused.
- **`npm run start:http`** binds `127.0.0.1`.
- **Startup line**: a local server without a token says
  `bound to loopback, so no bearer token is required; Host is limited to localhost`.
- **Docs**: the README, setup, env and architecture pages now describe local and server
  installs separately.
