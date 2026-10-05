//! User entity — represents any authenticated actor in the system.
//! Mirrors the four roles from the reference design: admin (Admin Pusat),
//! school (Portal Sekolah / school PIC), student (Portal Siswa), content (Tim Konten).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    School,
    Student,
    Content,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::School => "school",
            Role::Student => "student",
            Role::Content => "content",
        }
    }

    /// Roles allowed to create/edit/delete questions directly (question bank owners).
    pub fn can_manage_question_bank(&self) -> bool {
        matches!(self, Role::Admin | Role::Content)
    }

    /// Only the central admin can approve/reject/request revision on submitted questions.
    pub fn can_review_questions(&self) -> bool {
        matches!(self, Role::Admin)
    }
}

impl std::str::FromStr for Role {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "school" => Ok(Role::School),
            "student" => Ok(Role::Student),
            "content" => Ok(Role::Content),
            other => Err(format!("unknown role: {other}")),
        }
    }
}

/// Account lifecycle. `Pending` is meaningful (not decorative): `AuthService::login` refuses
/// to issue a token for a pending or inactive account, so this actually gates access — most
/// relevant for school accounts awaiting the onboarding wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Pending,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Inactive => "inactive",
            UserStatus::Pending => "pending",
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(UserStatus::Active),
            "inactive" => Ok(UserStatus::Inactive),
            "pending" => Ok(UserStatus::Pending),
            other => Err(format!("unknown user status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: Role,
    pub school_id: Option<Uuid>,
    pub school_name: Option<String>,
    pub status: UserStatus,
    pub last_login: Option<DateTime<Utc>>,
    pub phone: Option<String>,
    /// Student-only, resolved from the 1:1 `students` profile row.
    pub nisn: Option<String>,
    /// Student-only, resolved from the 1:1 `students` profile row (e.g. "Kelas 12").
    pub grade: Option<String>,
    /// Student-only — nomor induk sekolah (distinct from `nisn`, the national/Kemendikbud
    /// ID). See migration 20250101000038_student_import_fields.sql.
    pub nis: Option<String>,
    /// Student-only, raw value as given by the school's import file (not normalized to a
    /// fixed L/P enum — different schools spell this differently).
    pub gender: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
    /// Tanggal masuk sekolah (distinct from `created_at`, which is when the *account* was
    /// created in this system — a student may have enrolled at the school years earlier).
    pub enrolled_at: Option<chrono::NaiveDate>,
    /// "Kode rombel" — distinct from `grade` ("nama rombel"/"Kelas 12"), since some schools'
    /// import files give these as two different values (e.g. code "9A" vs display "Kelas 9 A").
    pub rombel_code: Option<String>,
    /// Raw "username" value from the school's import file — never used for authentication
    /// (login is always by email), kept only for reference/export fidelity.
    pub username: Option<String>,
    /// URL path (e.g. `/uploads/avatars/xxx.jpg`) to a locally-stored profile photo, or
    /// `None` if the user never uploaded one — the UI falls back to initials.
    pub avatar_url: Option<String>,
    /// Self-declared field/major of interest ("Jurusan yang Diminati") — a real signal the
    /// student types in themselves, distinct from `PtnProgram::rumpun` (a catalog attribute
    /// of one program, not a personal declaration). Only the student sets this (self-service
    /// profile update); schools/admins read it but never write it.
    pub minat_jurusan: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl User {
    /// Business rule: a `school` or `student` account should be tied to a school.
    /// Admin/content accounts are central and have no school affiliation.
    pub fn requires_school(&self) -> bool {
        matches!(self.role, Role::School | Role::Student)
    }
}
