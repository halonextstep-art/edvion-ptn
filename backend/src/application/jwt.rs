//! JWT issuing/verification. Shared by `AuthService` (issues tokens on login/register)
//! and the HTTP auth middleware (verifies tokens on protected routes).

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::user::Role;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject = user id.
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtService {
    secret: String,
    expiry_hours: i64,
}

impl JwtService {
    pub fn new(secret: String, expiry_hours: i64) -> Self {
        Self { secret, expiry_hours }
    }

    pub fn issue(&self, user_id: Uuid, email: &str, role: Role) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiry_hours);
        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            role: role.as_str().to_string(),
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
