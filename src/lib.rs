//! `lxagents-security`: the global security instruction set, served read-only as a Skill MCP
//! server.
//!
//! The two binaries in `src/bin/` are the only things that start anything. Every module here
//! is inert until called, so all of it can be tested without binding a port or reading stdin.

pub mod config;
pub mod http;
pub mod identity;
pub mod instructions;
pub mod run;
pub mod server;
pub mod stdio;
