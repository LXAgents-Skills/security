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

### Task 3 - refactor/typescript

Landed. `tsconfig.json` targets ES2022 with `NodeNext` module and resolution, `strict`, `src`
to `dist`, and `types: ["node"]` (TypeScript 7 no longer loads every `@types` package by
default). `test/tsconfig.json` extends it with `noEmit` so `npm run typecheck` checks the tests
too. Every file in `src/` and `test/` is `.ts`; imports keep their `.js` extension, as
`NodeNext` requires. New types: `Config` (`config.ts`), `ToolSummary` (`server.ts`),
`InstructionConfig` and `InstructionModule` (`from-content.ts`), `AppOptions` (`app.ts`). No
logic changed, and no `any` was written.

`package.json`: `main` and both `bin` entries point at `dist/`, `files` ships `dist` rather
than `src`, and the scripts are `build` and `prepare` (`tsc`), `typecheck`, `start`,
`start:http`, `cli` and `inspect` on `dist/`, `pretest` (build), and
`test` = `node --import tsx --test test/*.test.ts`. The tests import the source through `tsx`
and spawn the compiled `dist/index.js` and `dist/cli.js`, so the stdio and HTTP tests exercise
what a clone actually runs. The CLI help test now scans `src/**/*.ts` for `env.` reads.
`dist/` is in `.gitignore` and `.dockerignore`.

The `Dockerfile` has two stages: `build` runs `npm ci --ignore-scripts` and `npm run build`; the
runtime stage installs `--omit=dev`, copies `dist/` and `content/`, and runs
`node dist/index.js`.

Verified: typecheck clean; 69 of 69 tests pass; `tools` output is byte-identical to master
(12 tools); a fresh `git clone` and `npm install` produces `dist/` through `prepare` and
serves 12 tools; the runtime stage simulated outside Docker (production install, no compiler)
answers `/healthz` and `tools/list` over stdio. `docker build` itself could not be run in the
session's sandbox: its build containers have no route to the npm registry.

### Task 4 - docs/typescript

Landed. README, `AGENTS.md`, `wiki/information/` (overview, architecture) and
`wiki/environments/` (setup, env, docker) describe TypeScript in `src/`, the `prepare` build
into `dist/`, the new scripts, the two-stage image, and the registration path
`dist/index.js`; setup says an existing `src/index.js` registration must change.
`.agents/rules/repository.md` replaces "must not introduce a build step" with "no committed
`dist/`, no second build tool, no `any`", which the owner's request for a TypeScript build
overturned. `repository-map.md`, `tool-authoring.md` and `repository-state.md` name the `.ts`
files. Release logs and older task records keep their `.js` names, because they describe what
was true then.

### Task 5 - release/2.0.4

Landed. The version is `2.0.4`, a patch bump, which is the owner's standing choice. The
changelog in `wiki/logs/2/0/4/` says the one thing clients must do: point a local
registration at `dist/index.js`. It also has a row in `logs-index.md`, and the documented
image tag and startup lines are now 2.0.4. Final check on this branch: a clean `npm run build`
and `npm run typecheck`, 69 of 69 tests pass, and `npm audit` reports 0 vulnerabilities.
