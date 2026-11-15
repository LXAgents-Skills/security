//! The CLI: list the tools, print the version, or serve.
//!
//! The tool list is the server's own surface, so the two cannot drift apart.

use std::{collections::HashMap, process::ExitCode};

use lxagents_security::{
    config,
    identity::{SERVER_ID, VERSION},
    instructions, run,
};

const HELP: &str = "lxagents-security - the global security instruction set, served read-only

Usage
  lxagents-security

Commands
  serve            Run as a Skill MCP server
                   [--stdio | --http] [--port N] [--host ADDR]
  tools            List the tools this server exposes

Options
  -h, --help       Show this help
  -v, --version    Show the version

Environment
  MCP_TRANSPORT    stdio (default) or http
  MCP_AUTH_TOKEN   bearer token, 32+ characters; required for http unless HOST is
                   loopback (127.0.0.1, localhost, ::1); unused by stdio
  PORT             HTTP port, default 3000
  HOST             HTTP interface, default 0.0.0.0
  MCP_ALLOWED_HOSTS  comma-separated Host allow-list; unset means none is applied
  MCP_CLUSTER_WORKERS  HTTP worker threads, default one per CPU; 1 runs a single thread";

/// The value after `flag` in `args`, if there is one.
fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a String> {
    let at = args.iter().position(|arg| arg == flag)?;
    args.get(at + 1)
}

/// `serve`: the flags override the environment for this run only.
fn serve(args: &[String]) -> ExitCode {
    let mut env: HashMap<String, String> = std::env::vars().collect();
    if args.iter().any(|arg| arg == "--http") {
        env.insert("MCP_TRANSPORT".into(), "http".into());
    }
    if args.iter().any(|arg| arg == "--stdio") {
        env.insert("MCP_TRANSPORT".into(), "stdio".into());
    }
    if let Some(port) = flag_value(args, "--port") {
        env.insert("PORT".into(), port.clone());
    }
    if let Some(host) = flag_value(args, "--host") {
        env.insert("HOST".into(), host.clone());
    }
    match config::load(&env) {
        Ok(config) => run::run(&config),
        Err(error) => {
            eprintln!("{SERVER_ID} {error}");
            ExitCode::FAILURE
        }
    }
}

/// `tools`: one line per tool, the names padded to one column.
fn tools() -> ExitCode {
    let surface = match instructions::surface() {
        Ok(surface) => surface,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let width = surface
        .iter()
        .map(|tool| tool.name.len())
        .max()
        .unwrap_or(0);
    for tool in surface {
        println!("{:width$}  {}", tool.name, tool.description);
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (command, rest) = match args.split_first() {
        Some((command, rest)) => (Some(command.as_str()), rest),
        None => (None, &[][..]),
    };

    match command {
        Some("-v" | "--version") => {
            println!("{VERSION}");
            ExitCode::SUCCESS
        }
        None => {
            println!("{HELP}");
            ExitCode::from(2)
        }
        Some("-h" | "--help" | "help") => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("serve") => serve(rest),
        Some("tools") => tools(),
        Some(other) => {
            eprintln!("Unknown command \"{other}\". Try --help.");
            ExitCode::from(2)
        }
    }
}
