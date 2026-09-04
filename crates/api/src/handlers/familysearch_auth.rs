use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, state::AppState};
use neogenealogy_storage::external_research::ResearchProvider;
use neogenealogy_storage::familysearch::FamilySearchHttpExecutor;

#[derive(Deserialize)]
pub struct AuthorizeParams {
    pub redirect_uri: Option<String>,
}

#[derive(Serialize)]
pub struct AuthorizeResponse {
    pub authorization_url: String,
    pub state: String,
}

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Deserialize)]
pub struct FamilySearchSearchParams {
    pub q: Option<String>,
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    #[serde(rename = "surname")]
    pub surname: Option<String>,
    #[serde(rename = "birthLikeDate")]
    pub birth_date: Option<String>,
    #[serde(rename = "birthLikePlace")]
    pub birth_place: Option<String>,
    pub query: Option<String>,
}

/// GET /api/v1/auth/familysearch/authorize
pub async fn authorize(State(state): State<AppState>) -> Result<Json<AuthorizeResponse>, ApiError> {
    let cfg = neogenealogy_storage::familysearch::FamilySearchConfig::from_env();
    if cfg.client_id.is_none() {
        return Err(ApiError::bad_request(
            "FAMILYSEARCH_NOT_CONFIGURED",
            "FamilySearch is not configured. Set NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID",
        ));
    }
    let st = uuid::Uuid::new_v4().to_string();
    // Save state
    state
        .storage
        .save_oauth_state(&st)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    // Cleanup old states
    let _ = state.storage.cleanup_expired_oauth_states().await;
    let url = cfg
        .authorization_url(&st)
        .map_err(|e| ApiError::bad_request("AUTH_ERROR", e.message))?;
    Ok(Json(AuthorizeResponse {
        authorization_url: url,
        state: st,
    }))
}

/// GET /api/v1/auth/familysearch/callback?code=&state=
pub async fn callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
) -> Result<axum::response::Response, ApiError> {
    if let Some(err) = params.error {
        let desc = params
            .error_description
            .unwrap_or_else(|| "authorization failed".to_string());
        // Redirect to frontend with error (simple, no extra encoding for now)
        let cfg = neogenealogy_storage::familysearch::FamilySearchConfig::from_env();
        let frontend = cfg.frontend_redirect;
        // Basic sanitization: replace non-alphanum with _
        let safe_err: String = err
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let safe_desc: String = desc
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let url = format!(
            "{}?familysearch_error={}&familysearch_error_description={}",
            frontend, safe_err, safe_desc
        );
        return Ok(Redirect::temporary(&url).into_response());
    }
    let code = params
        .code
        .ok_or_else(|| ApiError::bad_request("INVALID_CODE", "missing code from FamilySearch"))?;
    let st = params
        .state
        .ok_or_else(|| ApiError::bad_request("INVALID_STATE", "missing state from FamilySearch"))?;
    // Validate state
    let valid = state
        .storage
        .consume_oauth_state(&st)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !valid {
        return Err(ApiError::bad_request(
            "INVALID_STATE",
            "invalid or expired state",
        ));
    }
    let cfg = neogenealogy_storage::familysearch::FamilySearchConfig::from_env();
    let client_id = cfg.client_id.clone().ok_or_else(|| {
        ApiError::bad_request(
            "FAMILYSEARCH_NOT_CONFIGURED",
            "FamilySearch is not configured",
        )
    })?;
    // Exchange code for token via HTTP
    let executor = neogenealogy_storage::familysearch::ReqwestExecutor::new(cfg.timeout_ms);
    let token = executor
        .fetch_token_authorization_code(
            &cfg.ident_base_url,
            &client_id,
            &cfg.redirect_uri,
            &code,
            cfg.timeout_ms,
        )
        .await
        .map_err(|e| match e.code {
            neogenealogy_storage::external_research::ProviderErrorCode::AUTH_REQUIRED => {
                ApiError::bad_request("AUTH_FAILED", e.message)
            }
            neogenealogy_storage::external_research::ProviderErrorCode::RATE_LIMITED => {
                ApiError::bad_request("RATE_LIMITED", e.message)
            }
            _ => ApiError::internal(e.message),
        })?;
    // Save token
    state
        .storage
        .save_familysearch_token(
            &token.access_token,
            token.expires_in,
            token.token_type.as_deref(),
            None,
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    tracing::info!("FamilySearch OAuth connected, token stored");
    // Redirect to frontend success
    let frontend = cfg.frontend_redirect;
    let url = format!("{}?familysearch=connected", frontend);
    Ok(Redirect::temporary(&url).into_response())
}

/// GET /api/v1/auth/familysearch/status
pub async fn status(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let cfg = neogenealogy_storage::familysearch::FamilySearchConfig::from_env();
    let configured = cfg.client_id.is_some();
    let enabled = cfg.enabled();
    let stored = state
        .storage
        .get_familysearch_token()
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let (connected, expires_at) = if let Some(row) = stored {
        let valid = if let Some(exp) = &row.expires_at {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(exp) {
                dt.with_timezone(&chrono::Utc) > chrono::Utc::now()
            } else {
                true
            }
        } else {
            true
        };
        (valid, row.expires_at)
    } else {
        (false, None)
    };
    // Also consider env access_token as connected
    let env_connected = cfg.access_token.is_some();
    let final_connected = connected || env_connected;
    let status_str = if !enabled {
        "disabled"
    } else if final_connected || configured {
        if final_connected {
            "connected"
        } else {
            "configured"
        }
    } else {
        "not_configured"
    };
    Ok(Json(serde_json::json!({
        "configured": configured,
        "enabled": enabled,
        "connected": final_connected,
        "status": status_str,
        "expires_at": expires_at,
        "has_env_token": env_connected,
        "has_stored_token": connected,
        "redirect_uri": cfg.redirect_uri,
        "requires_auth": !final_connected
    })))
}

/// POST /api/v1/auth/familysearch/disconnect
pub async fn disconnect(State(state): State<AppState>) -> Result<StatusCode, ApiError> {
    state
        .storage
        .delete_familysearch_token()
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    tracing::info!("FamilySearch OAuth disconnected");
    Ok(StatusCode::NO_CONTENT)
}

/// Helper to get effective config with stored token overlay
async fn effective_config(
    storage: &neogenealogy_storage::Storage,
) -> neogenealogy_storage::familysearch::FamilySearchConfig {
    let mut cfg = neogenealogy_storage::familysearch::FamilySearchConfig::from_env();
    if let Ok(Some(row)) = storage.get_familysearch_token().await {
        // Check expiry
        let valid = if let Some(exp) = &row.expires_at {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(exp) {
                dt.with_timezone(&chrono::Utc) > chrono::Utc::now()
            } else {
                true
            }
        } else {
            true
        };
        if valid {
            cfg.access_token = Some(row.access_token);
        }
    }
    cfg
}

/// GET /api/v1/familysearch/search?q=&givenName=&surname=&birthLikeDate=  (global, no tree)
pub async fn familysearch_global_search(
    State(state): State<AppState>,
    Query(params): Query<FamilySearchSearchParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Build free-text query from params
    let free_q = params.q.or(params.query);
    let mut parts: Vec<String> = Vec::new();
    if let Some(g) = params.given_name.clone() {
        if !g.trim().is_empty() {
            parts.push(g.trim().to_string());
        }
    }
    if let Some(s) = params.surname.clone() {
        if !s.trim().is_empty() {
            parts.push(s.trim().to_string());
        }
    }
    if let Some(d) = params.birth_date.clone() {
        if !d.trim().is_empty() {
            parts.push(d.trim().to_string());
        }
    }
    if let Some(p) = params.birth_place.clone() {
        if !p.trim().is_empty() {
            parts.push(p.trim().to_string());
        }
    }
    let query_text = if let Some(fq) = free_q {
        if fq.trim().is_empty() && parts.is_empty() {
            return Err(ApiError::bad_request(
                "INVALID_QUERY",
                "query must not be empty",
            ));
        }
        if parts.is_empty() {
            fq
        } else {
            // Prefer explicit params if provided, else free_q
            parts.join(" ")
        }
    } else if !parts.is_empty() {
        parts.join(" ")
    } else {
        return Err(ApiError::bad_request(
            "INVALID_QUERY",
            "query must not be empty",
        ));
    };

    let cfg = effective_config(&state.storage).await;
    if !cfg.enabled() {
        return Err(ApiError::bad_request(
            "PROVIDER_DISABLED",
            "FamilySearch provider is disabled",
        ));
    }
    if !cfg.is_configured() {
        // Check if stored token exists (effective_config would have it)
        // If still not configured, return AUTH_REQUIRED
        return Err(ApiError::bad_request(
            "FAMILYSEARCH_NOT_CONFIGURED",
            "FamilySearch is not configured. Set NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID or connect via /api/v1/auth/familysearch/authorize",
        ));
    }

    // Use provider directly without tree
    let provider = neogenealogy_storage::familysearch::FamilySearchProvider::new(cfg);
    let resp = provider.search(&query_text).await.map_err(|e| {
        let code = e.code.as_str();
        let msg = e.message.clone();
        match e.code {
            neogenealogy_storage::external_research::ProviderErrorCode::AUTH_REQUIRED => {
                ApiError::bad_request(code, msg)
            }
            neogenealogy_storage::external_research::ProviderErrorCode::INVALID_QUERY => {
                ApiError::bad_request(code, msg)
            }
            neogenealogy_storage::external_research::ProviderErrorCode::RATE_LIMITED => {
                ApiError::bad_request(code, msg)
            }
            neogenealogy_storage::external_research::ProviderErrorCode::TIMEOUT => {
                ApiError::bad_request(code, msg)
            }
            _ => ApiError::internal(msg),
        }
    })?;

    // Return normalized results directly, without persisting ResearchResult (global search)
    // Keep same shape as research_results but without execution_id/query_id persistence
    let provider_name = resp.provider.clone();
    let items: Vec<serde_json::Value> = resp
        .results
        .iter()
        .enumerate()
        .map(|(idx, r)| {
            serde_json::json!({
                "provider": provider_name,
                "external_id": r.external_id,
                "title": r.title,
                "description": r.description,
                "url": r.url,
                "record_type": r.record_type,
                "date": r.date,
                "place": r.place,
                "metadata": r.metadata,
                "position": idx
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "provider": "familysearch",
        "query": query_text,
        "provider_request_id": resp.provider_request_id,
        "provider_metadata": resp.provider_metadata,
        "results": items,
        "result_count": items.len(),
        "disclaimer": "FamilySearch Result ≠ Evidence — Global search without tree, not persisted as ResearchResult unless created via task"
    })))
}
