use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::application::school_tka_service::{TkaPackageGroup, TkaStudentRow, TkaSubjectScoreRow};
use crate::domain::school::SchoolType;

#[derive(Debug, Serialize)]
pub struct TkaSubjectScoreResponse {
    pub session_id: Uuid,
    pub session_title: String,
    pub is_elective: bool,
    pub attempted: bool,
    pub raw_score: Option<i32>,
    pub scaled_score: Option<i32>,
    pub is_istimewa: Option<bool>,
    pub submitted_at: Option<DateTime<Utc>>,
}

impl From<TkaSubjectScoreRow> for TkaSubjectScoreResponse {
    fn from(r: TkaSubjectScoreRow) -> Self {
        Self {
            session_id: r.session_id,
            session_title: r.session_title,
            is_elective: r.is_elective,
            attempted: r.attempted,
            raw_score: r.raw_score,
            scaled_score: r.scaled_score,
            is_istimewa: r.is_istimewa,
            submitted_at: r.submitted_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TkaStudentRowResponse {
    pub student_id: Uuid,
    pub student_name: String,
    pub elective_chosen_count: i32,
    pub subjects: Vec<TkaSubjectScoreResponse>,
}

impl From<TkaStudentRow> for TkaStudentRowResponse {
    fn from(r: TkaStudentRow) -> Self {
        Self {
            student_id: r.student_id,
            student_name: r.student_name,
            elective_chosen_count: r.elective_chosen_count,
            subjects: r.subjects.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TkaPackageGroupResponse {
    pub package_id: Uuid,
    pub package_name: String,
    pub elective_pick_count: i32,
    pub school_type_scope: Option<SchoolType>,
    pub score_scale_label: String,
    pub istimewa_threshold: i32,
    pub students: Vec<TkaStudentRowResponse>,
}

impl From<TkaPackageGroup> for TkaPackageGroupResponse {
    fn from(g: TkaPackageGroup) -> Self {
        Self {
            package_id: g.package_id,
            package_name: g.package_name,
            elective_pick_count: g.elective_pick_count,
            school_type_scope: g.school_type_scope,
            score_scale_label: g.score_scale_label.to_string(),
            istimewa_threshold: g.istimewa_threshold,
            students: g.students.into_iter().map(Into::into).collect(),
        }
    }
}
