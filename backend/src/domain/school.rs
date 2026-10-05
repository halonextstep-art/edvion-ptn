//! School entity — a partner institution (Sekolah Mitra).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchoolType {
    Sma,
    Smk,
    Ma,
    /// Jenjang SMP — ditambahkan seiring adanya jalur paket "TKA SMP" (lihat migration
    /// 20250101000037_school_type_smp.sql), supaya sekolah mitra tingkat SMP tidak terpaksa
    /// didaftarkan dengan jenjang sma/smk/ma yang salah.
    Smp,
}

impl SchoolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchoolType::Sma => "sma",
            SchoolType::Smk => "smk",
            SchoolType::Ma => "ma",
            SchoolType::Smp => "smp",
        }
    }
}

impl std::str::FromStr for SchoolType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sma" => Ok(SchoolType::Sma),
            "smk" => Ok(SchoolType::Smk),
            "ma" => Ok(SchoolType::Ma),
            "smp" => Ok(SchoolType::Smp),
            other => Err(format!("unknown school_type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageType {
    Basic,
    Premium,
    Enterprise,
}

impl PackageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PackageType::Basic => "basic",
            PackageType::Premium => "premium",
            PackageType::Enterprise => "enterprise",
        }
    }
}

impl std::str::FromStr for PackageType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "basic" => Ok(PackageType::Basic),
            "premium" => Ok(PackageType::Premium),
            "enterprise" => Ok(PackageType::Enterprise),
            other => Err(format!("unknown package_type: {other}")),
        }
    }
}

/// `Pending` lets a newly-added partner sit unapproved until the onboarding wizard (CSV
/// student import) finishes — mirrors `UserStatus::Pending`, but nothing gates on it at
/// login since schools authenticate via their PIC `User` account, not the school row itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchoolStatus {
    Active,
    Inactive,
    Pending,
}

impl SchoolStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchoolStatus::Active => "active",
            SchoolStatus::Inactive => "inactive",
            SchoolStatus::Pending => "pending",
        }
    }
}

impl std::str::FromStr for SchoolStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(SchoolStatus::Active),
            "inactive" => Ok(SchoolStatus::Inactive),
            "pending" => Ok(SchoolStatus::Pending),
            other => Err(format!("unknown school status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct School {
    pub id: Uuid,
    pub name: String,
    pub school_type: SchoolType,
    pub city: String,
    pub province: String,
    pub email: String,
    pub phone: String,
    pub join_date: NaiveDate,
    pub package_type: PackageType,
    pub contact_person: String,
    pub status: SchoolStatus,
    /// Percentage (0-50) of the contracted plan value paid back to the school.
    pub revenue_share: i32,
    /// Contracted plan value in Rupiah, admin-recorded (no billing system yet).
    pub monthly_revenue: i64,
    pub created_at: DateTime<Utc>,
}
