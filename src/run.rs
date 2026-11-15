//! Choose the transport, build the runtime it needs, and start it.
//!
//! Nothing here writes to stdout: on stdio, stdout is the JSON-RPC channel.

use std::process::ExitCode;

use crate::{
    config::{Config, Transport},
    http,
    identity::{SERVER_ID, VERSION},
    instructions, stdio,
};

/// The runtime for HTTP: one thread when `workers` is 0 or 1, otherwise `workers` threads
/// sharing one listener. Node needed forked processes to use more than one core; tokio does
/// not, so there is no child process to respawn or to outlive its parent.
fn http_runtime(workers: usize) -> std::io::Result<tokio::runtime::Runtime> {
    if workers <= 1 {
        eprintln!(
            "{SERVER_ID} {VERSION} MCP_CLUSTER_WORKERS is {workers}, so a single thread serves every request."
        );
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
    }
    eprintln!("{SERVER_ID} {VERSION} serving HTTP from {workers} worker threads");
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()
}

/// Start what `config` names, and return the process's exit code.
pub fn run(config: &Config) -> ExitCode {
    let surface = match instructions::surface() {
        Ok(surface) => surface,
        Err(error) => {
            eprintln!("{SERVER_ID} {VERSION} {error}");
            return ExitCode::FAILURE;
        }
    };

    match config.transport {
        Transport::Stdio => run_stdio(surface),
        Transport::Http => run_http(config, surface),
    }
}

fn run_stdio(surface: &'static [instructions::Instruction]) -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("{SERVER_ID} {VERSION} cannot start: {error}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(stdio::serve(surface)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{SERVER_ID} {VERSION} stdio stopped: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_http(config: &Config, surface: &'static [instructions::Instruction]) -> ExitCode {
    // Fail closed, before a runtime or a socket exists, and say so exactly once.
    let token = http::auth::configured_token();
    if let Err(message) = http::preflight(config, &token) {
        eprintln!("{message}");
        return ExitCode::FAILURE;
    }
    match http_runtime(config.workers) {
        Ok(runtime) => runtime.block_on(http::serve(config, surface, token)),
        Err(error) => {
            eprintln!("{SERVER_ID} {VERSION} cannot start: {error}");
            ExitCode::FAILURE
        }
    }
}
