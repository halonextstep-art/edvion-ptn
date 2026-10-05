//! Finance (Manajemen Keuangan) use-cases — a read-only aggregate dashboard, reusing the
//! same `EventRepository` / `SchoolRepository` / `PackageRepository` traits already
//! backing those modules. There is no separate Finance table: this service is pure
//! aggregation over data that already exists and is already trustworthy (Event revenue
//! estimates already power `EventManager`'s stat cards; School revenue share already
//! powers `SchoolManager`'s detail drawer). See `domain::finance` for why this
//! deliberately does not attempt to reproduce the reference's fabricated transaction ledger.

use std::sync::Arc;

use crate::domain::finance::{EventRevenueBreakdown, FinanceSummary, SchoolRevenueBreakdown};
use crate::domain::repository::{EventRepository, PackageRepository, SchoolRepository};
use crate::domain::user::Role;
use crate::error::AppResult;
use crate::interfaces::http::middleware::AuthUser;

pub struct FinanceService {
    events: Arc<dyn EventRepository>,
    schools: Arc<dyn SchoolRepository>,
    packages: Arc<dyn PackageRepository>,
}

impl FinanceService {
    pub fn new(events: Arc<dyn EventRepository>, schools: Arc<dyn SchoolRepository>, packages: Arc<dyn PackageRepository>) -> Self {
        Self { events, schools, packages }
    }

    pub async fn summary(&self, actor: &AuthUser) -> AppResult<FinanceSummary> {
        actor.require_role(&[Role::Admin])?;
        let events = self.events.list().await?;
        let schools = self.schools.list().await?;
        let packages = self.packages.list().await?;

        let paid_events: Vec<_> = events.iter().filter(|e| e.price > 0).collect();
        let total_event_revenue_estimate: i64 = paid_events.iter().map(|e| e.price as i64 * e.participants).sum();
        let total_paid_participants: i64 = paid_events.iter().map(|e| e.participants).sum();

        let school_count = schools.len() as i64;
        let avg_revenue_share_percent = if schools.is_empty() {
            0.0
        } else {
            schools.iter().map(|s| s.revenue_share as f64).sum::<f64>() / schools.len() as f64
        };
        let total_monthly_revenue: i64 = schools.iter().map(|s| s.monthly_revenue).sum();

        let active_packages: Vec<_> = packages.iter().filter(|p| p.active).collect();
        let package_catalog_value: i64 = active_packages.iter().map(|p| p.sale_price as i64).sum();

        Ok(FinanceSummary {
            total_event_revenue_estimate,
            paid_events_count: paid_events.len() as i64,
            total_paid_participants,
            school_count,
            avg_revenue_share_percent,
            total_monthly_revenue,
            active_package_count: active_packages.len() as i64,
            package_catalog_value,
        })
    }

    pub async fn event_breakdown(&self, actor: &AuthUser) -> AppResult<Vec<EventRevenueBreakdown>> {
        actor.require_role(&[Role::Admin])?;
        let mut events = self.events.list().await?;
        events.sort_by(|a, b| (b.price as i64 * b.participants).cmp(&(a.price as i64 * a.participants)));
        Ok(events
            .into_iter()
            .filter(|e| e.price > 0)
            .map(|e| EventRevenueBreakdown {
                event_id: e.id,
                event_name: e.name,
                price: e.price,
                participants: e.participants,
                revenue_estimate: e.price as i64 * e.participants,
            })
            .collect())
    }

    pub async fn school_breakdown(&self, actor: &AuthUser) -> AppResult<Vec<SchoolRevenueBreakdown>> {
        actor.require_role(&[Role::Admin])?;
        let mut schools = self.schools.list().await?;
        schools.sort_by(|a, b| b.monthly_revenue.cmp(&a.monthly_revenue));
        Ok(schools
            .into_iter()
            .map(|s| SchoolRevenueBreakdown {
                school_id: s.id,
                school_name: s.name,
                package_type: s.package_type.as_str().to_string(),
                revenue_share_percent: s.revenue_share,
                monthly_revenue: s.monthly_revenue,
            })
            .collect())
    }
}
