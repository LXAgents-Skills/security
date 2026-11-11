#!/usr/bin/env node

import { listTools } from "./server.js";
import { version } from "./version.js";

const HELP = `lxagents-security - the global security instruction set, served read-only

Usage
  lxagents-security

Commands
  serve            Run as a Skill MCP server
  tools            List the tools this server exposes

Options
  -h, --help       Show this help
  -v, --version    Show the version

Environment
  MCP_TRANSPORT    stdio (default) or http
  MCP_AUTH_TOKEN   bearer token, 32+ characters; required for http, unused by stdio
  PORT             HTTP port, default 3000
  HOST             HTTP interface, default 0.0.0.0
  MCP_ALLOWED_HOSTS  comma-separated Host allow-list; unset means none is applied
  MCP_CLUSTER_WORKERS  HTTP worker count, default one per CPU; 1 forks nothing`;

const [command, ...rest] = process.argv.slice(2);

if (command === "-v" || command === "--version") {
  process.stdout.write(`${version}\n`);
} else if (!command || command === "-h" || command === "--help" || command === "help") {
  process.stdout.write(`${HELP}\n`);
  process.exitCode = command ? 0 : 2;
} else if (command === "serve") {
  // Flags override the environment for this run. They are not written back to
  // process.env: a forked HTTP worker re-runs this file with the same arguments, so it
  // reaches the same configuration on its own.
  const overrides = {};
  if (rest.includes("--http")) overrides.MCP_TRANSPORT = "http";
  if (rest.includes("--stdio")) overrides.MCP_TRANSPORT = "stdio";

  const portFlag = rest.indexOf("--port");
  if (portFlag !== -1 && rest[portFlag + 1]) overrides.PORT = rest[portFlag + 1];

  const [{ loadConfig }, { run }] = await Promise.all([import("./config.js"), import("./run.js")]);
  await run(loadConfig({ ...process.env, ...overrides }));
} else if (command === "tools") {
  const tools = listTools();
  const width = Math.max(0, ...tools.map((tool) => tool.name.length));

  for (const tool of tools) {
    process.stdout.write(`${tool.name.padEnd(width)}  ${tool.description}\n`);
  }
} else {
  process.stderr.write(`Unknown command "${command}". Try --help.\n`);
  process.exitCode = 2;
}
