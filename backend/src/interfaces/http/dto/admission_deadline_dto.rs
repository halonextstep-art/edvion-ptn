use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::admission_deadline::AdmissionDeadline;

#[derive(Debug, Deserialize, Validate)]
pub struct AdmissionDeadlinePayload {
    /// "snbp" | "snbt" | "utbk"
    pub track: String,
    pub year: i32,
    #[validate(length(min = 1, message = "label deadline wajib diisi"))]
    pub label: String,
    #[serde(default)]
    pub description: String,
    pub deadline_date: NaiveDate,
}

impl AdmissionDeadlinePayload {
    pub fn into_create_input(
        self,
    ) -> Result<crate::application::admission_deadline_service::CreateAdmissionDeadlineInput, crate::error::AppError>
    {
        Ok(crate::application::admission_deadline_service::CreateAdmissionDeadlineInput {
            track: self.track.parse().map_err(crate::error::AppError::Validation)?,
            year: self.year,
            label: self.label,
            description: self.description,
            deadline_date: self.deadline_date,
        })
    }

    pub fn into_update_input(
        self,
    ) -> Result<crate::application::admission_deadline_service::UpdateAdmissionDeadlineInput, crate::error::AppError>
    {
        Ok(crate::application::admission_deadline_service::UpdateAdmissionDeadlineInput {
            track: self.track.parse().map_err(crate::error::AppError::Validation)?,
            year: self.year,
            label: self.label,
            description: self.description,
            deadline_date: self.deadline_date,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AdmissionDeadlineResponse {
    pub id: Uuid,
    pub track: String,
    pub year: i32,
    pub label: String,
    pub description: String,
    pub deadline_date: NaiveDate,
    /// Hari tersisa sampai deadline (negatif kalau sudah lewat) — dihitung live dari
    /// tanggal server saat respons dibuat, bukan disimpan sebagai kolom.
    pub days_remaining: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AdmissionDeadline> for AdmissionDeadlineResponse {
    fn from(d: AdmissionDeadline) -> Self {
        let today = Utc::now().date_naive();
        let days_remaining = (d.deadline_date - today).num_days();
        Self {
            id: d.id,
            track: d.track.as_str().to_string(),
            year: d.year,
            label: d.label,
            description: d.description,
            deadline_date: d.deadline_date,
            days_remaining,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}
