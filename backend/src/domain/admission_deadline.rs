//! Kalender Deadline SNBP/SNBT/UTBK — official selection-track dates (quota
//! announcements, registration windows, exam dates) entered by Admin as the single
//! source of truth. Read by Admin/School/Student for the "Deadline Mendatang"
//! countdown widget. There is no cron/scheduler infrastructure in this codebase — the
//! countdown is always computed live (`deadline_date - today`) at read time, never
//! pushed proactively.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdmissionTrack {
    Snbp,
    Snbt,
    Utbk,
}

impl AdmissionTrack {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdmissionTrack::Snbp => "snbp",
            AdmissionTrack::Snbt => "snbt",
            AdmissionTrack::Utbk => "utbk",
        }
    }
}

impl std::str::FromStr for AdmissionTrack {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "snbp" => Ok(AdmissionTrack::Snbp),
            "snbt" => Ok(AdmissionTrack::Snbt),
            "utbk" => Ok(AdmissionTrack::Utbk),
            other => Err(format!("unknown admission track: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissionDeadline {
    pub id: Uuid,
    pub track: AdmissionTrack,
    pub year: i32,
    pub label: String,
    pub description: String,
    pub deadline_date: NaiveDate,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
