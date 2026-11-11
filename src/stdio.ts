import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { SERVER_ID, createServer } from "./server.js";
import { version } from "./version.js";

/**
 * Serve over stdio.
 *
 * stdio never forks. stdout is the JSON-RPC channel here, and a worker's copy of it
 * would corrupt the stream. Nothing on this path reads `MCP_AUTH_TOKEN`: whoever can
 * use the pipe already runs the process.
 */
export async function startStdio(): Promise<void> {
  const server = createServer({ version });
  await server.connect(new StdioServerTransport());
  process.stderr.write(`${SERVER_ID} ${version} serving over stdio\n`);
}
