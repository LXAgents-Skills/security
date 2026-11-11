# 2.0.4

**Released:** 2026-11-11

The server is now written in TypeScript. The source in `src/` is compiled to `dist/`, and
Node runs `dist/`. `npm install` builds it for you through the `prepare` script, so the
install steps stay the same: `git clone`, then `npm install`. Every tool ID, every
description, both transports, the HTTP routes and every environment variable behave exactly as
they did in 2.0.3. All dependencies are on their latest versions, and `npm audit` reports 0
vulnerabilities, down from 9.

**Clients must:** point a local stdio registration at `dist/index.js` instead of
`src/index.js`. `src/index.js` no longer exists.

```json
{ "command": "node", "args": ["dist/index.js"], "cwd": "/path/to/security" }
```

- **A clone you update with `git pull`:** run `npm install` afterwards. That builds `dist/`.
- **The Docker image and the published package:** nothing changes for you. Both run
  `dist/index.js`.

## Added

- **`tsconfig.json`**: ES2022, `NodeNext`, `strict`, compiling `src/` to `dist/`.
  `test/tsconfig.json` extends it so the tests are type-checked too.
- **Scripts**: `build` (`tsc`), `prepare` (`tsc`, run by `npm install`), `typecheck`, and
  `pretest`, which builds before the suite runs.
- **devDependencies**: `typescript` 7.0.2, `@types/node` 26.6.6, `tsx` 4.23.15 and
  `@types/express` 5.0.6.
- **Types**: `Config`, `ToolSummary`, `InstructionConfig`, `InstructionModule` and
  `AppOptions`.

## Changed

- **`src/` and `test/`** are `.ts`. No logic changed.
- **`main`, both `bin` entries, `start`, `start:http`, `cli` and `inspect`** now point at
  `dist/`. The package ships `dist/` instead of `src/`.
- **`npm test`** runs `node --import tsx --test test/*.test.ts`. The stdio and HTTP tests
  spawn the compiled server. There are still 69 tests, and all of them pass.
- **Dependencies**: `@modelcontextprotocol/sdk` 1.32.2, `express` 5.3.0 and `zod` 4.6.7.
  Running `npm audit fix` cleared the transitive advisories, including the critical
  `fast-uri` one and the `hono` and `ip-address` ones.
- **Dockerfile**: there are now two stages. The build stage compiles with `tsc`. The
  runtime stage installs production dependencies only, copies `dist/` and `content/`, and runs
  `node dist/index.js`.
- **Rules**: "no build step" is replaced by three rules: never commit `dist/`, use no second
  build tool, and write no `any`.

## Removed

- **`src/*.js`**: the JavaScript sources. The compiled `dist/` is ignored by git, so it never
  appears in the repository.
