//! Finance entities (Manajemen Keuangan) — a read-only aggregate dashboard, deliberately
//! different in shape from the reference `AdminFinance` (which fabricates a full
//! transaction/payout ledger with no backing payment system). This codebase has no
//! payment gateway or checkout flow at all, so instead of inventing fake transactions we
//! surface the real revenue-shaped data that already exists: `Event.price *
//! Event.participants` (both already real — participants is itself derived live from
//! `attempts`), `School.revenue_share`/`monthly_revenue` (real admin-set business
//! fields), and the `Package` pricing catalogue. Every number here is either a direct
//! field from an existing entity or a straightforward sum/average over them — nothing is
//! stored, nothing is simulated.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceSummary {
    /// Sum of `price * participants` across all events — an honest *estimate* (not an
    /// actual recorded payment), clearly labeled as such by the DTO/UI layer.
    pub total_event_revenue_estimate: i64,
    pub paid_events_count: i64,
    pub total_paid_participants: i64,
    pub school_count: i64,
    /// Average of `School.revenue_share` across all schools (0 if there are none).
    pub avg_revenue_share_percent: f64,
    /// Sum of `School.monthly_revenue` — stays honestly 0 until a real billing system
    /// exists to populate it (same field already used by School Management).
    pub total_monthly_revenue: i64,
    pub active_package_count: i64,
    /// Sum of `Package.sale_price` for active packages — the catalogue's total list
    /// value, not a sales figure (no purchase system exists to produce one).
    pub package_catalog_value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRevenueBreakdown {
    pub event_id: Uuid,
    pub event_name: String,
    pub price: i32,
    pub participants: i64,
    pub revenue_estimate: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolRevenueBreakdown {
    pub school_id: Uuid,
    pub school_name: String,
    pub package_type: String,
    pub revenue_share_percent: i32,
    pub monthly_revenue: i64,
}
