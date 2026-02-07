//! MCP (Model Context Protocol) server implementation for NSIP Search API.

use crate::{NsipClient, SearchCriteria};
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars, tool, tool_handler, tool_router,
};

/// MCP server for NSIP Search API.
#[derive(Clone)]
pub struct NsipServer {
    tool_router: ToolRouter<Self>,
    client: NsipClient,
}

/// Parameters for searching animals.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    /// Breed group to filter by.
    #[schemars(description = "Breed group to filter by")]
    pub breed_group: Option<String>,

    /// Animal status to filter by.
    #[schemars(description = "Animal status to filter by")]
    pub status: Option<String>,

    /// Search query string.
    #[schemars(description = "Search query string")]
    pub query: Option<String>,

    /// Page number for pagination.
    #[schemars(description = "Page number for pagination")]
    pub page: Option<u32>,

    /// Number of results per page.
    #[schemars(description = "Number of results per page")]
    pub per_page: Option<u32>,
}

/// Parameters for getting animal details or lineage.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AnimalIdParams {
    /// Unique identifier of the animal.
    #[schemars(description = "Unique identifier of the animal")]
    pub animal_id: String,
}

impl Default for NsipServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl NsipServer {
    /// Creates a new NSIP MCP server.
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            client: NsipClient::new(),
        }
    }

    /// Search for animals in the NSIP database with optional filters.
    #[tool(description = "Search for animals in the NSIP database with optional filters")]
    async fn search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut criteria = SearchCriteria::new();

        if let Some(bg) = params.breed_group {
            criteria = criteria.with_breed_group(bg);
        }
        if let Some(s) = params.status {
            criteria = criteria.with_status(s);
        }
        if let Some(q) = params.query {
            criteria = criteria.with_query(q);
        }
        if let Some(p) = params.page {
            criteria = criteria.with_page(p);
        }
        if let Some(pp) = params.per_page {
            criteria = criteria.with_per_page(pp);
        }

        let results = self
            .client
            .search_animals(&criteria)
            .await
            .map_err(|e| McpError::internal_error(format!("Search failed: {e}"), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(format!("Serialization failed: {e}"), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get detailed information about a specific animal by ID.
    #[tool(description = "Get detailed information about a specific animal by ID")]
    async fn details(
        &self,
        Parameters(params): Parameters<AnimalIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let animal =
            self.client.details(&params.animal_id).await.map_err(|e| {
                McpError::internal_error(format!("Failed to fetch details: {e}"), None)
            })?;

        let json = serde_json::to_string_pretty(&animal)
            .map_err(|e| McpError::internal_error(format!("Serialization failed: {e}"), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get lineage (ancestry) information for a specific animal.
    #[tool(description = "Get lineage (ancestry) information for a specific animal")]
    async fn lineage(
        &self,
        Parameters(params): Parameters<AnimalIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let lineage =
            self.client.lineage(&params.animal_id).await.map_err(|e| {
                McpError::internal_error(format!("Failed to fetch lineage: {e}"), None)
            })?;

        let json = serde_json::to_string_pretty(&lineage)
            .map_err(|e| McpError::internal_error(format!("Serialization failed: {e}"), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

#[tool_handler]
impl ServerHandler for NsipServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "NSIP Search API server for querying livestock data from nsipsearch.nsip.org/api"
                    .to_string(),
            ),
        }
    }
}

/// Starts the MCP server on stdio transport.
///
/// # Errors
///
/// Returns an error if the server fails to start or encounters a runtime error.
pub async fn serve_stdio() -> crate::Result<()> {
    use rmcp::{ServiceExt, transport::stdio};

    let service =
        NsipServer::new()
            .serve(stdio())
            .await
            .map_err(|e| crate::Error::OperationFailed {
                operation: "mcp_serve".to_string(),
                cause: format!("{e}"),
            })?;

    service
        .waiting()
        .await
        .map_err(|e| crate::Error::OperationFailed {
            operation: "mcp_serve".to_string(),
            cause: format!("{e}"),
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = NsipServer::new();
        assert!(std::any::type_name_of_val(&server).contains("NsipServer"));
    }
}
