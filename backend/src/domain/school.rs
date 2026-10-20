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
    /// `core.tenant.slug` — the "kode sekolah" a legacy/linked-tenant student types in
    /// alongside their username to disambiguate login (see `AuthService::login` doc
    /// comment); irrelevant for a student with a real email (always unambiguous).
    pub slug: String,
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

/// An existing `core.tenant` not yet adopted by any `ptn.school_profile` — the "link to
/// existing tenant" dropdown shown when creating a new Mitra. Deliberately a small, separate
/// shape (not the full `School`) since `ptn.school_profile` doesn't exist for it yet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkableTenant {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

/// Per-school student counts for the admin "Manajemen Sekolah" list — one aggregate query
/// instead of fetching every student row across every school just to `.length` them (see
/// `SchoolRepository::roster_counts` doc comment).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SchoolRosterCount {
    pub school_id: Uuid,
    pub total_students: i64,
    pub active_students: i64,
}

/// An existing `core.akun` row (role = SISWA) at a linked-tenant school that has no
/// `ptn.student_profile` yet — the checklist shown in the import-siswa step for a
/// linked-tenant Mitra, instead of the CSV bulk-create form used for a standalone one. See
/// `SchoolService`/`UserRepository` doc comments on why this never touches `core.akun`
/// itself (no role mutation, no password reset — the account's existing credentials keep
/// working exactly as they do in whichever module created them).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkableStudent {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
}
