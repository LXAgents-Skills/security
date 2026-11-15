//! The request body: at most 4 MB, and JSON.
//!
//! Read here, after the token check, so a caller without the token cannot make this server read
//! a body at all. A body that is too large and a body that is not JSON get the same answer,
//! 400 with `-32700`, because the server has always answered both that way.

use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use super::rpc_error::rpc_error;

/// The request body ceiling, in bytes.
pub const BODY_LIMIT_BYTES: usize = 4 * 1024 * 1024;

/// What a refused body is told.
pub const PARSE_ERROR: &str = "Parse error: request body is not valid JSON";

/// Buffer the body up to the limit, check that it parses as JSON, and hand it on unchanged.
pub async fn require_json_body(request: Request, next: Next) -> Response {
    let (parts, body) = request.into_parts();
    let Ok(bytes) = to_bytes(body, BODY_LIMIT_BYTES).await else {
        return rpc_error(StatusCode::BAD_REQUEST, -32700, PARSE_ERROR);
    };
    if serde_json::from_slice::<serde_json::Value>(&bytes).is_err() {
        return rpc_error(StatusCode::BAD_REQUEST, -32700, PARSE_ERROR);
    }
    next.run(Request::from_parts(parts, Body::from(bytes)))
        .await
}
