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
    /// Breed group ID to filter by.
    #[schemars(description = "Breed group ID to filter by")]
    pub breed_group_id: Option<i64>,

    /// Breed ID to filter by.
    #[schemars(description = "Breed ID to filter by")]
    pub breed_id: Option<i64>,

    /// Animal status to filter by.
    #[schemars(description = "Animal status to filter by (e.g. CURRENT, SOLD, DEAD)")]
    pub status: Option<String>,

    /// Gender filter.
    #[schemars(description = "Gender filter (Male, Female, Both)")]
    pub gender: Option<String>,

    /// Page number for pagination (0-indexed).
    #[schemars(description = "Page number for pagination (0-indexed)")]
    pub page: Option<u32>,

    /// Number of results per page (1-100).
    #[schemars(description = "Number of results per page (1-100)")]
    pub page_size: Option<u32>,
}

/// Parameters for getting animal details or lineage.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AnimalIdParams {
    /// LPN ID or registration number of the animal.
    #[schemars(description = "LPN ID or registration number of the animal")]
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

        if let Some(bg) = params.breed_group_id {
            criteria = criteria.with_breed_group_id(bg);
        }
        if let Some(bid) = params.breed_id {
            criteria = criteria.with_breed_id(bid);
        }
        if let Some(s) = params.status {
            criteria = criteria.with_status(s);
        }
        if let Some(g) = params.gender {
            criteria = criteria.with_gender(g);
        }

        let page = params.page.unwrap_or(0);
        let page_size = params.page_size.unwrap_or(15);

        let results = self
            .client
            .search_animals(
                page,
                page_size,
                params.breed_id,
                None,
                None,
                Some(&criteria),
            )
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
        let animal = self
            .client
            .animal_details(&params.animal_id)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to fetch details: {e}"), None))?;

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
    fn test_server_creation() {
        let server = NsipServer::new();
        assert!(std::any::type_name_of_val(&server).contains("NsipServer"));
    }
}
