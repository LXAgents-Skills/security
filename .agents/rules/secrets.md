---
name: secrets
description: The contract for a tool that needs a credential - read it from the environment inside the handler, fail descriptively, and never log or echo it.
---

# Secrets

**No tool in this repository reads a credential.** Nothing here reaches an external
service, so there is no key and no `API_KEY` variable. This file is the contract to follow
*if* a future tool needs one, and the record of why none does today.

The key would be **one for the whole server**, not one per tool, read from the
`API_KEY` environment variable with `std::env::var`.

## Read it inside the handler, at call time

```rust
fn call_example(&self) -> Result<CallToolResult, ErrorData> {
    let api_key = std::env::var("API_KEY").ok().filter(|key| !key.trim().is_empty());
    let Some(api_key) = api_key else {
        return Err(ErrorData::internal_error(
            "<tool_name> requires an API key. Set the API_KEY environment variable \
             before starting the server.",
            None,
        ));
    };
    // ...
}
```

Never in a `static`, a `LazyLock`, or `Config`. A key captured at startup is the value that happened to be
set when the process first read it, which breaks three things at once: a process that
sets the key after startup, a test that sets and unsets it around a case, and an HTTP
server that builds a fresh handler per request but keeps every `static`.

## Registration never depends on the key

Every tool is registered whether or not a key is present. A server that hides its
authenticated tools when the key is missing reports "no such tool" to the caller -
which is indistinguishable from the tool not existing, and sends whoever is debugging
it looking in the wrong place entirely.

The tool is always listed; calling it without a key is what fails, and it fails saying
so.

## The failure must be descriptive

A missing key returns an `ErrorData` naming the tool, the variable, and the fix. Not
`"unauthorized"`, not `"missing config"`, and never a silent empty result.

## Never disclose the key

* Do not log it, at any level, on any transport.
* Do not include it in tool output, an error message, or a health check response.
* Do not write it into a test fixture, a wiki page, or a commit.
* `.env` is gitignored and stays that way.

A test that needs a key passes an obvious dummy to the spawned binary with
`Command::env("API_KEY", ...)`, never `std::env::set_var` in the test process (it is
`unsafe` in the 2024 edition and leaks into tests running in parallel).
