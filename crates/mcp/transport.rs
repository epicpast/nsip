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
/// normalization, session status correction, request logging, and
/// optional OAuth bearer token authentication.
///
/// When `oauth_state` is `Some`, the server requires bearer token
/// authentication on the `/mcp` endpoint and exposes OAuth protocol
/// endpoints (registration, authorization, callback, token exchange,
/// and discovery metadata).
///
/// # Errors
///
/// Returns an error if the server fails to bind or encounters a runtime error.
pub async fn serve_http(
    host: &str,
    port: u16,
    sets: super::tool_sets::EnabledToolSets,
    oauth_state: Option<super::oauth::OAuthState>,
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

    let mut router = axum::Router::new()
        .route_service("/mcp", service)
        .layer(middleware::from_fn(fix_rmcp_session_status))
        .layer(cors)
        .layer(middleware::from_fn(validate_origin))
        .layer(middleware::from_fn(log_requests))
        .layer(middleware::from_fn(ensure_sse_accept));

    // Wire in OAuth if configured.
    if let Some(ref state) = oauth_state {
        router = router
            .merge(super::oauth::oauth_router(state.clone()))
            .layer(middleware::from_fn_with_state(
                oauth_state.clone(),
                super::oauth::middleware::bearer_auth,
            ));
        tracing::info!("OAuth 2.1 + PAT bearer auth enabled");
    }

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
        let authority = origin
            .strip_prefix("http://")
            .or_else(|| origin.strip_prefix("https://"))
            .unwrap_or(origin);

        // IPv6 addresses are wrapped in brackets (e.g. `[::1]:9090`).
        // Split on `:` would break them, so handle bracketed hosts first.
        let host = if authority.starts_with('[') {
            authority
                .split_once(']')
                .map_or(authority, |(bracket, _)| bracket)
                .strip_prefix('[')
                .unwrap_or(authority)
        } else {
            authority.split(':').next().unwrap_or(authority)
        };

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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware as mw,
        routing::get,
    };
    use tower::ServiceExt as _;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    async fn unauthorized_handler() -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    /// Handler that echoes back the Accept header value in the response body.
    async fn echo_accept(req: axum::extract::Request) -> String {
        req.headers()
            .get(http::header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("<missing>")
            .to_owned()
    }

    // ── validate_origin ──────────────────────────────────────────────

    #[tokio::test]
    async fn validate_origin_no_header_passes() {
        let app = Router::new()
            .route("/", get(ok_handler))
            .layer(mw::from_fn(validate_origin));

        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn validate_origin_localhost_passes() {
        let app = Router::new()
            .route("/", get(ok_handler))
            .layer(mw::from_fn(validate_origin));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(http::header::ORIGIN, "http://localhost:8080")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn validate_origin_loopback_passes() {
        let app = Router::new()
            .route("/", get(ok_handler))
            .layer(mw::from_fn(validate_origin));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(http::header::ORIGIN, "http://127.0.0.1:3000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn validate_origin_ipv6_loopback_passes() {
        let app = Router::new()
            .route("/", get(ok_handler))
            .layer(mw::from_fn(validate_origin));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(http::header::ORIGIN, "http://[::1]:9090")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn validate_origin_all_interfaces_passes() {
        let app = Router::new()
            .route("/", get(ok_handler))
            .layer(mw::from_fn(validate_origin));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(http::header::ORIGIN, "http://0.0.0.0:8080")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn validate_origin_evil_domain_rejected() {
        let app = Router::new()
            .route("/", get(ok_handler))
            .layer(mw::from_fn(validate_origin));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(http::header::ORIGIN, "https://evil.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn validate_origin_example_com_rejected() {
        let app = Router::new()
            .route("/", get(ok_handler))
            .layer(mw::from_fn(validate_origin));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(http::header::ORIGIN, "http://example.com:8080")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    // ── ensure_sse_accept ────────────────────────────────────────────

    #[tokio::test]
    async fn ensure_sse_accept_no_header_adds_sse() {
        let app = Router::new()
            .route("/", get(echo_accept))
            .layer(mw::from_fn(ensure_sse_accept));

        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let accept = String::from_utf8(body.to_vec()).unwrap();
        assert!(
            accept.contains("text/event-stream"),
            "expected text/event-stream in Accept, got: {accept}"
        );
    }

    #[tokio::test]
    async fn ensure_sse_accept_json_only_adds_sse() {
        let app = Router::new()
            .route("/", get(echo_accept))
            .layer(mw::from_fn(ensure_sse_accept));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(http::header::ACCEPT, "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let accept = String::from_utf8(body.to_vec()).unwrap();
        assert!(
            accept.contains("text/event-stream"),
            "expected text/event-stream in Accept, got: {accept}"
        );
    }

    #[tokio::test]
    async fn ensure_sse_accept_already_present_unchanged() {
        let app = Router::new()
            .route("/", get(echo_accept))
            .layer(mw::from_fn(ensure_sse_accept));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(http::header::ACCEPT, "text/event-stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let accept = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(accept, "text/event-stream");
    }

    // ── fix_rmcp_session_status ──────────────────────────────────────

    #[tokio::test]
    async fn fix_rmcp_get_mcp_401_becomes_404() {
        let app = Router::new()
            .route("/mcp", get(unauthorized_handler))
            .layer(mw::from_fn(fix_rmcp_session_status));

        let resp = app
            .oneshot(Request::builder().uri("/mcp").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn fix_rmcp_post_mcp_401_stays_401() {
        let app = Router::new()
            .route("/mcp", axum::routing::post(unauthorized_handler))
            .layer(mw::from_fn(fix_rmcp_session_status));

        let resp = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/mcp")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn fix_rmcp_get_other_401_stays_401() {
        let app = Router::new()
            .route("/other", get(unauthorized_handler))
            .layer(mw::from_fn(fix_rmcp_session_status));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/other")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn fix_rmcp_get_mcp_200_stays_200() {
        let app = Router::new()
            .route("/mcp", get(ok_handler))
            .layer(mw::from_fn(fix_rmcp_session_status));

        let resp = app
            .oneshot(Request::builder().uri("/mcp").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── log_requests ─────────────────────────────────────────────────

    #[tokio::test]
    async fn log_requests_passes_through() {
        let app = Router::new()
            .route("/", get(ok_handler))
            .layer(mw::from_fn(log_requests));

        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
