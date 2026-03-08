//! Streamable HTTP transport for the MCP server.
//!
//! Contains `serve_http` and the HTTP middleware functions
//! (`log_requests`, `fix_rmcp_session_status`, `validate_origin`,
//! `ensure_sse_accept`).

use std::sync::Arc;

use axum::{http, middleware};

use crate::mcp::NsipServer;

/// Starts the MCP server over HTTP with SSE support.
///
/// Binds to `host:port` and serves JSON-RPC requests at `/mcp`.
/// Includes middleware for DNS rebinding protection, Accept header
/// normalization, session status correction, and request logging.
///
/// # Errors
///
/// Returns an error if the server fails to bind or encounters a runtime error.
pub async fn serve_http(
    host: &str,
    port: u16,
    sets: super::tool_sets::EnabledToolSets,
) -> crate::Result<()> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };

    let template_handler = NsipServer::with_tool_sets(sets);
    let config = StreamableHttpServerConfig::default();
    let ct = config.cancellation_token.clone();

    let service: StreamableHttpService<NsipServer, LocalSessionManager> =
        StreamableHttpService::new(move || Ok(template_handler.clone()), Arc::default(), config);

    // CORS: Claude Code sends an OPTIONS preflight before POST. Without
    // this layer rmcp returns 405 and the client never reaches initialize.
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::DELETE,
            http::Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any)
        .expose_headers([http::HeaderName::from_static("mcp-session-id")]);

    let router = axum::Router::new()
        .route_service("/mcp", service)
        .layer(middleware::from_fn(fix_rmcp_session_status))
        .layer(cors)
        .layer(middleware::from_fn(validate_origin))
        .layer(middleware::from_fn(log_requests))
        .layer(middleware::from_fn(ensure_sse_accept));

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

/// Middleware that logs inbound requests.
///
/// Logs all requests at INFO on arrival. Responses are logged at INFO for
/// success and WARN for 4xx/5xx so errors stand out without losing visibility
/// into normal traffic.
async fn log_requests(
    req: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let session = req
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("<none>")
        .to_owned();
    tracing::info!(%method, %uri, session_id = %session, "inbound request");
    let resp = next.run(req).await;
    let status = resp.status();
    if status.is_client_error() || status.is_server_error() {
        tracing::warn!(%method, %uri, %status, session_id = %session, "error response");
    } else {
        tracing::info!(%method, %uri, %status, "response");
    }
    resp
}

/// Middleware that fixes rmcp's incorrect 401 status for unknown/missing sessions.
///
/// Per the MCP spec, unknown sessions should return 404, not 401.
async fn fix_rmcp_session_status(
    req: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let is_get = req.method() == http::Method::GET;
    let path = req.uri().path().to_owned();
    let mut resp = next.run(req).await;
    // Only fix 401s from the /mcp endpoint on GET (SSE reconnect).
    if is_get && path == "/mcp" && resp.status() == http::StatusCode::UNAUTHORIZED {
        *resp.status_mut() = http::StatusCode::NOT_FOUND;
    }
    resp
}

/// Middleware that validates the `Origin` header for DNS rebinding protection.
///
/// When present, the `Origin` header must match a localhost address.
/// Requests with a non-local `Origin` are rejected with `403 Forbidden`.
async fn validate_origin(
    req: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    if let Some(origin) = req
        .headers()
        .get(http::header::ORIGIN)
        .and_then(|v| v.to_str().ok())
    {
        let host = origin
            .strip_prefix("http://")
            .or_else(|| origin.strip_prefix("https://"))
            .unwrap_or(origin)
            .split(':')
            .next()
            .unwrap_or(origin);

        let is_local = matches!(
            host,
            "localhost" | "127.0.0.1" | "[::1]" | "::1" | "0.0.0.0"
        );

        if !is_local {
            tracing::warn!(origin = %origin, "rejected non-local Origin (DNS rebinding protection)");
            return axum::response::Response::builder()
                .status(http::StatusCode::FORBIDDEN)
                .body(axum::body::Body::from("Forbidden: non-local Origin"))
                .unwrap_or_else(|_| {
                    axum::response::Response::new(axum::body::Body::from("Forbidden"))
                });
        }
    }

    next.run(req).await
}

/// Middleware that ensures the `Accept` header includes `text/event-stream`.
///
/// rmcp's `StreamableHttpService` rejects requests with 406 if the Accept
/// header does not include `text/event-stream`. This layer normalizes the
/// header before the request reaches rmcp.
async fn ensure_sse_accept(
    mut req: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let missing_sse = req
        .headers()
        .get(http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_none_or(|v| !v.contains("text/event-stream"));

    if missing_sse {
        req.headers_mut().insert(
            http::header::ACCEPT,
            http::HeaderValue::from_static("application/json, text/event-stream"),
        );
    }

    next.run(req).await
}
