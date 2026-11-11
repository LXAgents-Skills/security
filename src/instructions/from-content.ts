import { readdirSync, readFileSync } from "node:fs";
import { join, relative, sep } from "node:path";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { CONTENT_DIR } from "../version.js";

/** What a tool is called and how a client routes to it. */
export interface InstructionConfig {
  name: string;
  description: string;
}

/** One served file: its tool declaration and the handler that returns it whole. */
export interface InstructionModule {
  config: InstructionConfig;
  handler: () => Promise<CallToolResult>;
}

/**
 * The prefix every tool ID carries: `{org_name}_{repo_name}`, underscores only.
 *
 * A client that connects several servers sees one flat tool list, so a bare name such
 * as `skill` or `automation` can collide with another server's. The prefix makes every
 * ID unique to this repository.
 */
export const ID_PREFIX: string = "lxagents_skills_security";

/**
 * The longest ID this repository publishes.
 *
 * `TOOL_NAME` admits 128 characters, which is what the protocol allows. Several clients
 * cap a tool name at 64, so every ID here is kept to 64, and a base name that would push
 * an ID past it is shortened in `NAME_OVERRIDES`. The test suite holds every ID to this.
 */
export const MAX_PUBLISHED_ID_LENGTH: number = 64;

/**
 * IDs that do not survive mechanical derivation, keyed by set-relative path.
 *
 * Everything else is named after its own filename, so adding a file to `content/`
 * adds the tool that serves it without anyone deciding what to call it. An override
 * carries the full ID, prefix included, and only the base name is ever shortened:
 * `javascript_typescript` becomes `js_ts` and `javascript` becomes `js`, so that four
 * guides fit in 64 characters. The owner chose that on 2026-11-11.
 *
 * `AGENTS.md` would derive to `agents`, which says nothing about which document it is;
 * no such file is in the set today, and the override is kept for when one is.
 */
const NAME_OVERRIDES: Readonly<Record<string, string>> = {
  "AGENTS.md": `${ID_PREFIX}_agents_entry_point`,
  "references/javascript-general-web-frontend-security.md": `${ID_PREFIX}_js_general_web_frontend_security`,
  "references/javascript-typescript-nextjs-web-server-security.md": `${ID_PREFIX}_js_ts_nextjs_web_server_security`,
  "references/javascript-typescript-react-web-frontend-security.md": `${ID_PREFIX}_js_ts_react_web_frontend_security`,
  "references/javascript-typescript-vue-web-frontend-security.md": `${ID_PREFIX}_js_ts_vue_web_frontend_security`,
};

/** The shape an MCP client will accept as a tool name: 128 characters at most. */
const TOOL_NAME = /^[a-z][a-z0-9_]{0,127}$/;

/**
 * Derive a tool ID from a set-relative path: `{ID_PREFIX}_{base}`.
 *
 * The folder is dropped, so `references/python-flask-web-server-security.md` is
 * `lxagents_skills_security_python_flask_web_server_security` rather than a qualified
 * name. The cost is that two folders holding the same filename would collide — which
 * `buildContentInstructions` treats as a startup error rather than letting the second
 * file silently shadow the first.
 */
function instructionNameFor(relativePath: string): string {
  const override = NAME_OVERRIDES[relativePath];
  if (override) return override;

  const base = (relativePath.split("/").pop() ?? relativePath).replace(/\.md$/, "").toLowerCase().replace(/-/g, "_");
  return `${ID_PREFIX}_${base}`;
}

/** Every `.md` under the set root, as set-relative forward-slash paths, sorted. */
function markdownFiles(dir: string = CONTENT_DIR, base: string = CONTENT_DIR): string[] {
  const found: string[] = [];

  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) found.push(...markdownFiles(full, base));
    else if (entry.name.endsWith(".md")) {
      found.push(relative(base, full).split(sep).join("/"));
    }
  }

  return found.sort((a, b) => a.localeCompare(b));
}

/**
 * Pull `name` and `description` out of a file's frontmatter.
 *
 * Only single-line scalar fields are read. A folded YAML value would need a real
 * parser, and the alternative is a dependency added to serve one call site, which
 * `.agents/rules/repository.md` forbids. A field this misses is caught below as a
 * missing description rather than silently serving an empty one.
 *
 * The value is matched as `\S.*` rather than `.*`, so the run of spaces in front of
 * it and the value behind it are different character classes. Two overlapping
 * greedy runs in a row are what let a line that fails to match be retried from
 * every length; with `\S` in front, handing a space back can never turn a failure
 * into a match, so a non-matching line costs one pass instead of one pass per space.
 */
function parseFrontmatter(text: string): Record<string, string> | null {
  const block = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (block?.[1] === undefined) return null;

  const fields: Record<string, string> = {};
  for (const line of block[1].split(/\r?\n/)) {
    const field = line.match(/^([A-Za-z_][A-Za-z0-9_]*):[ \t]*(\S.*|)$/);
    if (field?.[1] !== undefined) fields[field[1]] = (field[2] ?? "").trim();
  }
  return fields;
}

/**
 * Build the whole per-file tool surface, reading the set once.
 *
 * Everything is resolved here, at import, so a malformed set fails the process
 * rather than surfacing as a wrong answer to the first caller that needed the file.
 * A handler is then a map lookup: there is no path argument, so there is nothing for
 * a caller to traverse with, and there is no filesystem I/O on the read path.
 *
 * The file is served whole, frontmatter included, byte-identical to what the set
 * holds — the frontmatter is part of the published text, not metadata to strip.
 */
function buildContentInstructions(): { tools: InstructionModule[]; files: Map<string, string> } {
  const tools: InstructionModule[] = [];
  const files = new Map<string, string>();
  const claimed = new Map<string, string>();

  for (const path of markdownFiles()) {
    const name = instructionNameFor(path);

    if (!TOOL_NAME.test(name)) {
      throw new Error(
        `${path} derives the tool name "${name}", which is not a valid MCP tool name. Add it to NAME_OVERRIDES.`,
      );
    }
    if (claimed.has(name)) {
      throw new Error(
        `${path} and ${claimed.get(name)} both derive the tool name "${name}". One would silently shadow the other; add one to NAME_OVERRIDES.`,
      );
    }
    claimed.set(name, path);

    const text = readFileSync(join(CONTENT_DIR, path), "utf8");
    const frontmatter = parseFrontmatter(text);

    if (!frontmatter?.description) {
      throw new Error(
        `${path} has no frontmatter \`description:\`. It would publish as a tool a client cannot route on.`,
      );
    }

    files.set(name, path);
    tools.push({
      config: { name, description: frontmatter.description },
      handler: async () => ({ content: [{ type: "text", text }] }),
    });
  }

  if (tools.length === 0) {
    throw new Error(`No markdown files under ${CONTENT_DIR}. This server would expose nothing.`);
  }

  return { tools, files };
}

const { tools, files } = buildContentInstructions();

export const CONTENT_INSTRUCTIONS: readonly InstructionModule[] = Object.freeze(tools);

/** Tool name -> the set-relative file it serves. The test suite pins both directions. */
export const INSTRUCTION_FILES: ReadonlyMap<string, string> = files;

export default CONTENT_INSTRUCTIONS;
