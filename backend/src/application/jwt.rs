//! JWT issuing/verification. Shared by `AuthService` (issues tokens on login/register)
//! and the HTTP auth middleware (verifies tokens on protected routes).

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::tenant::TenantCtx;
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject = user id (= `core.akun.public_id`).
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    /// RLS session context — carried in the token so every request can set
    /// `app.tenant_id`/`app.is_platform_admin` without a DB round trip first. `0` for
    /// admin/content accounts (see `TenantCtx::PLATFORM_TENANT_ID`).
    pub tenant_id: i64,
    pub is_platform_admin: bool,
    pub exp: usize,
    pub iat: usize,
}

impl Claims {
    pub fn tenant_ctx(&self) -> TenantCtx {
        TenantCtx { tenant_id: self.tenant_id, is_platform_admin: self.is_platform_admin }
    }
}

pub struct JwtService {
    secret: String,
    expiry_hours: i64,
}

impl JwtService {
    pub fn new(secret: String, expiry_hours: i64) -> Self {
        Self { secret, expiry_hours }
    }

    pub fn issue(&self, user_id: Uuid, email: &str, role: Role, ctx: TenantCtx) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiry_hours);
        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            role: role.as_str().to_string(),
            tenant_id: ctx.tenant_id,
            is_platform_admin: ctx.is_platform_admin,
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to issue token: {e}")))
    }

    pub fn verify(&self, token: &str) -> AppResult<Claims> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized("invalid or expired token".to_string()))
    }
}
