//! Streamable HTTP transport for the MCP server.

use std::sync::Arc;

use crate::mcp::NsipServer;

/// Starts the MCP server over HTTP with SSE support.
///
/// Binds to `host:port` and serves JSON-RPC requests at `/mcp`.
///
/// # Errors
///
/// Returns an error if the server fails to bind or encounters a runtime error.
pub async fn serve_http(host: &str, port: u16) -> crate::Result<()> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };

    let template_handler = NsipServer::new();
    let config = StreamableHttpServerConfig::default();
    let ct = config.cancellation_token.clone();

    let service: StreamableHttpService<NsipServer, LocalSessionManager> =
        StreamableHttpService::new(move || Ok(template_handler.clone()), Arc::default(), config);

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any)
        .expose_headers([axum::http::HeaderName::from_static("mcp-session-id")]);

    let router = axum::Router::new()
        .route_service("/mcp", service)
        .layer(cors);

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| crate::Error::Connection(format!("Failed to bind {addr}: {e}")))?;

    tracing::info!(addr = %addr, "MCP HTTP+SSE server listening");

    axum::serve(listener, router)
        .with_graceful_shutdown(async move { ct.cancelled_owned().await })
        .await
        .map_err(|e| crate::Error::Connection(format!("MCP HTTP server error: {e}")))?;

    Ok(())
}
