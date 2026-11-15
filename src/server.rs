//! The Skill MCP handler: lists the surface and answers a call with the file it names.
//!
//! The surface is not written here. It is the one `instructions::surface()` derives from
//! `content/`, so the CLI, stdio and HTTP all serve the same tools by construction.

use std::sync::{Arc, LazyLock};

use rmcp::{
    ErrorData, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
        JsonObject, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig,
        Tool,
    },
    service::{MaybeSendFuture, RequestContext, RoleServer},
};

use crate::{
    identity::{SERVER_ID, SERVER_INSTRUCTIONS, SERVER_TITLE, VERSION},
    instructions::Instruction,
};

/// The input schema every tool declares: an object with no properties. No tool takes an
/// argument, so there is no `path` to traverse with and no verb to act on.
static NO_ARGUMENTS: LazyLock<Arc<JsonObject>> = LazyLock::new(|| {
    let mut schema = JsonObject::new();
    schema.insert("type".into(), "object".into());
    schema.insert("properties".into(), JsonObject::new().into());
    Arc::new(schema)
});

/// The handler for one connection. Holds nothing but a reference to the immutable surface, so
/// a fresh one per HTTP request costs nothing and shares nothing.
#[derive(Clone, Copy)]
pub struct SecurityServer {
    surface: &'static [Instruction],
}

impl SecurityServer {
    pub fn new(surface: &'static [Instruction]) -> Self {
        Self { surface }
    }

    fn tool(instruction: &Instruction) -> Tool {
        Tool::new(
            instruction.name.clone(),
            instruction.description.clone(),
            NO_ARGUMENTS.clone(),
        )
    }
}

impl ServerHandler for SecurityServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(SERVER_ID, VERSION).with_title(SERVER_TITLE))
            .with_instructions(SERVER_INSTRUCTIONS)
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + MaybeSendFuture + '_ {
        let tools = self.surface.iter().map(Self::tool).collect();
        std::future::ready(Ok(ListToolsResult::with_all_items(tools)))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResponse, ErrorData>> + MaybeSendFuture + '_ {
        let found = self
            .surface
            .iter()
            .find(|instruction| instruction.name == request.name);
        let answer = match found {
            Some(instruction) => {
                Ok(CallToolResult::success(vec![ContentBlock::text(instruction.text)]).into())
            }
            None => Err(ErrorData::invalid_params(
                format!("Tool {} not found", request.name),
                None,
            )),
        };
        std::future::ready(answer)
    }
}
