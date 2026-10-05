use serde::Serialize;
use uuid::Uuid;

use crate::domain::finance::{EventRevenueBreakdown, FinanceSummary, SchoolRevenueBreakdown};

#[derive(Debug, Serialize)]
pub struct FinanceSummaryResponse {
    pub total_event_revenue_estimate: i64,
    pub paid_events_count: i64,
    pub total_paid_participants: i64,
    pub school_count: i64,
    pub avg_revenue_share_percent: f64,
    pub total_monthly_revenue: i64,
    pub active_package_count: i64,
    pub package_catalog_value: i64,
}
impl From<FinanceSummary> for FinanceSummaryResponse {
    fn from(s: FinanceSummary) -> Self {
        Self {
            total_event_revenue_estimate: s.total_event_revenue_estimate,
            paid_events_count: s.paid_events_count,
            total_paid_participants: s.total_paid_participants,
            school_count: s.school_count,
            avg_revenue_share_percent: s.avg_revenue_share_percent,
            total_monthly_revenue: s.total_monthly_revenue,
            active_package_count: s.active_package_count,
            package_catalog_value: s.package_catalog_value,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EventRevenueResponse {
    pub event_id: Uuid,
    pub event_name: String,
    pub price: i32,
    pub participants: i64,
    pub revenue_estimate: i64,
}
impl From<EventRevenueBreakdown> for EventRevenueResponse {
    fn from(e: EventRevenueBreakdown) -> Self {
        Self { event_id: e.event_id, event_name: e.event_name, price: e.price, participants: e.participants, revenue_estimate: e.revenue_estimate }
    }
}

#[derive(Debug, Serialize)]
pub struct SchoolRevenueResponse {
    pub school_id: Uuid,
    pub school_name: String,
    pub package_type: String,
    pub revenue_share_percent: i32,
    pub monthly_revenue: i64,
}
impl From<SchoolRevenueBreakdown> for SchoolRevenueResponse {
    fn from(s: SchoolRevenueBreakdown) -> Self {
        Self { school_id: s.school_id, school_name: s.school_name, package_type: s.package_type, revenue_share_percent: s.revenue_share_percent, monthly_revenue: s.monthly_revenue }
    }
}
