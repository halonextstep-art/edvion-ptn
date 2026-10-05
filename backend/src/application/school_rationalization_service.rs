//! School-portal-only use-cases for the Rasionalisasi SNBP sub-tabs (live dashboard
//! aggregates, full ranking, "Kaka Kelas" alumni benchmarks, "Eligible" per-year counts)
//! and for Rekap SNBT (real post-exam tracking per student: actual score, exam date,
//! admission status — one real UTBK exam per student, not per target).
//!
//! Alumni benchmarks, eligibility counts, and SNBT tracking are all real inputs typed in
//! by the School PIC (there is no dataset anywhere that could derive them automatically)
//! — same honest-input pattern as `RaporScore`/`PtnTarget` on the student side. The
//! dashboard's "Rata-rata Peluang SNBP" stat reuses the same `compute_chance` engine as
//! the student Rasionalisasi tab, run per-student across the school's real SNBP targets,
//! rather than showing the reference prototype's "Terkonfirmasi Lolos" (confirmed-accepted
//! count), which would require an admission-outcome-tracking feature this system does not
//! have for SNBP (Rekap SNBT's "Diterima" stat *is* real, since it comes from the school's
//! own tracked `status`, not an inferred admission outcome).

use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use chrono::{Datelike, NaiveDate, Utc};

use crate::application::tryout_service::{AttemptResult, ReviewItem, TryoutService};
use crate::domain::rationalization::{
    average_rapor_for_program, compute_chance_snbp, index_prestasi, AlumniBenchmark, AlumniRaporScore, ChanceTier,
    Priority, RaporScore, SchoolEligibility, SnbpStatus, SnbtTrackingStatus, Track,
};
use crate::domain::repository::{
    AchievementRepository, AlumniBenchmarkRepository, AlumniRaporScoreUpsert, AttemptRepository, NewAlumniBenchmark,
    PtnProgramRepository, PtnTargetRepository, RaporScoreRepository, RaporScoreUpsert, SchoolEligibilityRepository,
    SchoolEligibilityUpsert, SimulationRunRepository, SnbpParticipationRepository, SnbpParticipationUpsert,
    SnbtTrackingRepository, SnbtTrackingUpsert, UserFilter, UserRepository,
};
use crate::domain::tryout::Attempt;
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

/// Same normalization constants as `rationalization_service` — a real UTBK composite's
/// realistic bounds (425-825), used to rescale our own platform's 0-1000 tryout score onto
/// the same ballpark as `pg_snbt` so the two are comparable in the Rekap SNBT roster.
const UTBK_SCALE_MAX: f64 = 825.0;
const PLATFORM_SCALE_MAX: f64 = 1000.0;

pub struct AddAlumniInput {
    pub alumni_name: String,
    pub graduation_year: i32,
    pub track: Track,
    pub nama_ptn: String,
    pub nama_prodi: String,
    pub benchmark_score: f64,
}

pub struct UniversityCount {
    pub nama_ptn: String,
    pub count: i64,
}

pub struct StudentChanceSummary {
    pub student_id: Uuid,
    pub student_name: String,
    pub best_chance_percent: f64,
    pub tier: ChanceTier,
}

/// One row of the "Rasionalisasi" sub-tab's full ranking table — same computation as the
/// Dashboard's "Top Peluang" but uncapped (every SNBP-targeting student, not just top 5),
/// plus the real index inputs (`rapor_avg`, `prestasi_index`) that fed the chance estimate,
/// so the school can see *why* a student ranks where they do without opening the detail
/// view for every row.
pub struct SnbpRankingEntry {
    pub student_id: Uuid,
    pub student_name: String,
    pub rapor_avg: Option<f64>,
    pub prestasi_index: f64,
    pub target_count: i64,
    pub best_chance_percent: f64,
    pub tier: ChanceTier,
}

pub struct SnbpDashboard {
    pub total_siswa_aktif: i64,
    pub terasionalisasi: i64,
    pub avg_chance_snbp: Option<f64>,
    pub eligible_terbaru: Option<SchoolEligibility>,
    pub distribusi_universitas: Vec<UniversityCount>,
    pub top_peluang: Vec<StudentChanceSummary>,
}

/// One SNBT program choice (pilihan 1/2), with its real catalog competition fields —
/// same source as the "Pos. Minimum" tab, so the UI can show a genuine ratio instead of
/// fabricating one.
#[derive(Clone)]
pub struct SnbtChoice {
    pub nama_ptn: String,
    pub nama_prodi: String,
    pub priority: Priority,
    pub pg_snbt: f64,
    pub daya_tampung_snbt: i32,
    pub peminat_snbt: i32,
}

/// One row of the "Rekap SNBT" table — one student with real SNBT target(s), with the
/// school's real post-exam tracking merged in (if any has been entered yet). Tracking is
/// per-student (one real exam, one score/status), not per-target: see the doc comment on
/// `SnbtTracking`.
pub struct SnbtRosterRow {
    pub student_id: Uuid,
    pub student_name: String,
    pub pilihan_1: Option<SnbtChoice>,
    pub pilihan_2: Option<SnbtChoice>,
    /// Real estimate, rescaled to the UTBK ballpark so it's directly comparable to
    /// `pg_snbt`. Prefers the student's latest *completed* Simulasi UTBK run (a realistic
    /// full 7-subtest estimate) over the plain average of independent tryout attempts.
    /// `None` if the student has neither.
    pub est_score: Option<f64>,
    /// Where `est_score` came from: `"simulasi"` (a completed Simulasi UTBK run) or
    /// `"drilling"` (fallback average of independent tryout attempts) — surfaced to the
    /// school PIC so they can judge the estimate's reliability.
    pub est_score_source: Option<&'static str>,
    pub actual_score: Option<f64>,
    pub exam_date: Option<NaiveDate>,
    pub status: SnbtTrackingStatus,
    pub notes: String,
}

pub struct SnbtDashboard {
    pub terdaftar: i64,
    pub sudah_ujian: i64,
    pub diterima: i64,
    pub skor_perlu_diisi: i64,
}

pub struct SnbtTrackingInput {
    pub actual_score: Option<f64>,
    pub exam_date: Option<NaiveDate>,
    pub status: SnbtTrackingStatus,
    pub notes: String,
}

/// One row of the "Daftar Siswa" roster table. `pilihan_1`/`pilihan_2` are deliberately
/// derived live from the student's real `PtnTarget` rows (priority = utama/cadangan) rather
/// than stored — see the doc comment on the `snbp_participation` migration.
pub struct SnbpRosterRow {
    pub student_id: Uuid,
    pub student_name: String,
    pub kelas: Option<String>,
    pub konsultan: String,
    pub pilihan_1: Option<String>,
    pub pilihan_2: Option<String>,
    pub status: SnbpStatus,
    pub aktif: bool,
    pub year: i32,
    /// Real, student-declared "Jurusan yang Diminati" — read-only here (school never edits
    /// it; only the student sets it via self-service profile update). Used by the school
    /// portal's detail dialog to compute the "Alternatif Sesuai Minat Jurusan" suggestions
    /// with the same real search+preview mechanism as the rumpun-based alternatives.
    pub minat_jurusan: Option<String>,
}

pub struct SnbpParticipationInput {
    pub year: i32,
    pub konsultan: String,
    pub status: SnbpStatus,
    pub aktif: bool,
}

pub struct SchoolRationalizationService {
    users: Arc<dyn UserRepository>,
    ptn_targets: Arc<dyn PtnTargetRepository>,
    programs: Arc<dyn PtnProgramRepository>,
    rapor: Arc<dyn RaporScoreRepository>,
    alumni: Arc<dyn AlumniBenchmarkRepository>,
    eligibility: Arc<dyn SchoolEligibilityRepository>,
    achievements: Arc<dyn AchievementRepository>,
    attempts: Arc<dyn AttemptRepository>,
    snbt_tracking: Arc<dyn SnbtTrackingRepository>,
    snbp_participation: Arc<dyn SnbpParticipationRepository>,
    simulation_runs: Arc<dyn SimulationRunRepository>,
    /// Collaborator for the "Riwayat Pengerjaan TO" drill-down's per-attempt result/pembahasan
    /// (`student_attempt_result`/`student_attempt_review` below) — reuses `TryoutService`'s
    /// exact grading/breakdown logic instead of duplicating it here.
    tryout: Arc<TryoutService>,
}

impl SchoolRationalizationService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        ptn_targets: Arc<dyn PtnTargetRepository>,
        programs: Arc<dyn PtnProgramRepository>,
        rapor: Arc<dyn RaporScoreRepository>,
        alumni: Arc<dyn AlumniBenchmarkRepository>,
        eligibility: Arc<dyn SchoolEligibilityRepository>,
        achievements: Arc<dyn AchievementRepository>,
        attempts: Arc<dyn AttemptRepository>,
        snbt_tracking: Arc<dyn SnbtTrackingRepository>,
        snbp_participation: Arc<dyn SnbpParticipationRepository>,
        simulation_runs: Arc<dyn SimulationRunRepository>,
        tryout: Arc<TryoutService>,
    ) -> Self {
        Self {
            users,
            ptn_targets,
            programs,
            rapor,
            alumni,
            eligibility,
            achievements,
            attempts,
            snbt_tracking,
            snbp_participation,
            simulation_runs,
            tryout,
        }
    }

    // ─── SNBP roster ("Daftar Siswa") ───────────────────────────────────────────

    /// Every student on the school's roster (not just those with SNBP targets — unlike
    /// `full_ranking`, this is the administrative roster, so inactive/not-yet-targeting
    /// students still need to show up so the PIC can assign a consultant early).
    pub async fn snbp_roster(&self, actor: &AuthUser) -> AppResult<Vec<SnbpRosterRow>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let students = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() })
            .await?;
        let participation = self.snbp_participation.list_by_school(school_id).await?;
        let current_year = Utc::now().year();

        let mut out = Vec::with_capacity(students.len());
        for student in &students {
            let targets = self.ptn_targets.list_by_student(student.id).await?;
            let snbp_targets: Vec<_> = targets.into_iter().filter(|t| t.track == Track::Snbp).collect();

            let mut pilihan_1 = None;
            let mut pilihan_2 = None;
            for t in &snbp_targets {
                let Some(program) = self.programs.find_by_id(t.ptn_program_id).await? else { continue };
                let label = format!("{} - {}", program.nama_ptn, program.nama_prodi);
                match t.priority {
                    Priority::Utama if pilihan_1.is_none() => pilihan_1 = Some(label),
                    Priority::Cadangan if pilihan_2.is_none() => pilihan_2 = Some(label),
                    _ => {}
                }
            }

            let p = participation.iter().find(|p| p.student_id == student.id);
            out.push(SnbpRosterRow {
                student_id: student.id,
                student_name: student.name.clone(),
                kelas: student.grade.clone(),
                konsultan: p.map(|p| p.konsultan.clone()).unwrap_or_default(),
                pilihan_1,
                pilihan_2,
                status: p.map(|p| p.status).unwrap_or(SnbpStatus::Belum),
                aktif: p.map(|p| p.aktif).unwrap_or(true),
                year: p.map(|p| p.year).unwrap_or(current_year),
                minat_jurusan: student.minat_jurusan.clone(),
            });
        }
        Ok(out)
    }

    /// Upserts one student's roster metadata (konsultan/status/aktif). Ownership is verified
    /// via the same `ensure_owns_student` check used by the shared editor's school-edit path.
    pub async fn upsert_snbp_participation(
        &self,
        actor: &AuthUser,
        student_id: Uuid,
        input: SnbpParticipationInput,
    ) -> AppResult<SnbpRosterRow> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.ensure_owns_student(school_id, student_id).await?;

        self.snbp_participation
            .upsert(
                student_id,
                SnbpParticipationUpsert {
                    school_id,
                    year: input.year,
                    konsultan: input.konsultan,
                    status: input.status,
                    aktif: input.aktif,
                },
            )
            .await?;

        let roster = self.snbp_roster(actor).await?;
        roster
            .into_iter()
            .find(|r| r.student_id == student_id)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("baris roster SNBP hilang setelah disimpan")))
    }

    /// Confirms `student_id` actually belongs to this school before letting the PIC write
    /// roster metadata for them — mirrors the check already used by the shared SNBP editor's
    /// school-on-behalf routes (`authorize_student_edit` in the rationalization handler).
    async fn ensure_owns_student(&self, school_id: Uuid, student_id: Uuid) -> AppResult<()> {
        let student = self
            .users
            .find_by_id(student_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("siswa {student_id} tidak ditemukan")))?;
        if student.school_id != Some(school_id) {
            return Err(AppError::Forbidden("siswa ini bukan bagian dari sekolah Anda".to_string()));
        }
        Ok(())
    }

    // ─── Dashboard ──────────────────────────────────────────────────────────────

    pub async fn dashboard(&self, actor: &AuthUser) -> AppResult<SnbpDashboard> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let students = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() })
            .await?;

        let (ranking, uni_counts, chance_sum, chance_count) = self.snbp_ranking_for(&students).await?;

        let mut top: Vec<StudentChanceSummary> = ranking
            .iter()
            .map(|r| StudentChanceSummary {
                student_id: r.student_id,
                student_name: r.student_name.clone(),
                best_chance_percent: r.best_chance_percent,
                tier: r.tier,
            })
            .collect();
        top.truncate(5);

        let mut distribusi: Vec<UniversityCount> =
            uni_counts.into_iter().map(|(nama_ptn, count)| UniversityCount { nama_ptn, count }).collect();
        distribusi.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.nama_ptn.cmp(&b.nama_ptn)));
        distribusi.truncate(6);

        let eligible_terbaru = self
            .eligibility
            .list_by_school(school_id)
            .await?
            .into_iter()
            .max_by_key(|e| e.year);

        Ok(SnbpDashboard {
            total_siswa_aktif: students.len() as i64,
            terasionalisasi: ranking.len() as i64,
            avg_chance_snbp: if chance_count > 0 { Some(chance_sum / chance_count as f64) } else { None },
            eligible_terbaru,
            distribusi_universitas: distribusi,
            top_peluang: top,
        })
    }

    /// "Rasionalisasi" sub-tab's full ranking table — every SNBP-targeting student, sorted
    /// by their best real chance estimate, descending. Same computation as `dashboard`'s
    /// "Top Peluang", just uncapped and including the real index inputs per row.
    pub async fn full_ranking(&self, actor: &AuthUser) -> AppResult<Vec<SnbpRankingEntry>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let students = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() })
            .await?;
        let (ranking, _, _, _) = self.snbp_ranking_for(&students).await?;
        Ok(ranking)
    }

    /// Shared N+1 computation behind `dashboard` and `full_ranking`: for every given
    /// student, look at their real SNBP targets and compute each one's real chance (rapor +
    /// prestasi + program catalog), returning the per-student ranking rows (sorted, best
    /// chance first) plus the raw university-count map and chance sum/count needed for the
    /// dashboard's aggregate stats.
    async fn snbp_ranking_for(
        &self,
        students: &[crate::domain::user::User],
    ) -> AppResult<(Vec<SnbpRankingEntry>, HashMap<String, i64>, f64, i64)> {
        let mut uni_counts: HashMap<String, i64> = HashMap::new();
        let mut chance_sum = 0.0;
        let mut chance_count = 0i64;
        let mut ranking: Vec<SnbpRankingEntry> = Vec::new();

        for student in students {
            let targets = self.ptn_targets.list_by_student(student.id).await?;
            let snbp_targets: Vec<_> = targets.into_iter().filter(|t| t.track == Track::Snbp).collect();
            if snbp_targets.is_empty() {
                continue;
            }
            let rapor = self.rapor.list_by_student(student.id).await?;
            let rapor_avg = average_rapor(&rapor);
            let achievements = self.achievements.list_by_student(student.id).await?;
            let prestasi_bonus = index_prestasi(&achievements);

            let mut best: Option<(f64, ChanceTier)> = None;
            for t in &snbp_targets {
                let Some(program) = self.programs.find_by_id(t.ptn_program_id).await? else { continue };
                *uni_counts.entry(program.nama_ptn.clone()).or_insert(0) += 1;
                // No verified official stats for this program — exclude it from the
                // best/average chance aggregate rather than let a fabricated estimate skew
                // the school-wide ranking (still counted above for the university tally).
                if !program.has_official_stats {
                    continue;
                }
                // Blended toward THIS program's required subjects for the actual estimate
                // (see `average_rapor_for_program`); `rapor_avg` above stays the flat,
                // program-agnostic average used only for the ranking table's display column.
                let student_value = average_rapor_for_program(&rapor, &program.mapel_syarat).unwrap_or(0.0);
                let chance = compute_chance_snbp(student_value, program.pg_snbp, program.competition_ratio_snbp(), prestasi_bonus);
                chance_sum += chance.chance_percent;
                chance_count += 1;
                if best.as_ref().map_or(true, |(p, _)| chance.chance_percent > *p) {
                    best = Some((chance.chance_percent, chance.tier));
                }
            }
            if let Some((best_chance_percent, tier)) = best {
                ranking.push(SnbpRankingEntry {
                    student_id: student.id,
                    student_name: student.name.clone(),
                    rapor_avg,
                    prestasi_index: prestasi_bonus,
                    target_count: snbp_targets.len() as i64,
                    best_chance_percent,
                    tier,
                });
            }
        }

        ranking.sort_by(|a, b| b.best_chance_percent.partial_cmp(&a.best_chance_percent).unwrap_or(std::cmp::Ordering::Equal));
        Ok((ranking, uni_counts, chance_sum, chance_count))
    }

    // ─── Alumni ("Kaka Kelas") ──────────────────────────────────────────────────

    pub async fn list_alumni(&self, actor: &AuthUser) -> AppResult<Vec<AlumniBenchmark>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.alumni.list_by_school(school_id).await
    }

    pub async fn add_alumni(&self, actor: &AuthUser, input: AddAlumniInput) -> AppResult<AlumniBenchmark> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        if input.alumni_name.trim().is_empty() {
            return Err(AppError::Validation("nama alumni wajib diisi".to_string()));
        }
        if input.nama_ptn.trim().is_empty() || input.nama_prodi.trim().is_empty() {
            return Err(AppError::Validation("nama PTN dan program studi wajib diisi".to_string()));
        }
        self.alumni
            .create(NewAlumniBenchmark {
                school_id,
                alumni_name: input.alumni_name,
                graduation_year: input.graduation_year,
                track: input.track,
                nama_ptn: input.nama_ptn,
                nama_prodi: input.nama_prodi,
                benchmark_score: input.benchmark_score,
                created_by: actor.user_id,
            })
            .await
    }

    pub async fn delete_alumni(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let deleted = self.alumni.delete(school_id, id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("data alumni {id} tidak ditemukan")));
        }
        Ok(())
    }

    /// Verifikasi `alumni_id` benar-benar milik sekolah actor sebelum baca/tulis "Detail
    /// Rapor"-nya — mencegah PIC sekolah lain menebak UUID alumni sekolah lain untuk
    /// membaca/menimpa detail nilainya.
    async fn ensure_owns_alumni(&self, school_id: Uuid, alumni_id: Uuid) -> AppResult<()> {
        let alumni = self
            .alumni
            .find_by_id(alumni_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("data alumni {alumni_id} tidak ditemukan")))?;
        if alumni.school_id != school_id {
            return Err(AppError::Forbidden("data alumni ini bukan milik sekolah Anda".to_string()));
        }
        Ok(())
    }

    pub async fn list_alumni_rapor(&self, actor: &AuthUser, alumni_id: Uuid) -> AppResult<Vec<AlumniRaporScore>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.ensure_owns_alumni(school_id, alumni_id).await?;
        self.alumni.list_rapor(alumni_id).await
    }

    /// Batch upsert "Detail Rapor" alumni — sama pola dengan `RaporScoreRepository::upsert`
    /// milik siswa aktif (upsert on `(alumni_id, subject)`), tapi tanpa dimensi semester
    /// karena ini snapshot nilai akhir alumni, bukan riwayat per semester.
    pub async fn upsert_alumni_rapor(
        &self,
        actor: &AuthUser,
        alumni_id: Uuid,
        entries: Vec<AlumniRaporScoreUpsert>,
    ) -> AppResult<Vec<AlumniRaporScore>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.ensure_owns_alumni(school_id, alumni_id).await?;
        let mut saved = Vec::with_capacity(entries.len());
        for e in entries {
            if !(0.0..=100.0).contains(&e.score) {
                return Err(AppError::Validation("nilai rapor harus di antara 0 dan 100".to_string()));
            }
            if e.subject.trim().is_empty() {
                return Err(AppError::Validation("nama mata pelajaran wajib diisi".to_string()));
            }
            saved.push(self.alumni.upsert_rapor(alumni_id, e).await?);
        }
        Ok(saved)
    }

    pub async fn delete_alumni_rapor(&self, actor: &AuthUser, alumni_id: Uuid, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.ensure_owns_alumni(school_id, alumni_id).await?;
        let deleted = self.alumni.delete_rapor(alumni_id, id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("nilai rapor alumni {id} tidak ditemukan")));
        }
        Ok(())
    }

    // ─── Eligibility ────────────────────────────────────────────────────────────

    pub async fn list_eligibility(&self, actor: &AuthUser) -> AppResult<Vec<SchoolEligibility>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.eligibility.list_by_school(school_id).await
    }

    /// Sama seperti `list_eligibility`, tapi tiap baris disandingkan dengan jumlah siswa
    /// yang BENAR-BENAR terdaftar aktif SNBP tahun itu (`snbp_participation.aktif = true`
    /// pada tahun yang sama) — dua angka ini sebelumnya hidup independen: `eligible_count`
    /// diinput manual PIC, sementara jumlah terdaftar aktif berasal dari roster nyata
    /// "Daftar Siswa" tanpa pernah dicocokkan satu sama lain. Disandingkan di sini (bukan
    /// disimpan sebagai kolom) supaya angka terdaftarnya selalu live, tidak pernah basi
    /// kalau status aktif siswa berubah setelah eligible_count diinput.
    pub async fn list_eligibility_with_usage(&self, actor: &AuthUser) -> AppResult<Vec<(SchoolEligibility, i64)>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let eligibility = self.eligibility.list_by_school(school_id).await?;
        let participation = self.snbp_participation.list_by_school(school_id).await?;

        let mut aktif_per_year: HashMap<i32, i64> = HashMap::new();
        for p in &participation {
            if p.aktif {
                *aktif_per_year.entry(p.year).or_insert(0) += 1;
            }
        }

        Ok(eligibility
            .into_iter()
            .map(|e| {
                let terdaftar = aktif_per_year.get(&e.year).copied().unwrap_or(0);
                (e, terdaftar)
            })
            .collect())
    }

    pub async fn upsert_eligibility(&self, actor: &AuthUser, year: i32, eligible_count: i32) -> AppResult<SchoolEligibility> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        if eligible_count < 0 {
            return Err(AppError::Validation("jumlah eligible tidak boleh negatif".to_string()));
        }
        self.eligibility.upsert(school_id, SchoolEligibilityUpsert { year, eligible_count }).await
    }

    // ─── SNBT post-exam tracking ("Rekap SNBT") ────────────────────────────────

    /// Every student on the school's roster with at least one real SNBT target, merged
    /// with whatever real post-exam tracking the school has entered so far (status
    /// defaults to "terdaftar" and everything else to `None`/empty for students with no
    /// tracking row yet).
    pub async fn snbt_roster(&self, actor: &AuthUser) -> AppResult<Vec<SnbtRosterRow>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let students = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() })
            .await?;

        let mut out = Vec::new();
        for student in &students {
            let targets = self.ptn_targets.list_by_student(student.id).await?;
            let snbt_targets: Vec<_> = targets.into_iter().filter(|t| t.track == Track::Snbt).collect();
            if snbt_targets.is_empty() {
                continue;
            }
            let sim_run = self.simulation_runs.latest_completed_for_student(student.id).await?;
            let (est_score, est_score_source) = match sim_run.and_then(|r| r.combined_estimate_score) {
                Some(score) => (Some(score), Some("simulasi")),
                None => {
                    let attempts = self.attempts.list_by_student(student.id).await?;
                    let fallback = average_tryout_score(&attempts).map(|s| (s / PLATFORM_SCALE_MAX) * UTBK_SCALE_MAX);
                    let source = fallback.map(|_| "drilling");
                    (fallback, source)
                }
            };

            // Derive pilihan 1/2 live from the real PtnTarget rows (utama first, then
            // cadangan/aman) — same live-derivation pattern as `snbp_roster`'s pilihan_1/2,
            // rather than storing a redundant copy.
            let mut choices: Vec<SnbtChoice> = Vec::new();
            for t in &snbt_targets {
                let Some(program) = self.programs.find_by_id(t.ptn_program_id).await? else { continue };
                choices.push(SnbtChoice {
                    nama_ptn: program.nama_ptn,
                    nama_prodi: program.nama_prodi,
                    priority: t.priority,
                    pg_snbt: program.pg_snbt,
                    daya_tampung_snbt: program.daya_tampung_snbt,
                    peminat_snbt: program.peminat_snbt,
                });
            }
            choices.sort_by_key(|c| match c.priority {
                Priority::Utama => 0,
                Priority::Cadangan => 1,
                Priority::Aman => 2,
            });
            let mut iter = choices.into_iter();
            let pilihan_1 = iter.next();
            let pilihan_2 = iter.next();

            // One real exam result per student (not per target) — see `SnbtTracking` doc.
            let tracking = self.snbt_tracking.get_by_student(student.id).await?;
            out.push(SnbtRosterRow {
                student_id: student.id,
                student_name: student.name.clone(),
                pilihan_1,
                pilihan_2,
                est_score,
                est_score_source,
                actual_score: tracking.as_ref().and_then(|tr| tr.actual_score),
                exam_date: tracking.as_ref().and_then(|tr| tr.exam_date),
                status: tracking.as_ref().map(|tr| tr.status).unwrap_or(SnbtTrackingStatus::Terdaftar),
                notes: tracking.map(|tr| tr.notes).unwrap_or_default(),
            });
        }
        Ok(out)
    }

    /// Rekap SNBT's 4 stat cards — derived purely from `snbt_roster`, real counts only.
    /// Now counts students (one row = one student), not targets, since tracking is
    /// per-student.
    pub async fn snbt_dashboard(&self, actor: &AuthUser) -> AppResult<SnbtDashboard> {
        let roster = self.snbt_roster(actor).await?;
        let terdaftar = roster.len() as i64;
        let sudah_ujian = roster.iter().filter(|r| r.status != SnbtTrackingStatus::Terdaftar).count() as i64;
        let diterima = roster.iter().filter(|r| r.status == SnbtTrackingStatus::Diterima).count() as i64;
        let skor_perlu_diisi =
            roster.iter().filter(|r| r.status != SnbtTrackingStatus::Terdaftar && r.actual_score.is_none()).count() as i64;
        Ok(SnbtDashboard { terdaftar, sudah_ujian, diterima, skor_perlu_diisi })
    }

    /// Rekap SNBT drill-down: every real tryout/drilling/Simulasi UTBK ATTEMPT this student has
    /// made in the app (session title, type, score, submitted date) — distinct from
    /// `snbt_roster`'s `est_score`/`actual_score` (which are a single derived estimate and a
    /// manually-entered real SNBT exam score respectively). Lets the school PIC see the actual
    /// practice history behind a student's estimate rather than just the final number.
    pub async fn student_attempt_history(&self, actor: &AuthUser, student_id: Uuid) -> AppResult<Vec<Attempt>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.ensure_owns_student(school_id, student_id).await?;
        let mut list = self.attempts.list_by_student(student_id).await?;
        list.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(list)
    }

    /// Per-TO detail for the "Riwayat Pengerjaan TO" drill-down: score + subject breakdown for
    /// one specific attempt, so the school PIC can see exactly where a student is strong/weak
    /// on a given tryout instead of just the headline score — the same data the student sees
    /// on their own "Lihat Hasil" screen (`TryoutService::get_result`), just reached via a
    /// School-ownership check instead of a student-ownership check. Verifies `attempt_id`
    /// actually belongs to `student_id` (not just "some student on this school's roster") before
    /// delegating to `TryoutService::result_for_submitted_attempt`, which itself has no
    /// ownership check — this method IS that check.
    pub async fn student_attempt_result(&self, actor: &AuthUser, student_id: Uuid, attempt_id: Uuid) -> AppResult<AttemptResult> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.ensure_owns_student(school_id, student_id).await?;
        self.ensure_attempt_belongs_to_student(attempt_id, student_id).await?;
        self.tryout.result_for_submitted_attempt(attempt_id).await
    }

    /// Per-question pembahasan counterpart to `student_attempt_result` — same ownership
    /// verification, delegates to `TryoutService::review_for_submitted_attempt`.
    pub async fn student_attempt_review(&self, actor: &AuthUser, student_id: Uuid, attempt_id: Uuid) -> AppResult<Vec<ReviewItem>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.ensure_owns_student(school_id, student_id).await?;
        self.ensure_attempt_belongs_to_student(attempt_id, student_id).await?;
        self.tryout.review_for_submitted_attempt(attempt_id).await
    }

    async fn ensure_attempt_belongs_to_student(&self, attempt_id: Uuid, student_id: Uuid) -> AppResult<()> {
        let attempt = self
            .attempts
            .find_by_id(attempt_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("attempt {attempt_id} not found")))?;
        if attempt.student_id != student_id {
            return Err(AppError::Forbidden("attempt ini bukan milik siswa tersebut".to_string()));
        }
        Ok(())
    }

    /// Upserts one student's real exam tracking. Ownership is verified via the same
    /// `ensure_owns_student` check used by `upsert_snbp_participation`.
    pub async fn upsert_snbt_tracking(
        &self,
        actor: &AuthUser,
        student_id: Uuid,
        input: SnbtTrackingInput,
    ) -> AppResult<SnbtRosterRow> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.ensure_owns_student(school_id, student_id).await?;
        if let Some(score) = input.actual_score {
            if !(0.0..=1000.0).contains(&score) {
                return Err(AppError::Validation("skor aktual harus di antara 0 dan 1000".to_string()));
            }
        }
        self.snbt_tracking
            .upsert(
                student_id,
                SnbtTrackingUpsert {
                    actual_score: input.actual_score,
                    exam_date: input.exam_date,
                    status: input.status,
                    notes: input.notes,
                },
            )
            .await?;

        let updated = self.snbt_roster(actor).await?;
        updated
            .into_iter()
            .find(|r| r.student_id == student_id)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("baris rekap SNBT hilang setelah disimpan")))
    }

    async fn resolve_own_school_id(&self, actor: &AuthUser) -> AppResult<Uuid> {
        let me = self
            .users
            .find_by_id(actor.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("akun tidak ditemukan".to_string()))?;
        me.school_id
            .ok_or_else(|| AppError::Validation("akun sekolah ini belum terhubung ke data sekolah".to_string()))
    }

    // ─── Import nilai rapor massal (banyak siswa sekaligus, dari 1 file Excel) ─────────
    // Dicocokkan lewat NIS (bukan nama, yang gampang ambigu antar-siswa) terhadap roster
    // siswa sekolah ini SAJA — NIS di sekolah lain tidak pernah bisa "nyasar" menimpa nilai
    // siswa yang bukan bagian dari sekolah pemilik file, karena baris roster yang dipakai
    // untuk pencocokan sudah difilter `school_id` milik actor sejak query pertama. Baris
    // yang NIS-nya tidak ketemu, nilainya di luar 0-100, atau nama mapelnya kosong TIDAK
    // menggagalkan seluruh import — baris itu dikembalikan sebagai `errors` sementara
    // baris lain yang valid tetap tersimpan (partial success), supaya satu baris typo tidak
    // memaksa PIC sekolah mengulang dari nol.
    pub async fn bulk_import_rapor(&self, actor: &AuthUser, entries: Vec<BulkRaporEntryInput>) -> AppResult<BulkRaporImportResult> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let students = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() })
            .await?;
        let mut by_nis: HashMap<String, Uuid> = HashMap::new();
        for s in &students {
            if let Some(nis) = s.nis.as_deref() {
                let key = nis.trim();
                if !key.is_empty() {
                    by_nis.insert(key.to_string(), s.id);
                }
            }
        }

        let mut saved_count: i64 = 0;
        let mut errors: Vec<BulkRaporImportError> = Vec::new();
        for e in entries {
            let nis_key = e.nis.trim().to_string();
            let Some(&student_id) = by_nis.get(&nis_key) else {
                errors.push(BulkRaporImportError {
                    nis: e.nis, semester: e.semester, subject: e.subject,
                    reason: "NIS tidak ditemukan di antara siswa sekolah ini".to_string(),
                });
                continue;
            };
            if e.subject.trim().is_empty() {
                errors.push(BulkRaporImportError { nis: e.nis, semester: e.semester, subject: e.subject, reason: "nama mata pelajaran kosong".to_string() });
                continue;
            }
            if !(0.0..=100.0).contains(&e.score) {
                errors.push(BulkRaporImportError { nis: e.nis, semester: e.semester, subject: e.subject, reason: "nilai harus di antara 0 dan 100".to_string() });
                continue;
            }
            if !(1..=6).contains(&e.semester) {
                errors.push(BulkRaporImportError { nis: e.nis, semester: e.semester, subject: e.subject, reason: "semester harus antara 1 dan 6".to_string() });
                continue;
            }
            match self
                .rapor
                .upsert(student_id, RaporScoreUpsert { semester: e.semester, subject: e.subject.clone(), score: e.score, is_minat: e.is_minat })
                .await
            {
                Ok(_) => saved_count += 1,
                Err(err) => errors.push(BulkRaporImportError { nis: e.nis, semester: e.semester, subject: e.subject, reason: err.to_string() }),
            }
        }
        Ok(BulkRaporImportResult { saved_count, errors })
    }
}

pub struct BulkRaporEntryInput {
    pub nis: String,
    pub semester: i32,
    pub subject: String,
    pub score: f64,
    pub is_minat: bool,
}

pub struct BulkRaporImportError {
    pub nis: String,
    pub semester: i32,
    pub subject: String,
    pub reason: String,
}

pub struct BulkRaporImportResult {
    pub saved_count: i64,
    pub errors: Vec<BulkRaporImportError>,
}

/// Flat average across all of the student's real rapor entries, regardless of subject —
/// used ONLY for the ranking table's display column (one number per student, not tied to
/// any specific target). The actual chance estimate per target uses
/// `average_rapor_for_program` instead (blended toward that program's required subjects).
fn average_rapor(rapor: &[RaporScore]) -> Option<f64> {
    if rapor.is_empty() {
        return None;
    }
    Some(rapor.iter().map(|r| r.score).sum::<f64>() / rapor.len() as f64)
}

/// Average of the student's real, submitted tryout/drilling attempt scores (0-1000
/// platform scale) — mirrors `rationalization_service::average_tryout_score` (kept as a
/// small local duplicate for the same reason as `average_rapor` above).
fn average_tryout_score(attempts: &[Attempt]) -> Option<f64> {
    let scored: Vec<i32> = attempts.iter().filter_map(|a| a.score).collect();
    if scored.is_empty() {
        return None;
    }
    Some(scored.iter().map(|s| *s as f64).sum::<f64>() / scored.len() as f64)
}
