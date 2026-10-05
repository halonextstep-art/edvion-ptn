use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::application::voucher_service::RedemptionResult;
use crate::domain::package::Package;
use crate::domain::voucher::Voucher;

#[derive(Debug, Deserialize, Validate)]
pub struct VoucherPayload {
    /// "access" | "bulk_school" | "promo" | "referral"
    pub voucher_type: String,
    pub package_id: Uuid,
    /// "percent" | "fixed" | "full"
    pub discount_type: String,
    #[serde(default)]
    pub discount_value: i32,
    #[serde(default = "default_max_uses")]
    pub max_uses: i32,
    pub expires_at: NaiveDate,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub school_name: Option<String>,
    #[serde(default)]
    pub referrer_name: Option<String>,
    #[serde(default)]
    pub referrer_commission: Option<i32>,
    #[serde(default = "default_quantity")]
    pub quantity: i32,
}

fn default_max_uses() -> i32 {
    1
}
fn default_quantity() -> i32 {
    1
}

impl VoucherPayload {
    pub fn into_input(self) -> Result<crate::application::voucher_service::CreateVoucherInput, crate::error::AppError> {
        Ok(crate::application::voucher_service::CreateVoucherInput {
            voucher_type: self.voucher_type.parse().map_err(crate::error::AppError::Validation)?,
            package_id: self.package_id,
            discount_type: self.discount_type.parse().map_err(crate::error::AppError::Validation)?,
            discount_value: self.discount_value,
            max_uses: self.max_uses,
            expires_at: self.expires_at,
            note: self.note,
            school_name: self.school_name.filter(|s| !s.trim().is_empty()),
            referrer_name: self.referrer_name.filter(|s| !s.trim().is_empty()),
            referrer_commission: self.referrer_commission,
            quantity: self.quantity,
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct VoucherActivePayload {
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct VoucherResponse {
    pub id: Uuid,
    pub code: String,
    pub voucher_type: String,
    pub package_id: Uuid,
    pub package_name: String,
    pub discount_type: String,
    pub discount_value: i32,
    pub max_uses: i32,
    /// Real, live count — incremented by the student redemption flow (see
    /// `VoucherService::redeem` / `POST /api/vouchers/redeem`), never fabricated.
    pub used_count: i32,
    pub active: bool,
    /// Derived status combining `active` + expiry date + used_count, never stored directly.
    pub status: String,
    pub expires_at: NaiveDate,
    pub note: String,
    pub school_name: Option<String>,
    pub referrer_name: Option<String>,
    pub referrer_commission: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl From<Voucher> for VoucherResponse {
    fn from(v: Voucher) -> Self {
        let status = v.effective_status(Utc::now().date_naive()).as_str().to_string();
        Self {
            id: v.id,
            code: v.code,
            voucher_type: v.voucher_type.as_str().to_string(),
            package_id: v.package_id,
            package_name: v.package_name,
            discount_type: v.discount_type.as_str().to_string(),
            discount_value: v.discount_value,
            max_uses: v.max_uses,
            used_count: v.used_count,
            active: v.active,
            status,
            expires_at: v.expires_at,
            note: v.note,
            school_name: v.school_name,
            referrer_name: v.referrer_name,
            referrer_commission: v.referrer_commission,
            created_at: v.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct RedeemPayload {
    #[validate(length(min = 1, message = "kode voucher wajib diisi"))]
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct RedemptionResponse {
    pub voucher_code: String,
    pub package_id: Uuid,
    pub package_name: String,
    pub discount_type: String,
    pub discount_value: i32,
}

impl From<RedemptionResult> for RedemptionResponse {
    fn from(r: RedemptionResult) -> Self {
        let package: Package = r.package;
        Self {
            voucher_code: r.voucher.code,
            package_id: package.id,
            package_name: package.name,
            discount_type: r.voucher.discount_type.as_str().to_string(),
            discount_value: r.voucher.discount_value,
        }
    }
}
