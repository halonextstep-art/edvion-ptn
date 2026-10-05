//! Rasionalisasi SNBT/SNBP use-cases: PTN catalog CRUD (admin-managed, seeded from the real
//! published dataset via `bin/import_ptn_catalog`), student rapor-score entry, target
//! watchlist management, and the admission-chance calculation itself.
//!
//! Authorization model: the *catalog* (list/search programs) is readable by any
//! authenticated role — student, school, admin, content all need to browse it to pick
//! targets or review a school's students. Catalog *writes* (create/update/delete) are
//! admin-only, mirroring Package/Voucher. Rapor scores and targets are owned by the
//! student who entered them; a school PIC or admin may *read* a specific student's
//! computed chances (needed for the School Rekap/SNBP tabs) but can never write another
//! student's rapor/targets.

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::rationalization::{
    average_rapor_for_program, compute_chance, compute_chance_snbp, index_prestasi, unavailable_chance, Achievement,
    AchievementLevel, ChanceResult, Priority, PtnProgram, PtnTarget, RaporScore, Track,
};
use crate::domain::repository::{
    AchievementRepository, AttemptRepository, NewAchievement, NewPtnProgram, NewPtnTarget, PtnProgramFilter,
    PtnProgramRepository, PtnProgramUpdate, PtnTargetRepository, RaporScoreRepository, RaporScoreUpsert,
    SimulationRunRepository,
};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

/// Historical realistic UTBK composite bounds, taken from the same real dataset's "Nilai
/// Dasar" sheet (PG min utbk 425 / PG max utbk 825) — used only to rescale our own
/// platform's 0-1000 tryout score onto the same ballpark as `pg_snbt` so the two are
/// comparable. This is a normalization constant, not a fabricated data point.
const UTBK_SCALE_MAX: f64 = 825.0;
const PLATFORM_SCALE_MAX: f64 = 1000.0;

pub struct CreatePtnProgramInput {
    pub nama_ptn: String,
    pub nama_prodi: String,
    pub provinsi: String,
    pub kota: String,
    pub singkatan: String,
    pub rumpun: String,
    pub mapel_syarat: String,
    pub daya_tampung_snbp: i32,
    pub peminat_snbp: i32,
    pub daya_tampung_snbt: i32,
    pub peminat_snbt: i32,
    pub pg_snbt: f64,
    pub pg_snbp: f64,
    pub jenjang: String,
    pub has_official_stats: bool,
}

pub struct UpdatePtnProgramInput {
    pub nama_ptn: String,
    pub nama_prodi: String,
    pub provinsi: String,
    pub kota: String,
    pub singkatan: String,
    pub rumpun: String,
    pub mapel_syarat: String,
    pub daya_tampung_snbp: i32,
    pub peminat_snbp: i32,
    pub daya_tampung_snbt: i32,
    pub peminat_snbt: i32,
    pub pg_snbt: f64,
    pub pg_snbp: f64,
    pub jenjang: String,
    pub has_official_stats: bool,
}

pub struct RaporEntryInput {
    pub semester: i32,
    pub subject: String,
    pub score: f64,
    pub is_minat: bool,
}

/// A target program bundled with its live-computed chance — never stored, always
/// recalculated from the student's current rapor/tryout data and the program's current
/// catalog row.
pub struct TargetWithChance {
    pub target: PtnTarget,
    pub program: PtnProgram,
    pub chance: ChanceResult,
}

pub struct AchievementInput {
    pub nama: String,
    pub tingkat: AchievementLevel,
    pub tahun: i32,
    pub juara: String,
}

pub struct RationalizationService {
    programs: Arc<dyn PtnProgramRepository>,
    rapor: Arc<dyn RaporScoreRepository>,
    targets: Arc<dyn PtnTargetRepository>,
    attempts: Arc<dyn AttemptRepository>,
    achievements: Arc<dyn AchievementRepository>,
    simulation_runs: Arc<dyn SimulationRunRepository>,
}

impl RationalizationService {
    pub fn new(
        programs: Arc<dyn PtnProgramRepository>,
        rapor: Arc<dyn RaporScoreRepository>,
        targets: Arc<dyn PtnTargetRepository>,
        attempts: Arc<dyn AttemptRepository>,
        achievements: Arc<dyn AchievementRepository>,
        simulation_runs: Arc<dyn SimulationRunRepository>,
    ) -> Self {
        Self { programs, rapor, targets, attempts, achievements, simulation_runs }
    }

    /// SNBT chance input: prefers the student's latest *completed* Simulasi UTBK run's
    /// `combined_estimate_score` (a realistic full-7-subtest estimate) over the plain
    /// average of independent tryout attempts, which only reflects whichever subtests the
    /// student happened to drill individually. Both are rescaled to the same 0-825 UTBK
    /// ballpark before conversion to a percentage of `pg_snbt`, so downstream chance math
    /// is unaffected by which source produced the number.
    async fn snbt_student_pct(&self, student_id: Uuid) -> AppResult<f64> {
        let sim_run = self.simulation_runs.latest_completed_for_student(student_id).await?;
        if let Some(score) = sim_run.and_then(|r| r.combined_estimate_score) {
            return Ok((score / UTBK_SCALE_MAX) * 100.0);
        }
        let attempts = self.attempts.list_by_student(student_id).await?;
        Ok(average_tryout_score(&attempts).map(|s| (s / PLATFORM_SCALE_MAX) * 100.0).unwrap_or(0.0))
    }

    // ─── Achievements ("Prestasi", student-owned) ──────────────────────────────

    /// Core write, no role lock — callers (self-service handler, or the school/admin
    /// "edit on behalf of student" handler after `ensure_owns_student`) decide who's
    /// allowed to reach this for a given `student_id`.
    pub async fn add_achievement_for(&self, student_id: Uuid, input: AchievementInput) -> AppResult<Achievement> {
        if input.nama.trim().is_empty() {
            return Err(AppError::Validation("nama prestasi wajib diisi".to_string()));
        }
        self.achievements
            .create(NewAchievement {
                student_id,
                nama: input.nama,
                tingkat: input.tingkat,
                tahun: input.tahun,
                juara: input.juara,
            })
            .await
    }

    pub async fn add_my_achievement(&self, actor: &AuthUser, input: AchievementInput) -> AppResult<Achievement> {
        actor.require_role(&[Role::Student])?;
        self.add_achievement_for(actor.user_id, input).await
    }

    pub async fn list_achievements_for(&self, student_id: Uuid) -> AppResult<Vec<Achievement>> {
        self.achievements.list_by_student(student_id).await
    }

    pub async fn list_my_achievements(&self, actor: &AuthUser) -> AppResult<Vec<Achievement>> {
        actor.require_role(&[Role::Student])?;
        self.list_achievements_for(actor.user_id).await
    }

    pub async fn delete_achievement_for(&self, student_id: Uuid, id: Uuid) -> AppResult<()> {
        let deleted = self.achievements.delete(student_id, id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("prestasi {id} tidak ditemukan")));
        }
        Ok(())
    }

    pub async fn delete_my_achievement(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Student])?;
        self.delete_achievement_for(actor.user_id, id).await
    }

    /// Attaches a certificate scan to one of the caller's own achievements. Self-service
    /// only (unlike rapor/target/tracking, a certificate photo is something only the
    /// student physically has — there's no "school edits on behalf" variant for this).
    pub async fn set_my_achievement_certificate(
        &self,
        actor: &AuthUser,
        id: Uuid,
        certificate_url: Option<String>,
    ) -> AppResult<Achievement> {
        actor.require_role(&[Role::Student])?;
        self.achievements
            .set_certificate(actor.user_id, id, certificate_url)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("prestasi {id} tidak ditemukan")))
    }

    // ─── Catalog (admin write, everyone read) ──────────────────────────────────

    pub async fn create_program(&self, actor: &AuthUser, input: CreatePtnProgramInput) -> AppResult<PtnProgram> {
        actor.require_role(&[Role::Admin])?;
        validate_program(&input.nama_ptn, &input.nama_prodi)?;
        // Synthetic, monotonically-increasing kode for admin-added rows — the real
        // catalog's numeric codes only carry meaning for the imported CSV source.
        let kode = Utc::now().timestamp_millis();
        self.programs
            .create(NewPtnProgram {
                kode,
                nama_ptn: input.nama_ptn,
                nama_prodi: input.nama_prodi,
                provinsi: input.provinsi,
                kota: input.kota,
                singkatan: input.singkatan,
                rumpun: input.rumpun,
                mapel_syarat: input.mapel_syarat,
                daya_tampung_snbp: input.daya_tampung_snbp,
                peminat_snbp: input.peminat_snbp,
                daya_tampung_snbt: input.daya_tampung_snbt,
                peminat_snbt: input.peminat_snbt,
                pg_snbt: input.pg_snbt,
                pg_snbp: input.pg_snbp,
                jenjang: input.jenjang,
                has_official_stats: input.has_official_stats,
            })
            .await
    }

    pub async fn list_programs(
        &self,
        _actor: &AuthUser,
        search: Option<String>,
        rumpun: Option<String>,
        jenjang: Option<String>,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<PtnProgram>, i64)> {
        self.programs
            .list(PtnProgramFilter { search, rumpun, jenjang, page, page_size })
            .await
    }

    pub async fn get_program(&self, _actor: &AuthUser, id: Uuid) -> AppResult<PtnProgram> {
        self.programs
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("program studi {id} tidak ditemukan")))
    }

    pub async fn distinct_rumpun(&self, _actor: &AuthUser) -> AppResult<Vec<String>> {
        self.programs.distinct_rumpun().await
    }

    pub async fn catalog_size(&self, _actor: &AuthUser) -> AppResult<i64> {
        self.programs.count_all().await
    }

    pub async fn update_program(&self, actor: &AuthUser, id: Uuid, input: UpdatePtnProgramInput) -> AppResult<PtnProgram> {
        actor.require_role(&[Role::Admin])?;
        validate_program(&input.nama_ptn, &input.nama_prodi)?;
        self.programs
            .update(
                id,
                PtnProgramUpdate {
                    nama_ptn: input.nama_ptn,
                    nama_prodi: input.nama_prodi,
                    provinsi: input.provinsi,
                    kota: input.kota,
                    singkatan: input.singkatan,
                    rumpun: input.rumpun,
                    mapel_syarat: input.mapel_syarat,
                    daya_tampung_snbp: input.daya_tampung_snbp,
                    peminat_snbp: input.peminat_snbp,
                    daya_tampung_snbt: input.daya_tampung_snbt,
                    peminat_snbt: input.peminat_snbt,
                    pg_snbt: input.pg_snbt,
                    pg_snbp: input.pg_snbp,
                    jenjang: input.jenjang,
                    has_official_stats: input.has_official_stats,
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("program studi {id} tidak ditemukan")))
    }

    pub async fn delete_program(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        let deleted = self.programs.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("program studi {id} tidak ditemukan")));
        }
        Ok(())
    }

    // ─── Rapor scores (student-owned) ───────────────────────────────────────────

    pub async fn upsert_rapor_for(&self, student_id: Uuid, entries: Vec<RaporEntryInput>) -> AppResult<Vec<RaporScore>> {
        let mut saved = Vec::with_capacity(entries.len());
        for e in entries {
            if !(0.0..=100.0).contains(&e.score) {
                return Err(AppError::Validation("nilai rapor harus di antara 0 dan 100".to_string()));
            }
            if e.subject.trim().is_empty() {
                return Err(AppError::Validation("nama mata pelajaran wajib diisi".to_string()));
            }
            saved.push(
                self.rapor
                    .upsert(
                        student_id,
                        RaporScoreUpsert { semester: e.semester, subject: e.subject, score: e.score, is_minat: e.is_minat },
                    )
                    .await?,
            );
        }
        Ok(saved)
    }

    pub async fn upsert_my_rapor(&self, actor: &AuthUser, entries: Vec<RaporEntryInput>) -> AppResult<Vec<RaporScore>> {
        actor.require_role(&[Role::Student])?;
        self.upsert_rapor_for(actor.user_id, entries).await
    }

    pub async fn list_rapor_for(&self, student_id: Uuid) -> AppResult<Vec<RaporScore>> {
        self.rapor.list_by_student(student_id).await
    }

    pub async fn list_my_rapor(&self, actor: &AuthUser) -> AppResult<Vec<RaporScore>> {
        actor.require_role(&[Role::Student])?;
        self.list_rapor_for(actor.user_id).await
    }

    pub async fn delete_rapor_for(&self, student_id: Uuid, id: Uuid) -> AppResult<()> {
        let deleted = self.rapor.delete(student_id, id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("nilai rapor {id} tidak ditemukan")));
        }
        Ok(())
    }

    pub async fn delete_my_rapor(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Student])?;
        self.delete_rapor_for(actor.user_id, id).await
    }

    // ─── Targets + chance calculation (student-owned; school/admin may edit on
    //     behalf of their own student for verification — see `*_for` variants) ────

    pub async fn add_target_for(
        &self,
        student_id: Uuid,
        ptn_program_id: Uuid,
        track: Track,
        priority: Priority,
    ) -> AppResult<PtnTarget> {
        // Fail fast with a clean 404 if the program doesn't exist, rather than surfacing a
        // raw FK-violation from Postgres.
        self.programs
            .find_by_id(ptn_program_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("program studi {ptn_program_id} tidak ditemukan")))?;
        let existing = self.targets.list_by_student(student_id).await?;
        let sort_order = existing.len() as i32;
        self.targets
            .create(NewPtnTarget { student_id, ptn_program_id, track, priority, sort_order })
            .await
    }

    pub async fn add_target(
        &self,
        actor: &AuthUser,
        ptn_program_id: Uuid,
        track: Track,
        priority: Priority,
    ) -> AppResult<PtnTarget> {
        actor.require_role(&[Role::Student])?;
        self.add_target_for(actor.user_id, ptn_program_id, track, priority).await
    }

    pub async fn set_target_priority_for(&self, student_id: Uuid, id: Uuid, priority: Priority) -> AppResult<PtnTarget> {
        self.targets
            .set_priority(student_id, id, priority)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("target {id} tidak ditemukan")))
    }

    pub async fn set_target_priority(&self, actor: &AuthUser, id: Uuid, priority: Priority) -> AppResult<PtnTarget> {
        actor.require_role(&[Role::Student])?;
        self.set_target_priority_for(actor.user_id, id, priority).await
    }

    pub async fn remove_target_for(&self, student_id: Uuid, id: Uuid) -> AppResult<()> {
        let deleted = self.targets.delete(student_id, id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("target {id} tidak ditemukan")));
        }
        Ok(())
    }

    pub async fn remove_target(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Student])?;
        self.remove_target_for(actor.user_id, id).await
    }

    /// The student's own targets, each bundled with a freshly-computed chance. This is the
    /// main read used by the student Rasionalisasi SNBT page.
    pub async fn my_targets_with_chance(&self, actor: &AuthUser) -> AppResult<Vec<TargetWithChance>> {
        actor.require_role(&[Role::Student])?;
        self.targets_with_chance_for(actor.user_id).await
    }

    /// Same computation but for an arbitrary student — used by School Rekap SNBT / SNBP
    /// Rasionalisasi tabs (school PIC/admin viewing their own students' targets). Role
    /// enforcement (that the caller is actually allowed to see *this* student) is done by
    /// the caller (school/user service already scopes students to their own school).
    pub async fn targets_with_chance_for(&self, student_id: Uuid) -> AppResult<Vec<TargetWithChance>> {
        let targets = self.targets.list_by_student(student_id).await?;
        if targets.is_empty() {
            return Ok(Vec::new());
        }

        let rapor = self.rapor.list_by_student(student_id).await?;

        let student_pct_snbt = self.snbt_student_pct(student_id).await?;

        let achievements = self.achievements.list_by_student(student_id).await?;
        let prestasi_bonus = index_prestasi(&achievements);

        let mut out = Vec::with_capacity(targets.len());
        for target in targets {
            let program = match self.programs.find_by_id(target.ptn_program_id).await? {
                Some(p) => p,
                None => continue, // program was deleted from the catalog since the target was added
            };
            // Blended toward THIS program's required subjects (see `average_rapor_for_program`
            // doc comment) — computed per-target, not once per student, since each target can
            // have a different `mapel_syarat`.
            let rapor_average = average_rapor_for_program(&rapor, &program.mapel_syarat);
            let chance = if !program.has_official_stats {
                // No verified official daya-tampung/peminat/passing-grade for this program —
                // refuse to fabricate an estimate rather than let the zero-defaulted stat
                // fields read as "no competition" (see `unavailable_chance` doc comment).
                let student_value = match target.track {
                    Track::Snbp => rapor_average.unwrap_or(0.0),
                    Track::Snbt => student_pct_snbt,
                };
                unavailable_chance(student_value)
            } else {
                match target.track {
                    Track::Snbp => {
                        let student_value = rapor_average.unwrap_or(0.0);
                        compute_chance_snbp(student_value, program.pg_snbp, program.competition_ratio_snbp(), prestasi_bonus)
                    }
                    Track::Snbt => {
                        let threshold_pct = (program.pg_snbt / UTBK_SCALE_MAX) * 100.0;
                        compute_chance(student_pct_snbt, threshold_pct, program.competition_ratio_snbt())
                    }
                }
            };
            out.push(TargetWithChance { target, program, chance });
        }
        Ok(out)
    }

    /// Ad-hoc "what if I target this program" preview — used by the picker UI before the
    /// student commits to adding it as a target, so they see the estimate first.
    pub async fn preview_chance_for(&self, student_id: Uuid, ptn_program_id: Uuid, track: Track) -> AppResult<ChanceResult> {
        let program = self
            .programs
            .find_by_id(ptn_program_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("program studi {ptn_program_id} tidak ditemukan")))?;
        if !program.has_official_stats {
            let student_value = match track {
                Track::Snbp => {
                    let rapor = self.rapor.list_by_student(student_id).await?;
                    average_rapor_for_program(&rapor, &program.mapel_syarat).unwrap_or(0.0)
                }
                Track::Snbt => self.snbt_student_pct(student_id).await?,
            };
            return Ok(unavailable_chance(student_value));
        }
        let chance = match track {
            Track::Snbp => {
                let rapor = self.rapor.list_by_student(student_id).await?;
                let student_value = average_rapor_for_program(&rapor, &program.mapel_syarat).unwrap_or(0.0);
                let achievements = self.achievements.list_by_student(student_id).await?;
                let prestasi_bonus = index_prestasi(&achievements);
                compute_chance_snbp(student_value, program.pg_snbp, program.competition_ratio_snbp(), prestasi_bonus)
            }
            Track::Snbt => {
                let student_pct = self.snbt_student_pct(student_id).await?;
                let threshold_pct = (program.pg_snbt / UTBK_SCALE_MAX) * 100.0;
                compute_chance(student_pct, threshold_pct, program.competition_ratio_snbt())
            }
        };
        Ok(chance)
    }

    pub async fn preview_chance(&self, actor: &AuthUser, ptn_program_id: Uuid, track: Track) -> AppResult<ChanceResult> {
        actor.require_role(&[Role::Student])?;
        self.preview_chance_for(actor.user_id, ptn_program_id, track).await
    }
}

/// Average of the student's real tryout/drilling attempt scores (0-1000 platform scale).
/// Only counts attempts that have actually been submitted (have a score).
fn average_tryout_score(attempts: &[crate::domain::tryout::Attempt]) -> Option<f64> {
    let scored: Vec<i32> = attempts.iter().filter_map(|a| a.score).collect();
    if scored.is_empty() {
        return None;
    }
    Some(scored.iter().map(|s| *s as f64).sum::<f64>() / scored.len() as f64)
}

fn validate_program(nama_ptn: &str, nama_prodi: &str) -> AppResult<()> {
    if nama_ptn.trim().is_empty() {
        return Err(AppError::Validation("nama PTN wajib diisi".to_string()));
    }
    if nama_prodi.trim().is_empty() {
        return Err(AppError::Validation("nama program studi wajib diisi".to_string()));
    }
    Ok(())
}
