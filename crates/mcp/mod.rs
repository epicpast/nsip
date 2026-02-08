//! MCP (Model Context Protocol) server implementation for NSIP Search API.
//!
//! Provides a full MCP server with tools, resources, resource templates, and
//! prompts for livestock breeding intelligence.

pub mod analytics;
pub mod prompts;
pub mod resources;
mod tools;

use std::collections::HashMap;

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{
        GetPromptRequestParams, GetPromptResult, Implementation, ListPromptsResult,
        ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParams, ProtocolVersion,
        ReadResourceRequestParams, ReadResourceResult, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool_handler,
};

use crate::NsipClient;

/// MCP server for NSIP Search API with full protocol support.
///
/// Exposes 13 tools, 5 static resources, 4 resource templates, and 7 guided
/// breeding prompts backed by the NSIP API and local analytics.
#[derive(Clone)]
pub struct NsipServer {
    tool_router: ToolRouter<Self>,
    pub(crate) client: NsipClient,
}

impl Default for NsipServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler]
impl ServerHandler for NsipServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "NSIP Livestock Intelligence Server — search animals, compare EBVs, \
                 check inbreeding coefficients, get mating recommendations, and access \
                 guided breeding prompts. Covers the full nsipsearch.nsip.org/api surface \
                 with analytics-powered decision support for sheep breeders."
                    .to_string(),
            ),
        }
    }

    // -- Prompts ---------------------------------------------------------------

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(prompts::list_prompts())
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<rmcp::service::RoleServer>,
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

        prompts::get_prompt(&self.client, &request.name, &arguments).await
    }

    // -- Resources -------------------------------------------------------------

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(resources::list_resources())
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(resources::list_resource_templates())
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        resources::read_resource(&self.client, &request).await
    }
}

/// Starts the MCP server on stdio transport.
///
/// # Errors
///
/// Returns an error if the server fails to start or encounters a runtime error.
pub async fn serve_stdio() -> crate::Result<()> {
    use rmcp::{ServiceExt, transport::stdio};

    let service = NsipServer::new()
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
        assert_eq!(info.protocol_version, ProtocolVersion::V_2024_11_05);
    }

    #[test]
    fn server_default_is_same_as_new() {
        let server = NsipServer::default();
        let info = server.get_info();
        assert_eq!(info.protocol_version, ProtocolVersion::V_2024_11_05);
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
        let instructions = info.instructions.as_deref().unwrap();
        assert!(
            instructions.contains("NSIP"),
            "Instructions should mention NSIP"
        );
        assert!(
            instructions.contains("sheep"),
            "Instructions should mention sheep"
        );
        assert!(
            instructions.contains("breeding"),
            "Instructions should mention breeding"
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
    fn server_is_clone() {
        let server = NsipServer::new();
        #[allow(clippy::redundant_clone)]
        let cloned = server.clone();
        let info = cloned.get_info();
        assert_eq!(info.protocol_version, ProtocolVersion::V_2024_11_05);
    }
}
