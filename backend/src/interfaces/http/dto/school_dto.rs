use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::application::school_portal_service::{PtnDistributionItem, RecentActivityItem, SchoolOverview, StudentRosterStat};
use crate::domain::school::{School, SchoolStatus};

#[derive(Debug, Serialize)]
pub struct SchoolOverviewResponse {
    pub total_students: i64,
    pub avg_score: Option<f64>,
    pub tryouts_completed: i64,
    pub target_ptn_count: i64,
}

impl From<SchoolOverview> for SchoolOverviewResponse {
    fn from(o: SchoolOverview) -> Self {
        Self {
            total_students: o.total_students,
            avg_score: o.avg_score.map(|s| (s * 10.0).round() / 10.0),
            tryouts_completed: o.tryouts_completed,
            target_ptn_count: o.target_ptn_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StudentRosterStatResponse {
    pub student_id: Uuid,
    pub best_score: Option<i32>,
    pub avg_score: Option<f64>,
    pub tryouts_completed: i64,
    pub target_ptn_count: i64,
    pub last_attempt_at: Option<DateTime<Utc>>,
}

impl From<StudentRosterStat> for StudentRosterStatResponse {
    fn from(s: StudentRosterStat) -> Self {
        Self {
            student_id: s.student_id,
            best_score: s.best_score,
            avg_score: s.avg_score.map(|v| (v * 10.0).round() / 10.0),
            tryouts_completed: s.tryouts_completed,
            target_ptn_count: s.target_ptn_count,
            last_attempt_at: s.last_attempt_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PtnDistributionItemResponse {
    pub nama_ptn: String,
    pub count: i64,
}

impl From<PtnDistributionItem> for PtnDistributionItemResponse {
    fn from(p: PtnDistributionItem) -> Self {
        Self { nama_ptn: p.nama_ptn, count: p.count }
    }
}

#[derive(Debug, Serialize)]
pub struct RecentActivityItemResponse {
    pub student_name: String,
    pub session_title: String,
    pub session_type: String,
    pub score: Option<i32>,
    pub submitted_at: DateTime<Utc>,
}

impl From<RecentActivityItem> for RecentActivityItemResponse {
    fn from(a: RecentActivityItem) -> Self {
        Self {
            student_name: a.student_name,
            session_title: a.session_title,
            session_type: crate::interfaces::http::dto::tryout_dto::session_type_str(a.session_type).to_string(),
            score: a.score,
            submitted_at: a.submitted_at,
        }
    }
}

fn default_school_status() -> String {
    "active".to_string()
}
fn default_revenue_share() -> i32 {
    10
}

#[derive(Debug, Deserialize, Validate)]
pub struct SchoolPayload {
    #[validate(length(min = 1, message = "school name is required"))]
    pub name: String,
    /// "smp" | "sma" | "smk" | "ma"
    pub school_type: String,
    #[validate(length(min = 1))]
    pub city: String,
    #[validate(length(min = 1))]
    pub province: String,
    #[validate(email(message = "must be a valid email"))]
    pub email: String,
    #[validate(length(min = 1))]
    pub phone: String,
    /// "basic" | "premium" | "enterprise"
    pub package_type: String,
    #[validate(length(min = 1, message = "contact person is required"))]
    pub contact_person: String,
    /// "active" | "inactive" | "pending". Defaults to "active" if omitted.
    #[serde(default = "default_school_status")]
    pub status: String,
    /// Percentage (0-50) of the contracted plan value paid back to the school.
    #[serde(default = "default_revenue_share")]
    pub revenue_share: i32,
    /// Contracted plan value in Rupiah, admin-recorded.
    #[serde(default)]
    pub monthly_revenue: i64,
}

impl SchoolPayload {
    pub fn into_create_input(
        self,
    ) -> Result<crate::application::school_service::CreateSchoolInput, crate::error::AppError> {
        Ok(crate::application::school_service::CreateSchoolInput {
            name: self.name,
            school_type: self.school_type.parse().map_err(crate::error::AppError::Validation)?,
            city: self.city,
            province: self.province,
            email: self.email,
            phone: self.phone,
            package_type: self.package_type.parse().map_err(crate::error::AppError::Validation)?,
            contact_person: self.contact_person,
            status: self.status.parse().map_err(crate::error::AppError::Validation)?,
            revenue_share: self.revenue_share,
            monthly_revenue: self.monthly_revenue,
        })
    }

    pub fn into_update_input(
        self,
    ) -> Result<crate::application::school_service::UpdateSchoolInput, crate::error::AppError> {
        Ok(crate::application::school_service::UpdateSchoolInput {
            name: self.name,
            school_type: self.school_type.parse().map_err(crate::error::AppError::Validation)?,
            city: self.city,
            province: self.province,
            email: self.email,
            phone: self.phone,
            package_type: self.package_type.parse().map_err(crate::error::AppError::Validation)?,
            contact_person: self.contact_person,
            status: self.status.parse().map_err(crate::error::AppError::Validation)?,
            revenue_share: self.revenue_share,
            monthly_revenue: self.monthly_revenue,
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct SchoolStatusPayload {
    pub status: String,
}

impl SchoolStatusPayload {
    pub fn into_status(self) -> Result<SchoolStatus, crate::error::AppError> {
        self.status.parse().map_err(crate::error::AppError::Validation)
    }
}

#[derive(Debug, Serialize)]
pub struct SchoolResponse {
    pub id: Uuid,
    pub name: String,
    pub school_type: String,
    pub city: String,
    pub province: String,
    pub email: String,
    pub phone: String,
    pub join_date: NaiveDate,
    pub package_type: String,
    pub contact_person: String,
    pub status: String,
    pub revenue_share: i32,
    pub monthly_revenue: i64,
    pub created_at: DateTime<Utc>,
}

impl From<School> for SchoolResponse {
    fn from(s: School) -> Self {
        Self {
            id: s.id,
            name: s.name,
            school_type: s.school_type.as_str().to_string(),
            city: s.city,
            province: s.province,
            email: s.email,
            phone: s.phone,
            join_date: s.join_date,
            package_type: s.package_type.as_str().to_string(),
            contact_person: s.contact_person,
            status: s.status.as_str().to_string(),
            revenue_share: s.revenue_share,
            monthly_revenue: s.monthly_revenue,
            created_at: s.created_at,
        }
    }
}
