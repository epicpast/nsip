//! MCP (Model Context Protocol) server implementation for NSIP Search API.
//!
//! Provides a full MCP server with tools, resources, resource templates, and
//! prompts for livestock breeding intelligence.

pub mod analytics;
pub(crate) mod elicitation;
mod instructions;
pub mod oauth;
pub mod prompts;
pub mod resources;
#[cfg(feature = "telemetry")]
pub mod telemetry;
pub mod tool_sets;
mod tools;
mod transport;

use std::collections::HashMap;

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{
        GetPromptRequestParams, GetPromptResult, ListPromptsResult, ListResourceTemplatesResult,
        ListResourcesResult, PaginatedRequestParams, ProtocolVersion, ReadResourceRequestParams,
        ReadResourceResult, ServerCapabilities, ServerInfo, SubscribeRequestParams,
        UnsubscribeRequestParams,
    },
    service::{NotificationContext, RequestContext},
    tool_handler,
};

use crate::NsipClient;

/// Default page size for cursor-based pagination of list endpoints.
const DEFAULT_PAGE_SIZE: usize = 25;

/// Paginate a slice with opaque cursor-based pagination.
///
/// Returns the current page and an optional cursor for the next page.
///
/// # Errors
///
/// Returns an error if the cursor is not a valid offset or is out of range.
fn paginate<T: Clone>(
    items: &[T],
    cursor: Option<&str>,
    page_size: usize,
) -> Result<(Vec<T>, Option<String>), McpError> {
    let offset = match cursor {
        Some(c) => c
            .parse::<usize>()
            .map_err(|_| McpError::invalid_params("Invalid pagination cursor", None))?,
        None => 0,
    };
    if offset > items.len() {
        return Err(McpError::invalid_params("Cursor out of range", None));
    }
    let end = (offset + page_size).min(items.len());
    let page = items[offset..end].to_vec();
    let next = if end < items.len() {
        Some(end.to_string())
    } else {
        None
    };
    Ok((page, next))
}

/// MCP server for NSIP Search API with full protocol support.
///
/// Exposes up to 13 tools (filtered by enabled tool sets), 5 static resources,
/// 4 resource templates, and 7 guided breeding prompts backed by the NSIP API
/// and local analytics.
#[derive(Clone)]
pub struct NsipServer {
    tool_router: ToolRouter<Self>,
    pub(crate) client: NsipClient,
    pub(crate) enabled_tools: tool_sets::EnabledToolSets,
}

impl Default for NsipServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler]
impl ServerHandler for NsipServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .enable_prompts()
                .enable_prompts_list_changed()
                .enable_resources()
                .enable_resources_list_changed()
                .enable_logging()
                .build(),
        )
        .with_protocol_version(ProtocolVersion::LATEST)
        .with_instructions(instructions::build_instructions(&self.enabled_tools))
    }

    // -- Prompts ---------------------------------------------------------------

    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        let all = prompts::list_prompts();
        let cursor = request.as_ref().and_then(|r| r.cursor.as_deref());
        let (page, next_cursor) = paginate(&all.prompts, cursor, DEFAULT_PAGE_SIZE)?;
        Ok(ListPromptsResult {
            meta: None,
            next_cursor,
            prompts: page,
        })
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let arguments: HashMap<String, String> = request
            .arguments
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| {
                if let serde_json::Value::String(s) = v {
                    (k, s)
                } else {
                    (k, v.to_string())
                }
            })
            .collect();

        prompts::get_prompt(&self.client, &request.name, &arguments, Some(&context)).await
    }

    // -- Resources -------------------------------------------------------------

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let all = resources::list_resources();
        let cursor = request.as_ref().and_then(|r| r.cursor.as_deref());
        let (page, next_cursor) = paginate(&all.resources, cursor, DEFAULT_PAGE_SIZE)?;
        Ok(ListResourcesResult {
            meta: None,
            next_cursor,
            resources: page,
        })
    }

    async fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        let all = resources::list_resource_templates();
        let cursor = request.as_ref().and_then(|r| r.cursor.as_deref());
        let (page, next_cursor) = paginate(&all.resource_templates, cursor, DEFAULT_PAGE_SIZE)?;
        Ok(ListResourceTemplatesResult {
            meta: None,
            next_cursor,
            resource_templates: page,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        resources::read_resource(&self.client, &request).await
    }

    // -- Lifecycle -------------------------------------------------------------

    async fn on_initialized(&self, _context: NotificationContext<rmcp::service::RoleServer>) {
        tracing::info!("NSIP MCP client initialized");
    }

    async fn subscribe(
        &self,
        _request: SubscribeRequestParams,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<(), McpError> {
        Ok(())
    }

    async fn unsubscribe(
        &self,
        _request: UnsubscribeRequestParams,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<(), McpError> {
        Ok(())
    }
}

pub use transport::serve_http;

/// Starts the MCP server on stdio transport.
///
/// # Errors
///
/// Returns an error if the server fails to start or encounters a runtime error.
pub async fn serve_stdio(sets: tool_sets::EnabledToolSets) -> crate::Result<()> {
    use rmcp::{ServiceExt, transport::stdio};

    let service = NsipServer::with_tool_sets(sets)
        .serve(stdio())
        .await
        .map_err(|e| crate::Error::Connection(format!("MCP server failed to start: {e}")))?;

    service
        .waiting()
        .await
        .map_err(|e| crate::Error::Connection(format!("MCP server error: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_creation() {
        let server = NsipServer::new();
        let info = server.get_info();
        assert_eq!(info.protocol_version, ProtocolVersion::LATEST);
    }

    #[test]
    fn server_default_is_same_as_new() {
        let server = NsipServer::default();
        let info = server.get_info();
        assert_eq!(info.protocol_version, ProtocolVersion::LATEST);
    }

    #[test]
    fn capabilities_include_all_protocol_features() {
        let server = NsipServer::new();
        let info = server.get_info();
        let caps = &info.capabilities;
        assert!(caps.tools.is_some(), "tools capability missing");
        assert!(caps.prompts.is_some(), "prompts capability missing");
        assert!(caps.resources.is_some(), "resources capability missing");
    }

    #[test]
    fn server_info_has_instructions() {
        let server = NsipServer::new();
        let info = server.get_info();
        let text = info.instructions.as_deref().unwrap();
        assert!(text.contains("NSIP"), "Instructions should mention NSIP");
        assert!(
            text.contains("search"),
            "Instructions should mention search tool"
        );
        assert!(
            text.contains("evaluate-ram"),
            "Instructions should mention evaluate-ram prompt"
        );
        assert!(
            text.contains("nsip://"),
            "Instructions should reference nsip:// URIs"
        );
    }

    #[test]
    fn server_info_has_implementation() {
        let server = NsipServer::new();
        let info = server.get_info();
        // Implementation is built from env at compile time
        let impl_info = &info.server_info;
        assert!(
            !impl_info.name.is_empty(),
            "Server implementation name should not be empty"
        );
    }

    #[test]
    fn with_tool_sets_filters_tools() {
        let sets = tool_sets::EnabledToolSets::from_csv("search");
        let server = NsipServer::with_tool_sets(sets);
        let info = server.get_info();
        assert_eq!(info.protocol_version, ProtocolVersion::LATEST);
        // Instructions should mention search but not analytics
        let text = info.instructions.as_deref().unwrap_or_default();
        assert!(text.contains("## Search & Retrieval Tools"));
        assert!(!text.contains("## Analytics Tools"));
    }

    #[test]
    fn server_is_clone() {
        let server = NsipServer::new();
        #[allow(clippy::redundant_clone)]
        let cloned = server.clone();
        let info = cloned.get_info();
        assert_eq!(info.protocol_version, ProtocolVersion::LATEST);
    }

    // --- Paginate tests ---

    #[test]
    fn paginate_first_page() {
        let items: Vec<i32> = (0..10).collect();
        let (page, next) = paginate(&items, None, 3).unwrap();
        assert_eq!(page, vec![0, 1, 2]);
        assert_eq!(next.as_deref(), Some("3"));
    }

    #[test]
    fn paginate_middle_page() {
        let items: Vec<i32> = (0..10).collect();
        let (page, next) = paginate(&items, Some("3"), 3).unwrap();
        assert_eq!(page, vec![3, 4, 5]);
        assert_eq!(next.as_deref(), Some("6"));
    }

    #[test]
    fn paginate_last_page() {
        let items: Vec<i32> = (0..10).collect();
        let (page, next) = paginate(&items, Some("9"), 3).unwrap();
        assert_eq!(page, vec![9]);
        assert!(next.is_none());
    }

    #[test]
    fn paginate_exact_boundary() {
        let items: Vec<i32> = (0..6).collect();
        let (page, next) = paginate(&items, Some("3"), 3).unwrap();
        assert_eq!(page, vec![3, 4, 5]);
        assert!(next.is_none());
    }

    #[test]
    fn paginate_empty_items() {
        let items: Vec<i32> = vec![];
        let (page, next) = paginate(&items, None, 5).unwrap();
        assert!(page.is_empty());
        assert!(next.is_none());
    }

    #[test]
    fn paginate_invalid_cursor() {
        let items: Vec<i32> = (0..5).collect();
        let result = paginate(&items, Some("not-a-number"), 3);
        assert!(result.is_err());
    }

    #[test]
    fn paginate_cursor_out_of_range() {
        let items: Vec<i32> = (0..5).collect();
        let result = paginate(&items, Some("100"), 3);
        assert!(result.is_err());
    }

    #[test]
    fn paginate_cursor_at_end() {
        let items: Vec<i32> = (0..5).collect();
        let (page, next) = paginate(&items, Some("5"), 3).unwrap();
        assert!(page.is_empty());
        assert!(next.is_none());
    }

    #[test]
    fn paginate_page_size_larger_than_items() {
        let items: Vec<i32> = (0..3).collect();
        let (page, next) = paginate(&items, None, 100).unwrap();
        assert_eq!(page, vec![0, 1, 2]);
        assert!(next.is_none());
    }
}
