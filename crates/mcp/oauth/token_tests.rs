#![allow(clippy::unwrap_used)]

use super::*;

use crate::mcp::oauth::{
    OAuthState,
    config::OAuthConfig,
    store::{InMemoryOAuthStore, IssuedAuthCode},
};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::post,
};
use tower::ServiceExt as _;

#[test]
fn pkce_verification_valid() {
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);

    assert!(verify_pkce(verifier, &challenge).is_ok());
}

#[test]
fn pkce_verification_invalid() {
    assert!(verify_pkce("wrong-verifier", "wrong-challenge").is_err());
}

#[test]
fn pkce_constant_time_comparison() {
    let verifier = "test-verifier-string";
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);

    assert!(verify_pkce(verifier, &challenge).is_ok());

    let mut bad_challenge = challenge;
    bad_challenge.push('x');
    assert!(verify_pkce(verifier, &bad_challenge).is_err());
}

#[test]
fn generate_refresh_token_uniqueness() {
    let t1 = generate_refresh_token();
    let t2 = generate_refresh_token();
    assert_ne!(t1, t2);
    // Base64url-encoded 32 bytes = 43 chars
    assert_eq!(t1.len(), 43);
}

#[test]
fn generate_refresh_token_is_url_safe_base64() {
    let token = generate_refresh_token();
    assert!(
        token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
        "token contains invalid chars: {token}"
    );
}

#[test]
fn token_request_deserializes_authorization_code() {
    let json = r#"{
        "grant_type": "authorization_code",
        "code": "abc123",
        "redirect_uri": "http://localhost:8080/callback",
        "code_verifier": "verifier-string",
        "client_id": "client-123"
    }"#;
    let req: TokenRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.grant_type, "authorization_code");
    assert_eq!(req.code.as_deref(), Some("abc123"));
    assert_eq!(
        req.redirect_uri.as_deref(),
        Some("http://localhost:8080/callback")
    );
    assert_eq!(req.code_verifier.as_deref(), Some("verifier-string"));
    assert_eq!(req.client_id.as_deref(), Some("client-123"));
    assert!(req.refresh_token.is_none());
}

#[test]
fn token_request_deserializes_refresh_token() {
    let json = r#"{
        "grant_type": "refresh_token",
        "refresh_token": "rt-abc"
    }"#;
    let req: TokenRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.grant_type, "refresh_token");
    assert_eq!(req.refresh_token.as_deref(), Some("rt-abc"));
    assert!(req.code.is_none());
}

#[test]
fn token_request_defaults_optional_fields() {
    let json = r#"{"grant_type": "authorization_code"}"#;
    let req: TokenRequest = serde_json::from_str(json).unwrap();
    assert!(req.code.is_none());
    assert!(req.redirect_uri.is_none());
    assert!(req.code_verifier.is_none());
    assert!(req.client_id.is_none());
    assert!(req.refresh_token.is_none());
}

#[test]
fn token_response_serializes_correctly() {
    let resp = TokenResponse {
        access_token: "jwt-token".to_owned(),
        token_type: "bearer".to_owned(),
        expires_in: 3600,
        refresh_token: "rt-xyz".to_owned(),
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["access_token"], "jwt-token");
    assert_eq!(json["token_type"], "bearer");
    assert_eq!(json["expires_in"], 3600);
    assert_eq!(json["refresh_token"], "rt-xyz");
}

#[test]
fn pkce_verification_empty_verifier_fails() {
    assert!(verify_pkce("", "some-challenge").is_err());
}

#[test]
fn pkce_verification_empty_challenge_fails() {
    assert!(verify_pkce("some-verifier", "").is_err());
}

#[test]
fn pkce_error_is_invalid_grant() {
    let err = verify_pkce("wrong", "challenge").unwrap_err();
    assert_eq!(err.error_code(), "invalid_grant");
    assert!(err.description().contains("PKCE"));
}

#[test]
fn token_request_debug_format() {
    let req = TokenRequest {
        grant_type: "authorization_code".to_owned(),
        code: Some("abc".to_owned()),
        redirect_uri: None,
        code_verifier: None,
        client_id: None,
        refresh_token: None,
    };
    let debug = format!("{req:?}");
    assert!(debug.contains("authorization_code"));
    assert!(debug.contains("abc"));
}

#[test]
fn token_response_debug_format() {
    let resp = TokenResponse {
        access_token: "jwt".to_owned(),
        token_type: "bearer".to_owned(),
        expires_in: 1800,
        refresh_token: "rt".to_owned(),
    };
    let debug = format!("{resp:?}");
    assert!(debug.contains("jwt"));
    assert!(debug.contains("bearer"));
    assert!(debug.contains("1800"));
}

#[test]
fn token_response_serializes_all_fields() {
    let resp = TokenResponse {
        access_token: "at".to_owned(),
        token_type: "bearer".to_owned(),
        expires_in: 7200,
        refresh_token: "rt-value".to_owned(),
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"access_token\":\"at\""));
    assert!(json.contains("\"token_type\":\"bearer\""));
    assert!(json.contains("\"expires_in\":7200"));
    assert!(json.contains("\"refresh_token\":\"rt-value\""));
}

#[test]
fn token_request_all_fields_populated() {
    let json = r#"{
        "grant_type": "authorization_code",
        "code": "code-val",
        "redirect_uri": "http://localhost/cb",
        "code_verifier": "verifier-val",
        "client_id": "client-val",
        "refresh_token": "rt-val"
    }"#;
    let req: TokenRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.grant_type, "authorization_code");
    assert_eq!(req.code.as_deref(), Some("code-val"));
    assert_eq!(req.redirect_uri.as_deref(), Some("http://localhost/cb"));
    assert_eq!(req.code_verifier.as_deref(), Some("verifier-val"));
    assert_eq!(req.client_id.as_deref(), Some("client-val"));
    assert_eq!(req.refresh_token.as_deref(), Some("rt-val"));
}

#[test]
fn pkce_verification_with_known_verifier() {
    let verifier = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);

    assert!(verify_pkce(verifier, &challenge).is_ok());
    assert!(verify_pkce("different-verifier", &challenge).is_err());
}

#[test]
fn generate_refresh_token_batch_uniqueness() {
    let tokens: Vec<String> = (0..10).map(|_| generate_refresh_token()).collect();
    let unique: std::collections::HashSet<&str> = tokens.iter().map(String::as_str).collect();
    assert_eq!(unique.len(), tokens.len(), "all tokens should be unique");
}

#[test]
fn generate_refresh_token_consistent_length() {
    for _ in 0..5 {
        let token = generate_refresh_token();
        assert_eq!(token.len(), 43, "base64url(32 bytes) = 43 chars");
    }
}

#[test]
fn token_request_deserializes_minimal_refresh() {
    let json = r#"{"grant_type": "refresh_token", "refresh_token": "my-rt"}"#;
    let req: TokenRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.grant_type, "refresh_token");
    assert_eq!(req.refresh_token.as_deref(), Some("my-rt"));
    assert!(req.code.is_none());
    assert!(req.redirect_uri.is_none());
    assert!(req.code_verifier.is_none());
    assert!(req.client_id.is_none());
}

#[test]
fn token_response_zero_expires_in() {
    let resp = TokenResponse {
        access_token: "at".to_owned(),
        token_type: "bearer".to_owned(),
        expires_in: 0,
        refresh_token: "rt".to_owned(),
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["expires_in"], 0);
}

#[test]
fn generate_refresh_token_not_empty() {
    let token = generate_refresh_token();
    assert!(!token.is_empty());
}

#[test]
fn generate_refresh_token_no_padding() {
    let token = generate_refresh_token();
    assert!(
        !token.contains('='),
        "URL_SAFE_NO_PAD should not have padding: {token}"
    );
}

#[test]
fn token_request_unknown_grant_type() {
    let json = r#"{"grant_type": "client_credentials"}"#;
    let req: TokenRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.grant_type, "client_credentials");
}

#[test]
fn token_request_empty_grant_type() {
    let json = r#"{"grant_type": ""}"#;
    let req: TokenRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.grant_type, "");
}

#[test]
fn pkce_verification_long_verifier() {
    let verifier = "a".repeat(128);
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);
    assert!(verify_pkce(&verifier, &challenge).is_ok());
}

#[test]
fn pkce_verification_unicode_verifier() {
    let verifier = "unicode-test-\u{1F600}-verifier";
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);
    assert!(verify_pkce(verifier, &challenge).is_ok());
}

#[test]
fn token_response_large_expires_in() {
    let resp = TokenResponse {
        access_token: "at".to_owned(),
        token_type: "bearer".to_owned(),
        expires_in: u64::MAX,
        refresh_token: "rt".to_owned(),
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["expires_in"], u64::MAX);
}

#[test]
fn token_request_with_only_refresh_token_field() {
    let json = r#"{"grant_type": "refresh_token", "refresh_token": "tok-123", "client_id": "c1"}"#;
    let req: TokenRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.grant_type, "refresh_token");
    assert_eq!(req.refresh_token.as_deref(), Some("tok-123"));
    assert_eq!(req.client_id.as_deref(), Some("c1"));
}

// -------------------------------------------------------------------
// Integration tests exercising exchange_token handler
// -------------------------------------------------------------------

fn test_oauth_state() -> OAuthState {
    let config = OAuthConfig {
        github_client_id: "gh-id".into(),
        github_client_secret: "gh-secret".into(),
        base_url: "https://example.com".into(),
        issuer: "https://example.com".into(),
        auth_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        allowed_users: None,
    };
    let store = std::sync::Arc::new(InMemoryOAuthStore::new())
        as std::sync::Arc<dyn crate::mcp::oauth::store::OAuthStoreBackend>;
    OAuthState::new(config, store)
}

fn token_app(state: OAuthState) -> Router {
    Router::new()
        .route("/token", post(exchange_token))
        .with_state(state)
}

#[tokio::test]
async fn exchange_token_authorization_code_full_flow() {
    let state = test_oauth_state();

    // Pre-seed an auth code in the store
    let verifier = "test-code-verifier-string-here";
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);

    state
        .store
        .store_auth_code(IssuedAuthCode {
            code: "auth-code-123".into(),
            client_id: "client-1".into(),
            redirect_uri: "https://example.com/cb".into(),
            code_challenge: challenge,
            github_login: "testuser".into(),
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    let app = token_app(state);
    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "code": "auth-code-123",
        "redirect_uri": "https://example.com/cb",
        "code_verifier": verifier,
        "client_id": "client-1"
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["token_type"], "bearer");
    assert!(json["access_token"].is_string());
    assert!(json["refresh_token"].is_string());
    assert_eq!(json["expires_in"], 3600);
}

#[tokio::test]
async fn exchange_token_refresh_token_flow() {
    let state = test_oauth_state();

    let verifier = "another-verifier";
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);

    state
        .store
        .store_auth_code(IssuedAuthCode {
            code: "code-for-rt".into(),
            client_id: "client-2".into(),
            redirect_uri: "https://example.com/cb".into(),
            code_challenge: challenge,
            github_login: "user2".into(),
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    // Exchange auth code to get a refresh token
    let app = token_app(state.clone());
    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "code": "code-for-rt",
        "redirect_uri": "https://example.com/cb",
        "code_verifier": verifier,
        "client_id": "client-2"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let rt = json["refresh_token"].as_str().unwrap().to_owned();

    // Exchange the refresh token (rotation)
    let app = token_app(state);
    let body = serde_json::json!({"grant_type": "refresh_token", "refresh_token": rt});
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["token_type"], "bearer");
    assert!(json["access_token"].is_string());
    assert_ne!(json["refresh_token"].as_str().unwrap(), rt);
}

#[tokio::test]
async fn exchange_token_error_paths() {
    let state = test_oauth_state();

    // Unsupported grant type
    let app = token_app(state.clone());
    let body = serde_json::json!({"grant_type": "client_credentials"});
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(resp.status().is_client_error());

    // Missing code for authorization_code grant
    let app = token_app(state.clone());
    let body = serde_json::json!({"grant_type": "authorization_code"});
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(resp.status().is_client_error());

    // Invalid refresh token
    let app = token_app(state);
    let body = serde_json::json!({
        "grant_type": "refresh_token",
        "refresh_token": "bad"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn exchange_token_form_encoded_unsupported_grant() {
    let state = test_oauth_state();
    let app = token_app(state);
    let form_body = "grant_type=client_credentials";
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(form_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn exchange_token_client_id_and_redirect_mismatch() {
    let state = test_oauth_state();

    let verifier = "mismatch-verifier";
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);

    state
        .store
        .store_auth_code(IssuedAuthCode {
            code: "code-mismatch-cid".into(),
            client_id: "real-client".into(),
            redirect_uri: "https://example.com/cb".into(),
            code_challenge: challenge.clone(),
            github_login: "user3".into(),
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    // Wrong client_id
    let app = token_app(state.clone());
    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "code": "code-mismatch-cid",
        "redirect_uri": "https://example.com/cb",
        "code_verifier": verifier,
        "client_id": "wrong-client"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(resp.status().is_client_error());

    // Seed another code for redirect_uri mismatch test
    state
        .store
        .store_auth_code(IssuedAuthCode {
            code: "code-mismatch-redir".into(),
            client_id: "real-client".into(),
            redirect_uri: "https://example.com/cb".into(),
            code_challenge: challenge,
            github_login: "user3".into(),
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    let app = token_app(state);
    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "code": "code-mismatch-redir",
        "redirect_uri": "https://wrong.com/cb",
        "code_verifier": verifier,
        "client_id": "real-client"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(resp.status().is_client_error());
}
