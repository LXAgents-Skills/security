//! The HTTP transport: refuse to start without a usable token, then bind, announce which
//! controls are on, serve, and drain on a signal.

pub mod app;
pub mod auth;
pub mod body;
pub mod host;
pub mod rpc_error;

use std::{process::ExitCode, time::Duration};

use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use crate::{
    config::Config,
    identity::{SERVER_ID, VERSION},
    instructions::Instruction,
};

/// How long in-flight requests may run after a shutdown signal before they are cut off.
pub const DRAIN_GRACE: Duration = Duration::from_secs(5);

/// Check the token before anything is built or bound, and report it once, with the fix.
pub fn preflight(config: &Config, token: &str) -> Result<(), String> {
    match auth::token_problem(token, !config.local) {
        None => Ok(()),
        Some(problem) => Err(format!(
            "{SERVER_ID} {VERSION} will not start the HTTP transport: {problem}. Set MCP_AUTH_TOKEN to a secret of at least {} characters, for example the output of: openssl rand -hex 32. stdio, and HTTP bound to 127.0.0.1, need no token.",
            auth::MIN_TOKEN_LENGTH
        )),
    }
}

/// The address `host` binds: brackets around an IPv6 literal are dropped.
fn bind_host(host: &str) -> &str {
    host.strip_prefix('[')
        .and_then(|h| h.strip_suffix(']'))
        .unwrap_or(host)
}

/// The lines that say which control is on. None of them ever carries the token.
fn announce(config: &Config, token_set: bool) {
    let where_ = if config.host == "0.0.0.0" {
        "all interfaces"
    } else {
        config.host.as_str()
    };
    eprintln!(
        "{SERVER_ID} {VERSION} serving over http on :{}/mcp ({where_})",
        config.port
    );
    if token_set {
        eprintln!(
            "{SERVER_ID} {VERSION} bearer token required on every route except GET /healthz."
        );
        if config.allowed_hosts.is_empty() {
            eprintln!(
                "{SERVER_ID} {VERSION} MCP_ALLOWED_HOSTS is unset, so no Host header allow-list is applied."
            );
        }
    } else {
        let limit = if config.allowed_hosts.is_empty() {
            "localhost"
        } else {
            "MCP_ALLOWED_HOSTS"
        };
        eprintln!(
            "{SERVER_ID} {VERSION} bound to loopback, so no bearer token is required; Host is limited to {limit}."
        );
    }
}

/// Resolve on the first SIGINT or SIGTERM, naming it. A second signal exits at once: the
/// operator has stopped waiting for the drain.
async fn shutdown_signal() -> &'static str {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut interrupt = signal(SignalKind::interrupt()).expect("SIGINT handler");
        let mut terminate = signal(SignalKind::terminate()).expect("SIGTERM handler");
        let name = tokio::select! {
            _ = interrupt.recv() => "SIGINT",
            _ = terminate.recv() => "SIGTERM",
        };
        tokio::spawn(async move {
            tokio::select! {
                _ = interrupt.recv() => {},
                _ = terminate.recv() => {},
            }
            std::process::exit(0);
        });
        name
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
        "SIGINT"
    }
}

/// Serve HTTP on the configured address until a signal, then drain.
pub async fn serve(config: &Config, surface: &'static [Instruction], token: String) -> ExitCode {
    let cancel = CancellationToken::new();
    let options = app::AppOptions {
        token: token.clone(),
        allowed_hosts: config.allowed_hosts.clone(),
        local: config.local,
    };
    let router = match app::build(surface, &options, cancel.clone()) {
        Ok(router) => router,
        Err(problem) => {
            eprintln!("{SERVER_ID} {VERSION} {problem}");
            return ExitCode::FAILURE;
        }
    };

    let listener = match TcpListener::bind((bind_host(&config.host), config.port)).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!(
                "{SERVER_ID} {VERSION} cannot bind {}:{}: {error}",
                config.host, config.port
            );
            return ExitCode::FAILURE;
        }
    };
    announce(config, !token.is_empty());

    let drained = cancel.clone();
    let server = axum::serve(listener, router).with_graceful_shutdown(async move {
        let signal = shutdown_signal().await;
        eprintln!("{SERVER_ID} {VERSION} {signal}, draining");
        drained.cancel();
    });

    let served = tokio::select! {
        result = server => result,
        () = async {
            cancel.cancelled().await;
            tokio::time::sleep(DRAIN_GRACE).await;
        } => Ok(()),
    };
    match served {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{SERVER_ID} {VERSION} the HTTP server stopped: {error}");
            ExitCode::FAILURE
        }
    }
}
