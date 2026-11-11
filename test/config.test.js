import assert from "node:assert/strict";
import { availableParallelism } from "node:os";
import { test } from "node:test";
import { loadConfig } from "../src/config.js";

test("an empty environment gives the documented defaults", () => {
  assert.deepEqual({ ...loadConfig({}) }, {
    transport: "stdio",
    port: 3000,
    host: "0.0.0.0",
    workers: availableParallelism(),
    allowedHosts: [],
  });
});

test("every variable is read, and the transport is case-insensitive", () => {
  const config = loadConfig({
    MCP_TRANSPORT: "HTTP",
    PORT: "8080",
    HOST: "127.0.0.1",
    MCP_CLUSTER_WORKERS: "1",
    MCP_ALLOWED_HOSTS: " a.example , ,b.example,",
  });
  assert.equal(config.transport, "http");
  assert.equal(config.port, 8080);
  assert.equal(config.host, "127.0.0.1");
  assert.equal(config.workers, 1);
  assert.deepEqual([...config.allowedHosts], ["a.example", "b.example"]);
});

test("an unusable worker count falls back to one per CPU", () => {
  for (const value of ["", "-1", "many"]) {
    assert.equal(loadConfig({ MCP_CLUSTER_WORKERS: value }).workers, availableParallelism());
  }
});

test("the configuration cannot be changed after it is read", () => {
  const config = loadConfig({});
  assert.ok(Object.isFrozen(config));
  assert.ok(Object.isFrozen(config.allowedHosts));
});
