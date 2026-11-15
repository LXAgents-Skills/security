//! Bearer-token authentication for the HTTP transport.
//!
//! **The transport decides whether a token is needed.** stdio is a pipe its own client spawns,
//! so there is nobody to authenticate. HTTP is a socket anyone who can reach the port can open,
//! so every request must prove the caller holds the token, and the server refuses to run HTTP
//! without one rather than running open and saying so in a log nobody reads.
//!
//! **Where the socket listens decides whether that is a network.** A loopback bind can only be
//! reached from this machine, the same trust a stdio pipe has, so a token is optional there and
//! the localhost `Host` allow-list stops a web page reaching it by DNS rebinding. Any other bind
//! is a server, and it refuses to start without a token. A token that is set is always enforced.

use axum::{
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use super::rpc_error::rpc_error;

/// The shortest token the server will accept, in characters.
///
/// The token is the only thing standing between a network and this server, so a value like
/// `test` is refused at startup instead of being discovered by a scanner. 32 characters is what
/// `openssl rand -hex 16` produces; the documented command produces 64.
pub const MIN_TOKEN_LENGTH: usize = 32;

/// The configured token, trimmed, or an empty string when there is none.
///
/// Surrounding whitespace is dropped, because a token copied out of a file or an `env_file`
/// often carries a trailing newline and an HTTP header cannot.
pub fn configured_token() -> String {
    std::env::var("MCP_AUTH_TOKEN")
        .unwrap_or_default()
        .trim()
        .to_owned()
}

/// Why `token` cannot be used, or `None` when it can.
///
/// The message names the variable and the fix and never the value, so it is safe to log.
/// `required: false` is the local case: no token is then fine, but a token that is set and too
/// short is still a problem, because it was meant to protect something and would not.
pub fn token_problem(token: &str, required: bool) -> Option<String> {
    let length = token.chars().count();
    if length == 0 {
        return required.then(|| "MCP_AUTH_TOKEN is not set".to_owned());
    }
    (length < MIN_TOKEN_LENGTH).then(|| {
        format!("MCP_AUTH_TOKEN is {length} characters, and it must be at least {MIN_TOKEN_LENGTH}")
    })
}

/// The credential in an `Authorization: Bearer <credential>` header, or `None`.
///
/// The scheme is case-insensitive (RFC 7235). Another scheme, no credential, or two of them is
/// not a bearer credential.
fn bearer_credential(header: Option<&HeaderValue>) -> Option<&str> {
    let value = header?.to_str().ok()?;
    let (scheme, rest) = value.split_at_checked(6)?;
    if !scheme.eq_ignore_ascii_case("bearer") || !rest.starts_with(' ') {
        return None;
    }
    let credential = rest.trim_start_matches(' ');
    let valid = !credential.is_empty() && !credential.chars().any(char::is_whitespace);
    valid.then_some(credential)
}

/// The token a request must carry, as a fixed-length digest, so a comparison takes the same
/// time whatever the guess's length or how much of it was right.
#[derive(Clone)]
pub struct ExpectedToken([u8; 32]);

impl ExpectedToken {
    /// `token` is one that already passed [`token_problem`].
    pub fn new(token: &str) -> Self {
        Self(Sha256::digest(token.as_bytes()).into())
    }

    fn matches(&self, credential: &str) -> bool {
        let digest: [u8; 32] = Sha256::digest(credential.as_bytes()).into();
        self.0.ct_eq(&digest).into()
    }
}

/// Let a request through only when it carries the token. `GET /healthz` is the one exemption,
/// and it is exact: the method and the path, nothing that merely starts with it.
///
/// A refusal is a `401` with `WWW-Authenticate: Bearer`; a request that sent a credential which
/// did not match also gets `error="invalid_token"`. Neither reveals anything about the token.
pub async fn require_token(
    State(expected): State<ExpectedToken>,
    request: Request,
    next: Next,
) -> Response {
    if request.method() == Method::GET && request.uri().path() == "/healthz" {
        return next.run(request).await;
    }

    let credential = bearer_credential(request.headers().get(header::AUTHORIZATION));
    if credential.is_some_and(|credential| expected.matches(credential)) {
        return next.run(request).await;
    }

    let (challenge, message) = match credential {
        None => (
            "Bearer realm=\"mcp\"",
            "Unauthorized: send the token as Authorization: Bearer <token>",
        ),
        Some(_) => (
            "Bearer realm=\"mcp\", error=\"invalid_token\"",
            "Unauthorized: the token is not valid",
        ),
    };
    let mut response = rpc_error(StatusCode::UNAUTHORIZED, -32001, message);
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static(challenge),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "test-token-0123456789abcdef0123456789abcdef";

    #[test]
    fn token_problem_names_the_variable_and_never_the_value() {
        assert_eq!(
            token_problem("", true).as_deref(),
            Some("MCP_AUTH_TOKEN is not set")
        );
        assert_eq!(
            token_problem("", false),
            None,
            "no token is fine when not required"
        );
        let short = token_problem("short", false).unwrap();
        assert!(short.contains("must be at least 32") && !short.contains("short,"));
        assert_eq!(
            token_problem(&"k".repeat(31), true).as_deref(),
            Some("MCP_AUTH_TOKEN is 31 characters, and it must be at least 32")
        );
        assert_eq!(token_problem(&"k".repeat(32), true), None);
        assert_eq!(token_problem(TOKEN, true), None);
    }

    #[test]
    fn only_a_bearer_credential_counts() {
        let parse = |raw: &str| {
            bearer_credential(Some(&HeaderValue::from_str(raw).unwrap())).map(str::to_owned)
        };
        assert_eq!(parse(&format!("Bearer {TOKEN}")).as_deref(), Some(TOKEN));
        assert_eq!(parse(&format!("bearer   {TOKEN}")).as_deref(), Some(TOKEN));
        assert_eq!(parse(&format!("BEARER {TOKEN}")).as_deref(), Some(TOKEN));
        for refused in [
            format!("Basic {TOKEN}"),
            "Bearer".into(),
            "Bearer ".into(),
            format!("Bearer {TOKEN} {TOKEN}"),
            TOKEN.into(),
            format!("Bearer{TOKEN}"),
        ] {
            assert_eq!(parse(&refused), None, "{refused}");
        }
        assert_eq!(bearer_credential(None), None);
    }

    #[test]
    fn the_comparison_is_exact() {
        let expected = ExpectedToken::new(TOKEN);
        assert!(expected.matches(TOKEN));
        assert!(!expected.matches(&TOKEN[..TOKEN.len() - 1]));
        assert!(!expected.matches(&format!("{TOKEN}0")));
        assert!(!expected.matches(&TOKEN.to_uppercase()));
    }
}
