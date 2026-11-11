---
name: memory-tasks-typescript-migration
description: Moving src/ and test/ to strict TypeScript compiled to dist/, with the latest dependencies and a clean npm audit. Tool IDs and SDK calls unchanged.
---

# Task: migrate to TypeScript

The owner asked for every Node.js repository in the organization to move to TypeScript, one
repository per thread. This record is this repository's part: update every dependency to its
latest version, add `typescript`, `@types/node`, `tsx` and `@types/express`, clear `npm audit`,
convert `src/` and `test/` to strict TypeScript compiled to `dist/`, and prove the stdio and
HTTP transports and the tool surface behave exactly as they did.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | The record | `plan/typescript-migration` | |
| 2 | Latest dependencies, the TypeScript devDependencies, `npm audit fix` | `build/dependencies` | |
| 3 | `tsconfig.json` (ES2022, NodeNext, strict, `src` to `dist`); every `.js` in `src/` and `test/` renamed to `.ts` and typed; `main`, `bin`, `start` point at `dist/`; `build`, `prepare`, `pretest` scripts; tests run through `tsx`; two-stage `Dockerfile` | `refactor/typescript` | |
| 4 | README, `wiki/`, `.agents/` docs and rules describe the build step and `dist/index.js` | `docs/typescript` | |
| 5 | Release `2.0.4`, a patch by the owner's standing choice | `release/2.0.4` | |

**What stays:** every tool ID, every description, the SDK methods (`server.tool`, the
Streamable HTTP and stdio transports), the HTTP routes and refusals, and the environment
variables. The logic of every module is unchanged; only types are added.

**What changes for a consumer:** the entry point moves from `src/index.js` to
`dist/index.js`, so a local MCP registration must be updated.

**Decision - `dist/` is built on install, not committed.** Installs are by `git clone` and
`npm install`. A `prepare` script runs `tsc` during `npm install`, so a fresh clone has a
working `dist/` with no extra step, and `dist/` is in `.gitignore` so compiled output can
never drift from its source in history. The Docker image builds in a first stage and copies
`dist/` into the runtime stage.

## Progress

### Task 1 - plan/typescript-migration

Landed: this record and its row in `memory-index.md`. Also removed a leftover merge-conflict
marker from `memory-index.md` that had reached master.

### Task 2 - build/dependencies

Landed. `@modelcontextprotocol/sdk` 1.30.0 to 1.32.2, `express` 5.2.1 to 5.3.0, `zod` 4.5.4 to
4.6.7. New devDependencies: `typescript` 7.0.2, `@types/node` 26.6.6, `tsx` 4.23.15,
`@types/express` 5.0.6. `npm audit` reported 9 vulnerabilities on master (1 critical, 4 high,
4 moderate), all transitive (`fast-uri`, `hono`, `ip-address` among them); the updates and
`npm audit fix` leave 0. `npm outdated` is empty. Still JavaScript: 69 tests pass unchanged.
