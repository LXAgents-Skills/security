---
name: repository-rules
description: Rules specific to lxagents-security - the dual-surface contract, the stdout ban, the derived read-only surface, and what must not be introduced.
---

# Repository Rules

`lxagents-security` is a dual-purpose Skill MCP server and CLI over one implementation, and
it serves the global security set from `content/` read-only. Both facts constrain what
may be changed here.

## Mode and shared set

This repository is a **Mode B consumer**. The shared instruction set is resolved
through the `lxagents-agents-base` Skill MCP connector and is never copied into this tree.
See the bootstrap block in [`../../AGENTS.md`](../../AGENTS.md).

## The two surfaces stay in step

The CLI (`lxagents-security`) and the Skill MCP server (`lxagents-security-server`) are two
doors onto one implementation. A tool reachable from one is reachable from the other,
with the same name and the same description.

* The tool surface is built in exactly one place: `src/instructions/from-content.ts`, from the
  files in `content/`.
* `src/server.ts` registers that array and exports `listTools()`; it does not declare
  tools of its own.
* `src/cli.ts` never maintains its own list - it reads the declaration from
  `src/server.ts`.
* `test/server.test.ts` pins the agreement. A change that makes the two surfaces
  disagree fails the suite, and that failure is the point.

## Nothing writes to stdout except the CLI

On the stdio transport, stdout **is** the JSON-RPC channel. A stray `console.log` on
the server path corrupts the protocol stream and the client reports a parse error
that names nothing useful.

* Server-side logging goes to stderr.
* `serve` prints nothing of its own.
* Only CLI commands write to stdout.

## Where things go

| Thing | Path |
|---|---|
| The served security set | `content/` |
| The tool surface, derived from the set | `src/instructions/from-content.ts` |
| Tool registration and `listTools()` | `src/server.ts` |
| `ROOT`, `CONTENT_DIR`, and the version | `src/version.ts` |
| CLI commands | `src/cli.ts` |
| Compiled output, never committed | `dist/`, from `tsconfig.json` |
| Transport and entry point | `src/index.ts` → `src/run.ts` → `src/stdio.ts` or `src/http/` |
| Tests | `test/{subject}.test.ts` |

## Commands

```bash
npm install       # Node 20+; compiles src/ to dist/ through prepare
npm run build     # tsc, after a change to src/
npm run typecheck # src/ and test/, no output
npm test          # builds, then node --import tsx --test test/*.test.ts
npm run cli -- tools
npm start         # stdio
npm run start:http
npm run inspect   # MCP Inspector against the stdio server
```

## What must not be introduced

* Committed build output. `dist/` is compiled from `src/` by `npm install` and `npm test`
  and is ignored by git; a checked-in copy would drift from its source.
* A second build tool. `tsc` is the build; there is no bundler, and the compiled files
  keep the one-module-per-file shape of `src/`.
* An `any`. `strict` is on, and a type that cannot be named yet is `unknown`, narrowed
  where it is read.
* A second source of truth for the tool list, and a hand-written tool module beside the
  generator - see [`tool-authoring.md`](tool-authoring.md).
* Shared instruction content. If it can be read from `agents://`, it must not exist
  here as a file.
* A write path, and an argument. No tool may take a verb, a credential, or reach a
  network; no tool takes any argument at all. The read-only property is structural - the
  code that would write is absent - and it is the property a consuming repository depends
  on when it points at this server.
* An edit to a guide under `content/`. Those files are copied from the upstream
  workspace set; the change belongs there, and this repository follows it.
