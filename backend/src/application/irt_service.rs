//! "Estimasi IRT" use-cases — see `domain::irt` doc comment. Two responsibilities: (1) Admin-
//! triggered recalibration of item difficulties from historical `attempt_answers` data, and
//! (2) computing a single attempt's IRT ability estimate on demand from the LATEST calibration
//! snapshot (deliberately not persisted per-attempt — see doc comment on `estimate_for_attempt`).

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::irt::{calibrate_difficulty, estimate_theta, theta_to_display_score, MIN_ITEM_SAMPLE_SIZE};
use crate::domain::repository::{AttemptRepository, QuestionIrtParamRepository};
use crate::domain::user::Role;
use crate::error::AppResult;
use crate::interfaces::http::middleware::AuthUser;

/// Honest summary of one recalibration run — every count here is real, never fabricated.
pub struct RecalibrationSummary {
    pub calibrated_count: i64,
    pub skipped_insufficient_data_count: i64,
}

/// Current calibration state, for the Admin panel to show "belum pernah dikalibrasi" honestly
/// instead of a blank/misleading UI when calibration has never run.
pub struct CalibrationStatus {
    pub calibrated_item_count: i64,
    pub last_calibrated_at: Option<DateTime<Utc>>,
}

pub struct IrtService {
    attempts: Arc<dyn AttemptRepository>,
    irt_params: Arc<dyn QuestionIrtParamRepository>,
}

impl IrtService {
    pub fn new(attempts: Arc<dyn AttemptRepository>, irt_params: Arc<dyn QuestionIrtParamRepository>) -> Self {
        Self { attempts, irt_params }
    }

    /// Admin-only. Recomputes every question's difficulty from ALL historical graded answers
    /// platform-wide (`AttemptRepository::item_response_stats`) and upserts the result — a full
    /// snapshot recalibration, not an incremental update. No scheduler exists in this backend
    /// (see CLAUDE.md), so this is manually triggered from the Admin "Kalibrasi Ulang IRT"
    /// button; there is no automatic re-run.
    pub async fn recalibrate(&self, actor: &AuthUser) -> AppResult<RecalibrationSummary> {
        actor.require_role(&[Role::Admin])?;
        let stats = self.attempts.item_response_stats().await?;
        let mut to_upsert = Vec::new();
        let mut skipped = 0i64;
        for stat in stats {
            if stat.total < MIN_ITEM_SAMPLE_SIZE {
                skipped += 1;
                continue;
            }
            match calibrate_difficulty(stat.correct, stat.total) {
                Some(b) => to_upsert.push((stat.question_id, b, stat.total)),
                None => skipped += 1,
            }
        }
        let calibrated_count = to_upsert.len() as i64;
        self.irt_params.upsert_many(to_upsert).await?;
        Ok(RecalibrationSummary { calibrated_count, skipped_insufficient_data_count: skipped })
    }

    /// Admin-only read of the current calibration state — used by the "Kalibrasi Ulang IRT"
    /// panel to show real numbers ("N soal terkalibrasi, terakhir <tanggal>") or an honest
    /// "belum pernah dikalibrasi" instead of guessing.
    pub async fn status(&self, actor: &AuthUser) -> AppResult<CalibrationStatus> {
        actor.require_role(&[Role::Admin])?;
        match self.irt_params.calibration_summary().await? {
            Some((count, latest)) => Ok(CalibrationStatus { calibrated_item_count: count, last_calibrated_at: Some(latest) }),
            None => Ok(CalibrationStatus { calibrated_item_count: 0, last_calibrated_at: None }),
        }
    }

    /// Computes one attempt's IRT display score on demand from the latest calibration snapshot
    /// — NOT persisted (unlike `Attempt::score`/`accuracy`, which are written once at submit
    /// time). This is deliberate: as more students answer more questions, item calibration
    /// improves, and an on-demand read means the same past attempt's `irt_score` can honestly
    /// get more accurate over time rather than being frozen at a possibly under-calibrated
    /// snapshot. The tradeoff (explicitly accepted): a past attempt's displayed `irt_score` MAY
    /// shift slightly after a recalibration — this is presented as an estimate, never as a
    /// fixed official score.
    ///
    /// `responses` is `(question_id, is_correct)` for every scoreable (non-essay) question in
    /// the attempt (unanswered questions are passed as `is_correct = false`, matching how
    /// `domain::tryout::compute_score` also counts an omission as earning zero credit). Returns
    /// `None` if fewer than `domain::irt::MIN_CALIBRATED_ITEMS_FOR_SCORE` of those questions
    /// currently have a trusted calibration — an honest "belum tersedia", not a score computed
    /// from too little data.
    pub async fn estimate_for_attempt(&self, responses: &[(Uuid, bool)]) -> AppResult<Option<f64>> {
        let question_ids: Vec<Uuid> = responses.iter().map(|(id, _)| *id).collect();
        let params = self.irt_params.find_by_question_ids(&question_ids).await?;
        let difficulty_by_question: HashMap<Uuid, f64> = params.into_iter().map(|p| (p.question_id, p.difficulty_b)).collect();

        let calibrated_responses: Vec<(bool, f64)> = responses
            .iter()
            .filter_map(|(qid, is_correct)| difficulty_by_question.get(qid).map(|b| (*is_correct, *b)))
            .collect();

        Ok(estimate_theta(&calibrated_responses).map(theta_to_display_score))
    }
}
