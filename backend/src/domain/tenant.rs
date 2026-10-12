//! `TenantCtx` — the RLS session context every Postgres repository method needs before
//! it can safely query a `ptn.*`/`core.*` table. Carried by `AuthUser` (built from JWT
//! claims) and threaded explicitly into repository trait methods, since Postgres RLS reads
//! `current_setting('app.tenant_id')`/`current_setting('app.is_platform_admin')` from the
//! session (actually: the transaction, since we always set them as transaction-local GUCs)
//! rather than anything implicit.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantCtx {
    pub tenant_id: i64,
    pub is_platform_admin: bool,
}

impl TenantCtx {
    /// The reserved tenant id for admin/content accounts not tied to any one school —
    /// never a real school's id (see migration 20260101000001_core_extensions.sql /
    /// `core.tenant` row 0 = "Platform (System)"). RLS bypass for these accounts comes from
    /// `is_platform_admin`, not from this value.
    pub const PLATFORM_TENANT_ID: i64 = 0;

    pub fn platform_admin() -> Self {
        Self { tenant_id: Self::PLATFORM_TENANT_ID, is_platform_admin: true }
    }

    pub fn school(tenant_id: i64) -> Self {
        Self { tenant_id, is_platform_admin: false }
    }

    /// Derives the right ctx for an account given its role and raw `core.akun.tenant_id` —
    /// admin/content accounts are always platform-scoped regardless of what's actually
    /// stored (defensive: `is_platform_admin` should never come from anywhere other than
    /// this derivation at read time, never trusted verbatim from a stored column).
    pub fn for_role(role: crate::domain::user::Role, tenant_id: i64) -> Self {
        if role.is_platform_scoped() {
            Self::platform_admin()
        } else {
            Self::school(tenant_id)
        }
    }
}
