import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));

/** The repository root, resolved from this file rather than from cwd. */
export const ROOT: string = join(here, "..");

/**
 * The published set. Every served file is read from inside this directory and
 * nowhere else, so containment is a property of the constant rather than of
 * what a caller passed in.
 */
export const CONTENT_DIR: string = join(ROOT, "content");

const PACKAGE_JSON = join(ROOT, "package.json");

let resolved = "0.0.0";
try {
  const manifest = JSON.parse(await readFile(PACKAGE_JSON, "utf8")) as { version?: string };
  resolved = manifest.version ?? resolved;
} catch {
  // Keep the bundled server usable if package.json cannot be read.
}

export const version: string = resolved;
