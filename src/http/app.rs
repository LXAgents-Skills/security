//! The HTTP transport, as an application: an axum `Router`, built and returned. It does not
//! listen; `http::serve` owns the port, so the app can be built in a test without binding one.
//!
//! The surface is deliberately narrow: `POST /mcp` and `GET /healthz`, a JSON-RPC 404 for every
//! other path, and a 405 for any other method on `/mcp`. Requests pass three checks in this
//! order: the `Host` allow-list, the bearer token (everything except `GET /healthz`), and, on
//! `POST /mcp`, the 4 MB JSON body.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::Request,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post_service},
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::{
    auth::{ExpectedToken, require_token, token_problem},
    body::{BODY_LIMIT_BYTES, require_json_body},
    host::{AllowedHosts, LOCALHOST_NAMES, require_allowed_host},
    rpc_error::rpc_error,
};
use crate::{
    identity::{SERVER_ID, VERSION},
    instructions::Instruction,
    server::SecurityServer,
};

/// What the app is built from. Nothing here is read from the environment.
#[derive(Debug, Clone, Default)]
pub struct AppOptions {
    /// The bearer token every request but `GET /healthz` must carry. Empty means none.
    pub token: String,
    /// The `Host` allow-list; empty applies none, unless the server is local and open.
    pub allowed_hosts: Vec<String>,
    /// Whether the server binds loopback. Only then may `token` be empty.
    pub local: bool,
}

/// Build the HTTP application, or say why it cannot be built.
///
/// **There is no way to build it without a usable token** unless `local` is set: a missing or
/// too-short token is an error, so an unauthenticated server on a network cannot come from
/// forgetting an option. A local server with no token gets the localhost `Host` list when no
/// list is configured, because with no token the `Host` check is what stops a web page that
/// rebinds its own name to 127.0.0.1 from reading this server through the browser.
pub fn build(
    surface: &'static [Instruction],
    options: &AppOptions,
    shutdown: CancellationToken,
) -> Result<Router, String> {
    if let Some(problem) = token_problem(&options.token, !options.local) {
        return Err(format!("Refusing to build the HTTP app: {problem}."));
    }

    let routes = Router::new()
        .route("/healthz", get(health).fallback(not_found))
        .route(
            "/mcp",
            post_service(mcp_service(surface, shutdown))
                .route_layer(middleware::from_fn(require_json_body))
                .fallback(method_not_allowed),
        )
        .fallback(not_found);

    // The checks wrap the whole route table as one service, not each route: a layer added per
    // route runs inside axum's method dispatch, which would stamp `Allow:` on a 401 and tell a
    // caller without the token which methods a path takes.
    let mut router = Router::new().fallback_service(routes);

    // Layers wrap outward: the last one added runs first. So the token check is added before
    // the `Host` check, and the `Host` check is what a request meets first.
    let open = options.token.is_empty();
    if !open {
        let expected = ExpectedToken::new(&options.token);
        router = router.layer(middleware::from_fn_with_state(expected, require_token));
    }
    let hosts = match (options.allowed_hosts.is_empty(), open) {
        (false, _) => Some(AllowedHosts::new(options.allowed_hosts.iter().cloned())),
        (true, true) => Some(AllowedHosts::new(LOCALHOST_NAMES.iter().copied())),
        (true, false) => None,
    };
    if let Some(hosts) = hosts {
        router = router.layer(middleware::from_fn_with_state(hosts, require_allowed_host));
    }

    Ok(router)
}

/// The Skill MCP endpoint: stateless Streamable HTTP, a fresh handler per request, JSON
/// responses. There is no session store to keep bounded, because there are no sessions.
///
/// rmcp's own `Host` check is turned off because the router's layer above is the one place
/// that decision is made; two lists could disagree.
fn mcp_service(
    surface: &'static [Instruction],
    shutdown: CancellationToken,
) -> StreamableHttpService<SecurityServer, NeverSessionManager> {
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_max_request_body_bytes(BODY_LIMIT_BYTES)
        .with_cancellation_token(shutdown)
        .disable_allowed_hosts();
    StreamableHttpService::new(
        move || Ok(SecurityServer::new(surface)),
        Arc::new(NeverSessionManager::default()),
        config,
    )
}

/// The health check. Answers without a session, a request, or a tool.
async fn health() -> Response {
    Json(json!({ "status": "ok", "server": SERVER_ID, "version": VERSION })).into_response()
}

/// Any other method on `/mcp`. A refusal and not a 404: the path exists.
async fn method_not_allowed(request: Request) -> Response {
    rpc_error(
        StatusCode::METHOD_NOT_ALLOWED,
        -32000,
        format!("{} is not supported in stateless mode", request.method()),
    )
}

/// Anything else is not a route this server has.
async fn not_found(request: Request) -> Response {
    let target = request
        .uri()
        .path_and_query()
        .map_or_else(|| request.uri().path().to_owned(), ToString::to_string);
    rpc_error(
        StatusCode::NOT_FOUND,
        -32601,
        format!("Not found: {target}"),
    )
}
