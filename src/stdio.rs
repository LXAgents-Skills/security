//! Serve over stdio.
//!
//! stdout is the JSON-RPC channel here, so nothing on this path writes to it but the
//! transport. Nothing here reads `MCP_AUTH_TOKEN`: whoever can use the pipe already runs the
//! process.

use rmcp::{ServiceExt, service::ServerInitializeError, transport::stdio};

use crate::{
    identity::{SERVER_ID, VERSION},
    instructions::Instruction,
    server::SecurityServer,
};

/// Serve `surface` on stdin and stdout until the client closes the pipe.
///
/// A pipe closed before the handshake is a client that went away, not a failure: the process
/// ends cleanly, as it does when the pipe closes after one.
pub async fn serve(surface: &'static [Instruction]) -> std::io::Result<()> {
    let running = match SecurityServer::new(surface).serve(stdio()).await {
        Ok(running) => running,
        Err(ServerInitializeError::ConnectionClosed(_)) => return Ok(()),
        Err(error) => return Err(std::io::Error::other(error)),
    };
    eprintln!("{SERVER_ID} {VERSION} serving over stdio");
    running.waiting().await.map_err(std::io::Error::other)?;
    Ok(())
}
