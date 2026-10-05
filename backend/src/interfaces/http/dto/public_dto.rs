//! Response shapes for the no-auth `/api/public/*` routes that back the marketing landing
//! page. Deliberately hand-picked/trimmed (never a raw domain struct dump) so nothing
//! internal ever leaks to an anonymous visitor — see the doc comments on
//! `VoucherService::check_code` and `PackageService::list_public` for what's excluded and
//! why.

use chrono::NaiveDate;
use serde::Serialize;

use crate::application::voucher_service::VoucherCheckResult;
use crate::domain::analytics::AnalyticsSummary;

#[derive(Debug, Serialize)]
pub struct PublicStatsResponse {
    pub total_students: i64,
    pub total_schools: i64,
    pub total_questions: i64,
    pub tryouts_completed: i64,
    pub average_score: f64,
}

impl From<AnalyticsSummary> for PublicStatsResponse {
    fn from(s: AnalyticsSummary) -> Self {
        Self {
            total_students: s.total_students,
            total_schools: s.total_schools,
            total_questions: s.total_questions,
            tryouts_completed: s.submitted_attempts,
            average_score: (s.average_score * 10.0).round() / 10.0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PublicVoucherCheckResponse {
    pub valid: bool,
    pub reason: Option<String>,
    pub package_name: Option<String>,
    /// "percent" | "fixed" | "full"
    pub discount_type: Option<String>,
    pub discount_value: Option<i32>,
    pub expires_at: Option<NaiveDate>,
}

impl From<VoucherCheckResult> for PublicVoucherCheckResponse {
    fn from(r: VoucherCheckResult) -> Self {
        Self {
            valid: r.valid,
            reason: r.reason,
            package_name: r.package_name,
            discount_type: r.discount_type.map(|d| d.as_str().to_string()),
            discount_value: r.discount_value,
            expires_at: r.expires_at,
        }
    }
}
