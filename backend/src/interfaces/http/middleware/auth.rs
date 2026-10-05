//! `AuthUser` — an Axum extractor that verifies the `Authorization: Bearer <jwt>` header
//! and yields the caller's identity/role. Handlers that need auth simply add `AuthUser`
//! as a parameter; handlers that don't, omit it. Role checks are done explicitly in each
//! handler via `AuthUser::require_role`, keeping authorization visible at the call site.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use uuid::Uuid;

use crate::domain::user::Role;
use crate::error::AppError;
use crate::interfaces::http::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: Role,
}

impl AuthUser {
    pub fn require_role(&self, allowed: &[Role]) -> Result<(), AppError> {
        if allowed.contains(&self.role) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "role '{}' is not permitted to perform this action",
                self.role.as_str()
            )))
        }
    }
}

// `FromRequestParts` is declared in axum-core via the `#[async_trait]` macro (rustc's
// E0195 diagnostics just render the pre-expansion `async fn` source for readability).
// The impl has to go through the *same* macro — `axum::async_trait` is axum's own
// re-export of it — or the desugared `Pin<Box<dyn Future>>` signatures won't line up.
#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized("missing or malformed Authorization header".to_string()))?;

        let claims = state.jwt.verify(bearer.token())?;
        let role = claims
            .role
            .parse::<Role>()
            .map_err(|e| AppError::Unauthorized(format!("invalid role in token: {e}")))?;

        Ok(AuthUser {
            user_id: claims.sub,
            email: claims.email,
            role,
        })
    }
}
