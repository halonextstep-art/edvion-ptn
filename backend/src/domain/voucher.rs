//! Voucher entity — access/bulk-school/promo/referral discount codes (Manajemen Voucher),
//! mirroring the reference `AdminVoucher`. `effective_status` is derived (never stored):
//! a voucher is "expired" once past `expires_at` and "redeemed" once `used_count` reaches
//! `max_uses`, regardless of the admin-set `active` flag — so the displayed status can
//! never drift from the underlying facts. `used_count` is always 0 in this build: no
//! checkout/redemption flow exists anywhere in the codebase yet, so nothing ever
//! increments it — it is real (unfabricated) infrastructure waiting for that flow.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoucherType {
    Access,
    BulkSchool,
    Promo,
    Referral,
}

impl VoucherType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VoucherType::Access => "access",
            VoucherType::BulkSchool => "bulk_school",
            VoucherType::Promo => "promo",
            VoucherType::Referral => "referral",
        }
    }

    /// Short prefix used when generating a human-readable code.
    pub fn code_prefix(&self) -> &'static str {
        match self {
            VoucherType::Access => "GASPL",
            VoucherType::BulkSchool => "BULK",
            VoucherType::Promo => "PROMO",
            VoucherType::Referral => "REF",
        }
    }
}

impl std::str::FromStr for VoucherType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "access" => Ok(VoucherType::Access),
            "bulk_school" => Ok(VoucherType::BulkSchool),
            "promo" => Ok(VoucherType::Promo),
            "referral" => Ok(VoucherType::Referral),
            other => Err(format!("unknown voucher_type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscountType {
    Percent,
    Fixed,
    Full,
}

impl DiscountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiscountType::Percent => "percent",
            DiscountType::Fixed => "fixed",
            DiscountType::Full => "full",
        }
    }
}

impl std::str::FromStr for DiscountType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "percent" => Ok(DiscountType::Percent),
            "fixed" => Ok(DiscountType::Fixed),
            "full" => Ok(DiscountType::Full),
            other => Err(format!("unknown discount_type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoucherStatus {
    Active,
    Inactive,
    Expired,
    Redeemed,
}

impl VoucherStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            VoucherStatus::Active => "active",
            VoucherStatus::Inactive => "inactive",
            VoucherStatus::Expired => "expired",
            VoucherStatus::Redeemed => "redeemed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voucher {
    pub id: Uuid,
    pub code: String,
    pub voucher_type: VoucherType,
    pub package_id: Uuid,
    pub package_name: String,
    pub discount_type: DiscountType,
    pub discount_value: i32,
    pub max_uses: i32,
    pub used_count: i32,
    pub active: bool,
    pub expires_at: NaiveDate,
    pub note: String,
    pub school_name: Option<String>,
    pub referrer_name: Option<String>,
    pub referrer_commission: Option<i32>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Voucher {
    pub fn effective_status(&self, today: NaiveDate) -> VoucherStatus {
        compute_status(self.active, self.used_count, self.max_uses, self.expires_at, today)
    }
}

pub fn compute_status(active: bool, used_count: i32, max_uses: i32, expires_at: NaiveDate, today: NaiveDate) -> VoucherStatus {
    if expires_at < today {
        return VoucherStatus::Expired;
    }
    if max_uses > 0 && used_count >= max_uses {
        return VoucherStatus::Redeemed;
    }
    if active {
        VoucherStatus::Active
    } else {
        VoucherStatus::Inactive
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn expired_wins_over_everything_else() {
        let status = compute_status(true, 0, 10, d(2025, 1, 1), d(2025, 6, 1));
        assert_eq!(status, VoucherStatus::Expired);
    }

    #[test]
    fn redeemed_when_used_count_reaches_max() {
        let status = compute_status(true, 10, 10, d(2025, 12, 31), d(2025, 6, 1));
        assert_eq!(status, VoucherStatus::Redeemed);
    }

    #[test]
    fn active_when_not_expired_not_redeemed_and_toggled_on() {
        let status = compute_status(true, 3, 10, d(2025, 12, 31), d(2025, 6, 1));
        assert_eq!(status, VoucherStatus::Active);
    }

    #[test]
    fn inactive_when_toggled_off() {
        let status = compute_status(false, 3, 10, d(2025, 12, 31), d(2025, 6, 1));
        assert_eq!(status, VoucherStatus::Inactive);
    }
}
