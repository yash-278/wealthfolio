use crate::{
    error::{ApiError, ApiResult},
    main_lib::AppState,
    mcp::auth::{generate_token, hash_token, token_prefix},
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use subtle::ConstantTimeEq;
use wealthfolio_storage_sqlite::agent::{NewPersonalAccessToken, PersonalAccessTokenDB};

const SCOPES: &str = "[\"captures:submit\",\"captures:read-own\"]";
#[derive(Clone)]
pub struct CapturePrincipal(pub Option<String>);

pub async fn authenticate(
    State(state): State<Arc<AppState>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    if let Some(token) = token.filter(|t| t.starts_with("wfp_")) {
        let Some(prefix) = token_prefix(token) else {
            return StatusCode::UNAUTHORIZED.into_response();
        };
        let hash = hash_token(token);
        let tokens = match state.pat_repository.find_by_prefix(prefix) {
            Ok(tokens) => tokens,
            Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
        };
        let found = tokens.into_iter().find(|row| {
            row.scopes_json == SCOPES
                && row.revoked_at.is_none()
                && row.expires_at.as_ref().is_none_or(|d| {
                    chrono::DateTime::parse_from_rfc3339(d).is_ok_and(|d| d > chrono::Utc::now())
                })
                && bool::from(hash.as_bytes().ct_eq(row.token_hash.as_bytes()))
        });
        let Some(found) = found else {
            return StatusCode::UNAUTHORIZED.into_response();
        };
        let _ = state.pat_repository.touch_last_used(&found.id).await;
        request
            .extensions_mut()
            .insert(CapturePrincipal(Some(format!("token:{}", found.id))));
        return next.run(request).await;
    }
    request.extensions_mut().insert(CapturePrincipal(None));
    match crate::auth::require_jwt(State(state), request, next).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenInfo {
    id: String,
    name: String,
    created_at: String,
    expires_at: Option<String>,
    last_used_at: Option<String>,
}
impl From<PersonalAccessTokenDB> for TokenInfo {
    fn from(row: PersonalAccessTokenDB) -> Self {
        Self {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
            expires_at: row.expires_at,
            last_used_at: row.last_used_at,
        }
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateToken {
    name: String,
    expires_at: Option<String>,
}
#[derive(Serialize)]
struct CreatedToken {
    #[serde(flatten)]
    info: TokenInfo,
    token: String,
}
async fn list(State(state): State<Arc<AppState>>) -> ApiResult<Json<Vec<TokenInfo>>> {
    Ok(Json(
        state
            .pat_repository
            .list()?
            .into_iter()
            .filter(|r| r.scopes_json == SCOPES)
            .map(Into::into)
            .collect(),
    ))
}
async fn create(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateToken>,
) -> ApiResult<Json<CreatedToken>> {
    if body.name.trim().is_empty() || body.name.len() > 128 {
        return Err(ApiError::BadRequest(
            "Token name must contain 1 to 128 characters".into(),
        ));
    }
    if let Some(ref expiration) = body.expires_at {
        if !chrono::DateTime::parse_from_rfc3339(expiration).is_ok_and(|d| d > chrono::Utc::now()) {
            return Err(ApiError::BadRequest(
                "Expiration must be a future RFC3339 time".into(),
            ));
        }
    }
    let token = generate_token();
    let row = state
        .pat_repository
        .create(NewPersonalAccessToken {
            name: body.name.trim().into(),
            token_prefix: token_prefix(&token)
                .ok_or_else(|| ApiError::Internal("Cannot generate token".into()))?
                .into(),
            token_hash: hash_token(&token),
            scopes_json: SCOPES.into(),
            expires_at: body.expires_at,
        })
        .await?;
    Ok(Json(CreatedToken {
        info: row.into(),
        token,
    }))
}
async fn revoke(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> ApiResult<Json<()>> {
    if !state
        .pat_repository
        .list()?
        .iter()
        .any(|r| r.id == id && r.scopes_json == SCOPES)
    {
        return Err(ApiError::NotFound);
    }
    state.pat_repository.delete(&id).await?;
    Ok(Json(()))
}
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/quick-add/tokens", get(list).post(create))
        .route("/quick-add/tokens/{id}", axum::routing::delete(revoke))
}
