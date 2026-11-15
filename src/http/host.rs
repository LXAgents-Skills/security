//! The `Host` header allow-list.
//!
//! Applied ahead of the token check and every route, `/healthz` included: an allow-list that
//! guarded `/mcp` and not `/healthz` would let a deployment watch a green health check on a
//! service nothing can reach.

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};

use super::rpc_error::rpc_error;

/// The names a server answers to when it runs without a token on loopback.
pub const LOCALHOST_NAMES: &[&str] = &["localhost", "127.0.0.1", "[::1]"];

/// The hostnames a request's `Host` header may name. The port is not part of the match.
#[derive(Clone)]
pub struct AllowedHosts(Arc<[String]>);

impl AllowedHosts {
    pub fn new(hosts: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self(
            hosts
                .into_iter()
                .map(|host| host.into().to_lowercase())
                .collect(),
        )
    }

    fn allows(&self, hostname: &str) -> bool {
        self.0.iter().any(|allowed| allowed == hostname)
    }
}

/// The hostname in a `Host` header: lowercased, without the port, brackets kept for IPv6.
pub fn hostname_of(host: &str) -> &str {
    if host.starts_with('[') {
        return host.find(']').map_or(host, |end| &host[..=end]);
    }
    host.rsplit_once(':').map_or(host, |(name, _port)| name)
}

/// Refuse a request whose `Host` is missing or not on the list, with a 403 in the JSON-RPC
/// envelope.
pub async fn require_allowed_host(
    State(allowed): State<AllowedHosts>,
    request: Request,
    next: Next,
) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if host.is_empty() {
        return rpc_error(StatusCode::FORBIDDEN, -32000, "Missing Host header");
    }
    if !allowed.allows(&hostname_of(host).to_lowercase()) {
        return rpc_error(
            StatusCode::FORBIDDEN,
            -32000,
            format!("Invalid Host: {}", hostname_of(host)),
        );
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_port_is_not_part_of_the_match() {
        assert_eq!(
            hostname_of("security.example.test:8080"),
            "security.example.test"
        );
        assert_eq!(
            hostname_of("security.example.test"),
            "security.example.test"
        );
        assert_eq!(hostname_of("[::1]:3000"), "[::1]");
        assert_eq!(hostname_of("[::1]"), "[::1]");
    }

    #[test]
    fn the_list_is_exact_and_case_insensitive() {
        let allowed = AllowedHosts::new(["Security.Example.Test"]);
        assert!(allowed.allows("security.example.test"));
        assert!(!allowed.allows("evil.test"));
        assert!(!allowed.allows("security.example.test.evil.test"));
    }
}
