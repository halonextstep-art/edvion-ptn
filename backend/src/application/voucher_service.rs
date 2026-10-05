//! Voucher (Manajemen Voucher) CRUD + code-generation use-cases. Admin-only, mirroring
//! the reference `AdminVoucher`. Codes are generated server-side with a real CSPRNG
//! (`rand_core::OsRng`, the same source already used for password salts) — never a
//! predictable/fake sequence. `package_id` is validated against the real package
//! catalogue up front, same pattern as `EventService::ensure_session_exists`.

use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use rand_core::{OsRng, RngCore};
use uuid::Uuid;

use crate::domain::package::Package;
use crate::domain::repository::{NewVoucher, PackageRepository, VoucherRepository};
use crate::domain::user::Role;
use crate::domain::voucher::{DiscountType, Voucher, VoucherStatus, VoucherType};
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

/// Result of a successful redemption — bundles the voucher with the package it unlocks so
/// the student-facing UI can show what they actually got, without a second round trip.
pub struct RedemptionResult {
    pub voucher: Voucher,
    pub package: Package,
}

/// Result of a no-auth, read-only voucher-code check (public landing page "cek voucher"
/// widget) — deliberately a hand-picked subset of `Voucher`'s fields, never the whole
/// struct: `note`, `school_name`, `referrer_name`/`referrer_commission`, and `created_by`
/// are internal/admin-facing and must never leak to an anonymous visitor.
pub struct VoucherCheckResult {
    pub valid: bool,
    pub reason: Option<String>,
    pub package_name: Option<String>,
    pub discount_type: Option<DiscountType>,
    pub discount_value: Option<i32>,
    pub expires_at: Option<NaiveDate>,
}

pub struct CreateVoucherInput {
    pub voucher_type: VoucherType,
    pub package_id: Uuid,
    pub discount_type: DiscountType,
    pub discount_value: i32,
    pub max_uses: i32,
    pub expires_at: NaiveDate,
    pub note: String,
    pub school_name: Option<String>,
    pub referrer_name: Option<String>,
    pub referrer_commission: Option<i32>,
    /// How many codes to generate in one go (bulk-school / access batches). 1 for
    /// promo/referral, which are always a single shared/personal code.
    pub quantity: i32,
}

pub struct VoucherService {
    vouchers: Arc<dyn VoucherRepository>,
    packages: Arc<dyn PackageRepository>,
}

impl VoucherService {
    pub fn new(vouchers: Arc<dyn VoucherRepository>, packages: Arc<dyn PackageRepository>) -> Self {
        Self { vouchers, packages }
    }

    pub async fn create(&self, actor: &AuthUser, input: CreateVoucherInput) -> AppResult<Vec<Voucher>> {
        actor.require_role(&[Role::Admin])?;
        self.ensure_package_exists(input.package_id).await?;
        if input.discount_type == DiscountType::Percent && !(0..=100).contains(&input.discount_value) {
            return Err(AppError::Validation("diskon persen harus antara 0-100".to_string()));
        }
        let qty = input.quantity.clamp(1, 500);

        let mut batch = Vec::with_capacity(qty as usize);
        for _ in 0..qty {
            batch.push(NewVoucher {
                code: generate_code(input.voucher_type),
                voucher_type: input.voucher_type,
                package_id: input.package_id,
                discount_type: input.discount_type,
                discount_value: input.discount_value,
                max_uses: if matches!(input.voucher_type, VoucherType::Access) { 1 } else { input.max_uses.max(1) },
                expires_at: input.expires_at,
                note: input.note.clone(),
                school_name: input.school_name.clone(),
                referrer_name: input.referrer_name.clone(),
                referrer_commission: input.referrer_commission,
                created_by: actor.user_id,
            });
        }

        if batch.len() == 1 {
            Ok(vec![self.vouchers.create(batch.into_iter().next().unwrap()).await?])
        } else {
            self.vouchers.create_batch(batch).await
        }
    }

    pub async fn list(&self, actor: &AuthUser) -> AppResult<Vec<Voucher>> {
        actor.require_role(&[Role::Admin])?;
        self.vouchers.list().await
    }

    pub async fn get(&self, actor: &AuthUser, id: Uuid) -> AppResult<Voucher> {
        actor.require_role(&[Role::Admin])?;
        self.vouchers
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("voucher {id} not found")))
    }

    pub async fn set_active(&self, actor: &AuthUser, id: Uuid, active: bool) -> AppResult<Voucher> {
        actor.require_role(&[Role::Admin])?;
        self.vouchers
            .set_active(id, active)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("voucher {id} not found")))
    }

    pub async fn delete(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        let v = self
            .vouchers
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("voucher {id} not found")))?;
        if v.used_count > 0 {
            return Err(AppError::Conflict("voucher yang sudah dipakai tidak bisa dihapus".to_string()));
        }
        self.vouchers.delete(id).await?;
        Ok(())
    }

    /// Student-facing redemption. Validates the code is genuinely usable right now (not
    /// expired, not exhausted, not deactivated by admin) using the same
    /// `Voucher::effective_status` the admin UI displays — no separate/duplicated rule set —
    /// then delegates the atomic increment+audit-row insert to the repository.
    pub async fn redeem(&self, actor: &AuthUser, code: &str) -> AppResult<RedemptionResult> {
        actor.require_role(&[Role::Student])?;
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("kode voucher wajib diisi".to_string()));
        }
        let voucher = self
            .vouchers
            .find_by_code(trimmed)
            .await?
            .ok_or_else(|| AppError::NotFound("kode voucher tidak ditemukan".to_string()))?;

        match voucher.effective_status(Utc::now().date_naive()) {
            VoucherStatus::Active => {}
            VoucherStatus::Expired => return Err(AppError::Validation("voucher ini sudah kedaluwarsa".to_string())),
            VoucherStatus::Redeemed => return Err(AppError::Validation("voucher ini sudah mencapai batas penggunaan".to_string())),
            VoucherStatus::Inactive => return Err(AppError::Validation("voucher ini sedang tidak aktif".to_string())),
        }

        let updated = self.vouchers.redeem(voucher.id, actor.user_id).await?;
        let package = self
            .packages
            .find_by_id(updated.package_id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("package referenced by voucher vanished")))?;
        Ok(RedemptionResult { voucher: updated, package })
    }

    /// No-auth, read-only check for the public landing page's "Punya Kode Voucher?"
    /// widget — mirrors `redeem()`'s validity logic exactly (same `effective_status`) but
    /// never mutates anything, so a visitor can check a code before creating an account.
    pub async fn check_code(&self, code: &str) -> AppResult<VoucherCheckResult> {
        let empty = || VoucherCheckResult {
            valid: false,
            reason: None,
            package_name: None,
            discount_type: None,
            discount_value: None,
            expires_at: None,
        };
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return Ok(VoucherCheckResult { reason: Some("kode voucher wajib diisi".to_string()), ..empty() });
        }
        let Some(voucher) = self.vouchers.find_by_code(trimmed).await? else {
            return Ok(VoucherCheckResult { reason: Some("kode voucher tidak ditemukan".to_string()), ..empty() });
        };

        match voucher.effective_status(Utc::now().date_naive()) {
            VoucherStatus::Active => Ok(VoucherCheckResult {
                valid: true,
                reason: None,
                package_name: Some(voucher.package_name),
                discount_type: Some(voucher.discount_type),
                discount_value: Some(voucher.discount_value),
                expires_at: Some(voucher.expires_at),
            }),
            VoucherStatus::Expired => {
                Ok(VoucherCheckResult { reason: Some("voucher ini sudah kedaluwarsa".to_string()), ..empty() })
            }
            VoucherStatus::Redeemed => {
                Ok(VoucherCheckResult { reason: Some("voucher ini sudah mencapai batas penggunaan".to_string()), ..empty() })
            }
            VoucherStatus::Inactive => {
                Ok(VoucherCheckResult { reason: Some("voucher ini sedang tidak aktif".to_string()), ..empty() })
            }
        }
    }

    /// Used by `PaymentService::fulfill` when an admin confirms a manual purchase request —
    /// generates a real single-use Access voucher and immediately redeems it for the buyer,
    /// so the student doesn't have to separately type in a code. No role check here: the
    /// caller (`PaymentService::fulfill`) already requires `Role::Admin` before calling this.
    pub async fn grant_access(&self, package_id: Uuid, buyer_user_id: Uuid, created_by: Uuid, note: String) -> AppResult<Voucher> {
        self.ensure_package_exists(package_id).await?;
        let voucher = self
            .vouchers
            .create(NewVoucher {
                code: generate_code(VoucherType::Access),
                voucher_type: VoucherType::Access,
                package_id,
                discount_type: DiscountType::Full,
                discount_value: 100,
                max_uses: 1,
                expires_at: (Utc::now() + chrono::Duration::days(365)).date_naive(),
                note,
                school_name: None,
                referrer_name: None,
                referrer_commission: None,
                created_by,
            })
            .await?;
        self.vouchers.redeem(voucher.id, buyer_user_id).await
    }

    async fn ensure_package_exists(&self, package_id: Uuid) -> AppResult<()> {
        self.packages
            .find_by_id(package_id)
            .await?
            .ok_or_else(|| AppError::Validation("paket yang dipilih tidak ditemukan".to_string()))?;
        Ok(())
    }
}

/// `PREFIX-XXXX-XXXX` using an unambiguous alphabet (no 0/O/1/I) and a real CSPRNG.
fn generate_code(voucher_type: VoucherType) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = OsRng;
    let mut segment = || -> String {
        (0..4)
            .map(|_| {
                let idx = (rng.next_u32() as usize) % ALPHABET.len();
                ALPHABET[idx] as char
            })
            .collect()
    };
    format!("{}-{}-{}", voucher_type.code_prefix(), segment(), segment())
}
