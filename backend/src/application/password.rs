//! Shared password hashing helpers — used by `AuthService` (self-registration/login) and
//! `UserService` (admin-created accounts, password resets) so there is exactly one place
//! that knows how passwords are hashed/verified.
//!
//! New PTN accounts are always hashed with Argon2id (`hash_password`). But `core.akun` is
//! shared with other modules (academic/tahfiz/office/etc. — see `lms-api`), and accounts
//! created there were hashed with bcrypt instead. `verify_password` dispatches on the
//! stored hash's format so those pre-existing accounts can still log into PTN — it never
//! *produces* a bcrypt hash itself, only verifies one it finds.

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

use crate::error::{AppError, AppResult};

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to hash password: {e}")))
}

/// bcrypt hashes always look like `$2a$`/`$2b$`/`$2y$<cost>$<53 more chars>` — distinct
/// enough from Argon2id's `$argon2id$v=19$...` that sniffing the prefix is reliable.
fn is_bcrypt_hash(hash: &str) -> bool {
    hash.starts_with("$2a$") || hash.starts_with("$2b$") || hash.starts_with("$2y$")
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    if is_bcrypt_hash(hash) {
        return bcrypt::verify(password, hash)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("stored bcrypt hash is invalid: {e}")));
    }
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("stored password hash is invalid: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
