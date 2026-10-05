use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use crate::domain::rationalization::{
    Achievement, AchievementLevel, AlumniBenchmark, AlumniRaporScore, AuditLogEntry, PtnProgram, PtnTarget, Priority,
    RaporScore, SchoolEligibility, SnbpParticipation, SnbpStatus, SnbtTracking, SnbtTrackingStatus, Track,
};
use crate::domain::repository::{
    AchievementRepository, AlumniBenchmarkRepository, AlumniRaporScoreUpsert, AuditLogRepository, NewAchievement,
    NewAlumniBenchmark, NewAuditLogEntry, NewPtnProgram, NewPtnTarget, PtnProgramFilter, PtnProgramRepository,
    PtnProgramUpdate, PtnTargetRepository, RaporScoreRepository, RaporScoreUpsert, SchoolEligibilityRepository,
    SchoolEligibilityUpsert, SnbpParticipationRepository, SnbpParticipationUpsert, SnbtTrackingRepository,
    SnbtTrackingUpsert,
};
use crate::error::{AppError, AppResult};

// ─── PTN Program catalog ────────────────────────────────────────────────────────

pub struct PostgresPtnProgramRepository {
    pool: PgPool,
}

impl PostgresPtnProgramRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PtnProgramRow {
    id: Uuid,
    kode: i64,
    nama_ptn: String,
    nama_prodi: String,
    provinsi: String,
    kota: String,
    singkatan: String,
    rumpun: String,
    mapel_syarat: String,
    daya_tampung_snbp: i32,
    peminat_snbp: i32,
    daya_tampung_snbt: i32,
    peminat_snbt: i32,
    pg_snbt: f64,
    pg_snbp: f64,
    jenjang: String,
    has_official_stats: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PtnProgramRow> for PtnProgram {
    fn from(r: PtnProgramRow) -> Self {
        PtnProgram {
            id: r.id,
            kode: r.kode,
            nama_ptn: r.nama_ptn,
            nama_prodi: r.nama_prodi,
            provinsi: r.provinsi,
            kota: r.kota,
            singkatan: r.singkatan,
            rumpun: r.rumpun,
            mapel_syarat: r.mapel_syarat,
            daya_tampung_snbp: r.daya_tampung_snbp,
            peminat_snbp: r.peminat_snbp,
            daya_tampung_snbt: r.daya_tampung_snbt,
            peminat_snbt: r.peminat_snbt,
            pg_snbt: r.pg_snbt,
            pg_snbp: r.pg_snbp,
            jenjang: r.jenjang,
            has_official_stats: r.has_official_stats,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

const SELECT_PTN_PROGRAM: &str = r#"
    SELECT id, kode, nama_ptn, nama_prodi, provinsi, kota, singkatan, rumpun, mapel_syarat,
           daya_tampung_snbp, peminat_snbp, daya_tampung_snbt, peminat_snbt, pg_snbt, pg_snbp,
           jenjang, has_official_stats, created_at, updated_at
    FROM ptn_programs
"#;

#[async_trait]
impl PtnProgramRepository for PostgresPtnProgramRepository {
    async fn upsert_by_kode(&self, n: NewPtnProgram) -> AppResult<PtnProgram> {
        let row = sqlx::query_as::<_, PtnProgramRow>(&format!(
            r#"INSERT INTO ptn_programs
                (kode, nama_ptn, nama_prodi, provinsi, kota, singkatan, rumpun, mapel_syarat,
                 daya_tampung_snbp, peminat_snbp, daya_tampung_snbt, peminat_snbt, pg_snbt, pg_snbp, jenjang,
                 has_official_stats)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
               ON CONFLICT (kode) DO UPDATE SET
                 nama_ptn = EXCLUDED.nama_ptn, nama_prodi = EXCLUDED.nama_prodi,
                 provinsi = EXCLUDED.provinsi, kota = EXCLUDED.kota, singkatan = EXCLUDED.singkatan,
                 rumpun = EXCLUDED.rumpun, mapel_syarat = EXCLUDED.mapel_syarat,
                 daya_tampung_snbp = EXCLUDED.daya_tampung_snbp, peminat_snbp = EXCLUDED.peminat_snbp,
                 daya_tampung_snbt = EXCLUDED.daya_tampung_snbt, peminat_snbt = EXCLUDED.peminat_snbt,
                 pg_snbt = EXCLUDED.pg_snbt, pg_snbp = EXCLUDED.pg_snbp, jenjang = EXCLUDED.jenjang,
                 has_official_stats = EXCLUDED.has_official_stats,
                 updated_at = now()
               RETURNING id, kode, nama_ptn, nama_prodi, provinsi, kota, singkatan, rumpun, mapel_syarat,
                         daya_tampung_snbp, peminat_snbp, daya_tampung_snbt, peminat_snbt, pg_snbt, pg_snbp,
                         jenjang, has_official_stats, created_at, updated_at"#
        ))
        .bind(n.kode)
        .bind(&n.nama_ptn)
        .bind(&n.nama_prodi)
        .bind(&n.provinsi)
        .bind(&n.kota)
        .bind(&n.singkatan)
        .bind(&n.rumpun)
        .bind(&n.mapel_syarat)
        .bind(n.daya_tampung_snbp)
        .bind(n.peminat_snbp)
        .bind(n.daya_tampung_snbt)
        .bind(n.peminat_snbt)
        .bind(n.pg_snbt)
        .bind(n.pg_snbp)
        .bind(&n.jenjang)
        .bind(n.has_official_stats)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn create(&self, n: NewPtnProgram) -> AppResult<PtnProgram> {
        // Admin-added rows go through the same upsert-by-kode path; the service layer is
        // responsible for generating a fresh, non-colliding `kode` for manually-added
        // programs (the imported catalog's codes are only meaningful for the CSV source).
        self.upsert_by_kode(n).await
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<PtnProgram>> {
        let row = sqlx::query_as::<_, PtnProgramRow>(&format!("{SELECT_PTN_PROGRAM} WHERE id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(PtnProgram::from))
    }

    async fn list(&self, filter: PtnProgramFilter) -> AppResult<(Vec<PtnProgram>, i64)> {
        let page = filter.page.max(1);
        let page_size = filter.page_size.clamp(1, 200);
        let offset = (page - 1) * page_size;

        // Kata kunci dipecah PER KATA, bukan dicocokkan sebagai satu frasa utuh — supaya
        // pencarian gabungan "nama PTN + nama prodi" dalam satu kotak (mis. "UGM Kedokteran"
        // atau "Teknik Informatika ITB") tetap ketemu. Query lama mensyaratkan SELURUH string
        // yang diketik jadi satu substring yang sama persis di nama_ptn ATAU nama_prodi — yang
        // mustahil kalau frasanya menyeberang dua kolom berbeda (nama_ptn cuma berisi nama
        // universitas, nama_prodi cuma berisi nama prodi), jadi selalu 0 hasil meski datanya
        // ADA. Sekarang: tiap kata WAJIB muncul di nama_ptn, nama_prodi, ATAU singkatan (AND
        // antar-kata, OR antar-kolom per kata) — "UGM Kedokteran" jadi cocok karena "UGM"
        // ketemu di kolom singkatan (atau nama_ptn) dan "Kedokteran" ketemu di nama_prodi,
        // tanpa peduli urutan katanya. Kolom singkatan turut disertakan karena orang lazim
        // mencari pakai singkatan kampus (ITB, UGM, IPB, dst), bukan nama panjangnya.
        let words: Vec<String> = filter
            .search
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .split_whitespace()
            .map(|w| format!("%{w}%"))
            .collect();

        let mut qb = QueryBuilder::new(format!("{SELECT_PTN_PROGRAM} WHERE 1=1"));
        let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM ptn_programs WHERE 1=1");
        for word in &words {
            qb.push(" AND (lower(nama_ptn) LIKE ").push_bind(word.clone())
                .push(" OR lower(nama_prodi) LIKE ").push_bind(word.clone())
                .push(" OR lower(singkatan) LIKE ").push_bind(word.clone()).push(")");
            count_qb.push(" AND (lower(nama_ptn) LIKE ").push_bind(word.clone())
                .push(" OR lower(nama_prodi) LIKE ").push_bind(word.clone())
                .push(" OR lower(singkatan) LIKE ").push_bind(word.clone()).push(")");
        }
        if let Some(rumpun) = &filter.rumpun {
            qb.push(" AND rumpun = ").push_bind(rumpun.clone());
            count_qb.push(" AND rumpun = ").push_bind(rumpun.clone());
        }
        if let Some(jenjang) = &filter.jenjang {
            qb.push(" AND jenjang = ").push_bind(jenjang.clone());
            count_qb.push(" AND jenjang = ").push_bind(jenjang.clone());
        }
        qb.push(" ORDER BY nama_ptn ASC, nama_prodi ASC LIMIT ")
            .push_bind(page_size)
            .push(" OFFSET ")
            .push_bind(offset);

        let rows: Vec<PtnProgramRow> = qb.build_query_as().fetch_all(&self.pool).await?;
        let total: i64 = count_qb.build_query_scalar().fetch_one(&self.pool).await?;

        Ok((rows.into_iter().map(PtnProgram::from).collect(), total))
    }

    async fn update(&self, id: Uuid, u: PtnProgramUpdate) -> AppResult<Option<PtnProgram>> {
        let result = sqlx::query(
            r#"UPDATE ptn_programs SET
                nama_ptn = $2, nama_prodi = $3, provinsi = $4, kota = $5, singkatan = $6,
                rumpun = $7, mapel_syarat = $8, daya_tampung_snbp = $9, peminat_snbp = $10,
                daya_tampung_snbt = $11, peminat_snbt = $12, pg_snbt = $13, pg_snbp = $14,
                jenjang = $15, has_official_stats = $16, updated_at = now()
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(&u.nama_ptn)
        .bind(&u.nama_prodi)
        .bind(&u.provinsi)
        .bind(&u.kota)
        .bind(&u.singkatan)
        .bind(&u.rumpun)
        .bind(&u.mapel_syarat)
        .bind(u.daya_tampung_snbp)
        .bind(u.peminat_snbp)
        .bind(u.daya_tampung_snbt)
        .bind(u.peminat_snbt)
        .bind(u.pg_snbt)
        .bind(u.pg_snbp)
        .bind(&u.jenjang)
        .bind(u.has_official_stats)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM ptn_programs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::from_sqlx_delete(e, "program masih menjadi target siswa aktif"))?;
        Ok(result.rows_affected() > 0)
    }

    async fn count_all(&self) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ptn_programs")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    async fn distinct_rumpun(&self) -> AppResult<Vec<String>> {
        let rows: Vec<String> =
            sqlx::query_scalar("SELECT DISTINCT rumpun FROM ptn_programs ORDER BY rumpun ASC")
                .fetch_all(&self.pool)
                .await?;
        Ok(rows)
    }
}

// ─── Student rapor scores ───────────────────────────────────────────────────────

pub struct PostgresRaporScoreRepository {
    pool: PgPool,
}

impl PostgresRaporScoreRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RaporScoreRow {
    id: Uuid,
    student_id: Uuid,
    semester: i32,
    subject: String,
    score: f64,
    is_minat: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<RaporScoreRow> for RaporScore {
    fn from(r: RaporScoreRow) -> Self {
        RaporScore {
            id: r.id,
            student_id: r.student_id,
            semester: r.semester,
            subject: r.subject,
            score: r.score,
            is_minat: r.is_minat,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl RaporScoreRepository for PostgresRaporScoreRepository {
    async fn upsert(&self, student_id: Uuid, entry: RaporScoreUpsert) -> AppResult<RaporScore> {
        let row = sqlx::query_as::<_, RaporScoreRow>(
            r#"INSERT INTO student_rapor_scores (student_id, semester, subject, score, is_minat)
               VALUES ($1,$2,$3,$4,$5)
               ON CONFLICT (student_id, semester, subject) DO UPDATE SET
                 score = EXCLUDED.score, is_minat = EXCLUDED.is_minat, updated_at = now()
               RETURNING id, student_id, semester, subject, score, is_minat, created_at, updated_at"#,
        )
        .bind(student_id)
        .bind(entry.semester)
        .bind(&entry.subject)
        .bind(entry.score)
        .bind(entry.is_minat)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn list_by_student(&self, student_id: Uuid) -> AppResult<Vec<RaporScore>> {
        let rows = sqlx::query_as::<_, RaporScoreRow>(
            "SELECT id, student_id, semester, subject, score, is_minat, created_at, updated_at
             FROM student_rapor_scores WHERE student_id = $1 ORDER BY semester ASC, subject ASC",
        )
        .bind(student_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(RaporScore::from).collect())
    }

    async fn delete(&self, student_id: Uuid, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM student_rapor_scores WHERE id = $1 AND student_id = $2")
            .bind(id)
            .bind(student_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

// ─── Student PTN targets ────────────────────────────────────────────────────────

pub struct PostgresPtnTargetRepository {
    pool: PgPool,
}

impl PostgresPtnTargetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PtnTargetRow {
    id: Uuid,
    student_id: Uuid,
    ptn_program_id: Uuid,
    track: String,
    priority: String,
    sort_order: i32,
    created_at: DateTime<Utc>,
}

impl TryFrom<PtnTargetRow> for PtnTarget {
    type Error = AppError;
    fn try_from(r: PtnTargetRow) -> Result<Self, Self::Error> {
        Ok(PtnTarget {
            id: r.id,
            student_id: r.student_id,
            ptn_program_id: r.ptn_program_id,
            track: r
                .track
                .parse::<Track>()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt track: {e}")))?,
            priority: r
                .priority
                .parse::<Priority>()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt priority: {e}")))?,
            sort_order: r.sort_order,
            created_at: r.created_at,
        })
    }
}

#[async_trait]
impl PtnTargetRepository for PostgresPtnTargetRepository {
    async fn create(&self, n: NewPtnTarget) -> AppResult<PtnTarget> {
        let row = sqlx::query_as::<_, PtnTargetRow>(
            r#"INSERT INTO student_ptn_targets (student_id, ptn_program_id, track, priority, sort_order)
               VALUES ($1,$2,$3,$4,$5)
               ON CONFLICT (student_id, ptn_program_id, track) DO UPDATE SET
                 priority = EXCLUDED.priority, sort_order = EXCLUDED.sort_order
               RETURNING id, student_id, ptn_program_id, track, priority, sort_order, created_at"#,
        )
        .bind(n.student_id)
        .bind(n.ptn_program_id)
        .bind(n.track.as_str())
        .bind(n.priority.as_str())
        .bind(n.sort_order)
        .fetch_one(&self.pool)
        .await?;
        PtnTarget::try_from(row)
    }

    async fn list_by_student(&self, student_id: Uuid) -> AppResult<Vec<PtnTarget>> {
        let rows = sqlx::query_as::<_, PtnTargetRow>(
            "SELECT id, student_id, ptn_program_id, track, priority, sort_order, created_at
             FROM student_ptn_targets WHERE student_id = $1 ORDER BY sort_order ASC, created_at ASC",
        )
        .bind(student_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(PtnTarget::try_from).collect()
    }

    async fn set_priority(&self, student_id: Uuid, id: Uuid, priority: Priority) -> AppResult<Option<PtnTarget>> {
        let row = sqlx::query_as::<_, PtnTargetRow>(
            r#"UPDATE student_ptn_targets SET priority = $3 WHERE id = $1 AND student_id = $2
               RETURNING id, student_id, ptn_program_id, track, priority, sort_order, created_at"#,
        )
        .bind(id)
        .bind(student_id)
        .bind(priority.as_str())
        .fetch_optional(&self.pool)
        .await?;
        row.map(PtnTarget::try_from).transpose()
    }

    async fn delete(&self, student_id: Uuid, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM student_ptn_targets WHERE id = $1 AND student_id = $2")
            .bind(id)
            .bind(student_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

// ─── School alumni benchmarks ("Kaka Kelas") ────────────────────────────────────

pub struct PostgresAlumniBenchmarkRepository {
    pool: PgPool,
}

impl PostgresAlumniBenchmarkRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AlumniBenchmarkRow {
    id: Uuid,
    school_id: Uuid,
    alumni_name: String,
    graduation_year: i32,
    track: String,
    nama_ptn: String,
    nama_prodi: String,
    benchmark_score: f64,
    created_by: Uuid,
    created_at: DateTime<Utc>,
}

impl TryFrom<AlumniBenchmarkRow> for AlumniBenchmark {
    type Error = AppError;
    fn try_from(r: AlumniBenchmarkRow) -> Result<Self, Self::Error> {
        Ok(AlumniBenchmark {
            id: r.id,
            school_id: r.school_id,
            alumni_name: r.alumni_name,
            graduation_year: r.graduation_year,
            track: r
                .track
                .parse::<Track>()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt track: {e}")))?,
            nama_ptn: r.nama_ptn,
            nama_prodi: r.nama_prodi,
            benchmark_score: r.benchmark_score,
            created_by: r.created_by,
            created_at: r.created_at,
        })
    }
}

#[async_trait]
impl AlumniBenchmarkRepository for PostgresAlumniBenchmarkRepository {
    async fn create(&self, n: NewAlumniBenchmark) -> AppResult<AlumniBenchmark> {
        let row = sqlx::query_as::<_, AlumniBenchmarkRow>(
            r#"INSERT INTO school_alumni_benchmarks
                (school_id, alumni_name, graduation_year, track, nama_ptn, nama_prodi, benchmark_score, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
               RETURNING id, school_id, alumni_name, graduation_year, track, nama_ptn, nama_prodi,
                         benchmark_score, created_by, created_at"#,
        )
        .bind(n.school_id)
        .bind(&n.alumni_name)
        .bind(n.graduation_year)
        .bind(n.track.as_str())
        .bind(&n.nama_ptn)
        .bind(&n.nama_prodi)
        .bind(n.benchmark_score)
        .bind(n.created_by)
        .fetch_one(&self.pool)
        .await?;
        AlumniBenchmark::try_from(row)
    }

    async fn list_by_school(&self, school_id: Uuid) -> AppResult<Vec<AlumniBenchmark>> {
        let rows = sqlx::query_as::<_, AlumniBenchmarkRow>(
            "SELECT id, school_id, alumni_name, graduation_year, track, nama_ptn, nama_prodi,
                    benchmark_score, created_by, created_at
             FROM school_alumni_benchmarks WHERE school_id = $1 ORDER BY graduation_year DESC, alumni_name ASC",
        )
        .bind(school_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(AlumniBenchmark::try_from).collect()
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<AlumniBenchmark>> {
        let row = sqlx::query_as::<_, AlumniBenchmarkRow>(
            "SELECT id, school_id, alumni_name, graduation_year, track, nama_ptn, nama_prodi,
                    benchmark_score, created_by, created_at
             FROM school_alumni_benchmarks WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(AlumniBenchmark::try_from).transpose()
    }

    async fn delete(&self, school_id: Uuid, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM school_alumni_benchmarks WHERE id = $1 AND school_id = $2")
            .bind(id)
            .bind(school_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn upsert_rapor(&self, alumni_id: Uuid, entry: AlumniRaporScoreUpsert) -> AppResult<AlumniRaporScore> {
        let row = sqlx::query_as::<_, AlumniRaporScoreRow>(
            r#"INSERT INTO alumni_rapor_scores (alumni_id, subject, score)
               VALUES ($1, $2, $3)
               ON CONFLICT (alumni_id, subject) DO UPDATE SET score = EXCLUDED.score, updated_at = now()
               RETURNING id, alumni_id, subject, score, created_at, updated_at"#,
        )
        .bind(alumni_id)
        .bind(&entry.subject)
        .bind(entry.score)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn list_rapor(&self, alumni_id: Uuid) -> AppResult<Vec<AlumniRaporScore>> {
        let rows = sqlx::query_as::<_, AlumniRaporScoreRow>(
            "SELECT id, alumni_id, subject, score, created_at, updated_at
             FROM alumni_rapor_scores WHERE alumni_id = $1 ORDER BY subject ASC",
        )
        .bind(alumni_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete_rapor(&self, alumni_id: Uuid, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM alumni_rapor_scores WHERE id = $1 AND alumni_id = $2")
            .bind(id)
            .bind(alumni_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[derive(sqlx::FromRow)]
struct AlumniRaporScoreRow {
    id: Uuid,
    alumni_id: Uuid,
    subject: String,
    score: f64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<AlumniRaporScoreRow> for AlumniRaporScore {
    fn from(r: AlumniRaporScoreRow) -> Self {
        Self { id: r.id, alumni_id: r.alumni_id, subject: r.subject, score: r.score, created_at: r.created_at, updated_at: r.updated_at }
    }
}

// ─── SNBT post-exam tracking ("Rekap SNBT") ─────────────────────────────────────

pub struct PostgresSnbtTrackingRepository {
    pool: PgPool,
}

impl PostgresSnbtTrackingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SnbtTrackingRow {
    id: Uuid,
    student_id: Uuid,
    actual_score: Option<f64>,
    exam_date: Option<NaiveDate>,
    status: String,
    notes: String,
    updated_at: DateTime<Utc>,
}

impl TryFrom<SnbtTrackingRow> for SnbtTracking {
    type Error = AppError;
    fn try_from(r: SnbtTrackingRow) -> Result<Self, Self::Error> {
        Ok(SnbtTracking {
            id: r.id,
            student_id: r.student_id,
            actual_score: r.actual_score,
            exam_date: r.exam_date,
            status: r
                .status
                .parse::<SnbtTrackingStatus>()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt snbt tracking status: {e}")))?,
            notes: r.notes,
            updated_at: r.updated_at,
        })
    }
}

#[async_trait]
impl SnbtTrackingRepository for PostgresSnbtTrackingRepository {
    async fn upsert(&self, student_id: Uuid, entry: SnbtTrackingUpsert) -> AppResult<SnbtTracking> {
        let row = sqlx::query_as::<_, SnbtTrackingRow>(
            r#"INSERT INTO student_snbt_tracking (student_id, actual_score, exam_date, status, notes)
               VALUES ($1,$2,$3,$4,$5)
               ON CONFLICT (student_id) DO UPDATE SET
                 actual_score = EXCLUDED.actual_score, exam_date = EXCLUDED.exam_date,
                 status = EXCLUDED.status, notes = EXCLUDED.notes, updated_at = now()
               RETURNING id, student_id, actual_score, exam_date, status, notes, updated_at"#,
        )
        .bind(student_id)
        .bind(entry.actual_score)
        .bind(entry.exam_date)
        .bind(entry.status.as_str())
        .bind(&entry.notes)
        .fetch_one(&self.pool)
        .await?;
        SnbtTracking::try_from(row)
    }

    async fn get_by_student(&self, student_id: Uuid) -> AppResult<Option<SnbtTracking>> {
        let row = sqlx::query_as::<_, SnbtTrackingRow>(
            "SELECT id, student_id, actual_score, exam_date, status, notes, updated_at
             FROM student_snbt_tracking WHERE student_id = $1",
        )
        .bind(student_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(SnbtTracking::try_from).transpose()
    }
}

// ─── SNBP roster metadata ("Daftar Siswa") ──────────────────────────────────────

pub struct PostgresSnbpParticipationRepository {
    pool: PgPool,
}

impl PostgresSnbpParticipationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SnbpParticipationRow {
    id: Uuid,
    student_id: Uuid,
    school_id: Uuid,
    year: i32,
    konsultan: String,
    status: String,
    aktif: bool,
    updated_at: DateTime<Utc>,
}

impl TryFrom<SnbpParticipationRow> for SnbpParticipation {
    type Error = AppError;
    fn try_from(r: SnbpParticipationRow) -> Result<Self, Self::Error> {
        Ok(SnbpParticipation {
            id: r.id,
            student_id: r.student_id,
            school_id: r.school_id,
            year: r.year,
            konsultan: r.konsultan,
            status: r
                .status
                .parse::<SnbpStatus>()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt snbp participation status: {e}")))?,
            aktif: r.aktif,
            updated_at: r.updated_at,
        })
    }
}

#[async_trait]
impl SnbpParticipationRepository for PostgresSnbpParticipationRepository {
    async fn upsert(&self, student_id: Uuid, entry: SnbpParticipationUpsert) -> AppResult<SnbpParticipation> {
        let row = sqlx::query_as::<_, SnbpParticipationRow>(
            r#"INSERT INTO snbp_participation (student_id, school_id, year, konsultan, status, aktif)
               VALUES ($1,$2,$3,$4,$5,$6)
               ON CONFLICT (student_id) DO UPDATE SET
                 year = EXCLUDED.year, konsultan = EXCLUDED.konsultan,
                 status = EXCLUDED.status, aktif = EXCLUDED.aktif, updated_at = now()
               RETURNING id, student_id, school_id, year, konsultan, status, aktif, updated_at"#,
        )
        .bind(student_id)
        .bind(entry.school_id)
        .bind(entry.year)
        .bind(&entry.konsultan)
        .bind(entry.status.as_str())
        .bind(entry.aktif)
        .fetch_one(&self.pool)
        .await?;
        SnbpParticipation::try_from(row)
    }

    async fn get_by_student(&self, student_id: Uuid) -> AppResult<Option<SnbpParticipation>> {
        let row = sqlx::query_as::<_, SnbpParticipationRow>(
            "SELECT id, student_id, school_id, year, konsultan, status, aktif, updated_at
             FROM snbp_participation WHERE student_id = $1",
        )
        .bind(student_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(SnbpParticipation::try_from).transpose()
    }

    async fn list_by_school(&self, school_id: Uuid) -> AppResult<Vec<SnbpParticipation>> {
        let rows = sqlx::query_as::<_, SnbpParticipationRow>(
            "SELECT id, student_id, school_id, year, konsultan, status, aktif, updated_at
             FROM snbp_participation WHERE school_id = $1",
        )
        .bind(school_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(SnbpParticipation::try_from).collect()
    }
}

// ─── School SNBP eligibility ("Eligible") ───────────────────────────────────────

pub struct PostgresSchoolEligibilityRepository {
    pool: PgPool,
}

impl PostgresSchoolEligibilityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SchoolEligibilityRow {
    id: Uuid,
    school_id: Uuid,
    year: i32,
    eligible_count: i32,
    updated_at: DateTime<Utc>,
}

impl From<SchoolEligibilityRow> for SchoolEligibility {
    fn from(r: SchoolEligibilityRow) -> Self {
        SchoolEligibility { id: r.id, school_id: r.school_id, year: r.year, eligible_count: r.eligible_count, updated_at: r.updated_at }
    }
}

#[async_trait]
impl SchoolEligibilityRepository for PostgresSchoolEligibilityRepository {
    async fn upsert(&self, school_id: Uuid, entry: SchoolEligibilityUpsert) -> AppResult<SchoolEligibility> {
        let row = sqlx::query_as::<_, SchoolEligibilityRow>(
            r#"INSERT INTO school_snbp_eligibility (school_id, year, eligible_count)
               VALUES ($1,$2,$3)
               ON CONFLICT (school_id, year) DO UPDATE SET
                 eligible_count = EXCLUDED.eligible_count, updated_at = now()
               RETURNING id, school_id, year, eligible_count, updated_at"#,
        )
        .bind(school_id)
        .bind(entry.year)
        .bind(entry.eligible_count)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn list_by_school(&self, school_id: Uuid) -> AppResult<Vec<SchoolEligibility>> {
        let rows = sqlx::query_as::<_, SchoolEligibilityRow>(
            "SELECT id, school_id, year, eligible_count, updated_at
             FROM school_snbp_eligibility WHERE school_id = $1 ORDER BY year DESC",
        )
        .bind(school_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(SchoolEligibility::from).collect())
    }
}

// ─── Student achievements ("Prestasi") ───────────────────────────────────────────

pub struct PostgresAchievementRepository {
    pool: PgPool,
}

impl PostgresAchievementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AchievementRow {
    id: Uuid,
    student_id: Uuid,
    nama: String,
    tingkat: String,
    tahun: i32,
    juara: String,
    certificate_url: Option<String>,
    created_at: DateTime<Utc>,
}

impl TryFrom<AchievementRow> for Achievement {
    type Error = AppError;
    fn try_from(r: AchievementRow) -> Result<Self, Self::Error> {
        Ok(Achievement {
            id: r.id,
            student_id: r.student_id,
            nama: r.nama,
            tingkat: r
                .tingkat
                .parse::<AchievementLevel>()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt achievement level: {e}")))?,
            tahun: r.tahun,
            juara: r.juara,
            certificate_url: r.certificate_url,
            created_at: r.created_at,
        })
    }
}

#[async_trait]
impl AchievementRepository for PostgresAchievementRepository {
    async fn create(&self, n: NewAchievement) -> AppResult<Achievement> {
        let row = sqlx::query_as::<_, AchievementRow>(
            r#"INSERT INTO student_achievements (student_id, nama, tingkat, tahun, juara)
               VALUES ($1,$2,$3,$4,$5)
               RETURNING id, student_id, nama, tingkat, tahun, juara, certificate_url, created_at"#,
        )
        .bind(n.student_id)
        .bind(&n.nama)
        .bind(n.tingkat.as_str())
        .bind(n.tahun)
        .bind(&n.juara)
        .fetch_one(&self.pool)
        .await?;
        Achievement::try_from(row)
    }

    async fn list_by_student(&self, student_id: Uuid) -> AppResult<Vec<Achievement>> {
        let rows = sqlx::query_as::<_, AchievementRow>(
            "SELECT id, student_id, nama, tingkat, tahun, juara, certificate_url, created_at
             FROM student_achievements WHERE student_id = $1 ORDER BY tahun DESC, created_at DESC",
        )
        .bind(student_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Achievement::try_from).collect()
    }

    async fn delete(&self, student_id: Uuid, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM student_achievements WHERE id = $1 AND student_id = $2")
            .bind(id)
            .bind(student_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_certificate(&self, student_id: Uuid, id: Uuid, certificate_url: Option<String>) -> AppResult<Option<Achievement>> {
        let row = sqlx::query_as::<_, AchievementRow>(
            r#"UPDATE student_achievements SET certificate_url = $3 WHERE id = $1 AND student_id = $2
               RETURNING id, student_id, nama, tingkat, tahun, juara, certificate_url, created_at"#,
        )
        .bind(id)
        .bind(student_id)
        .bind(&certificate_url)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Achievement::try_from).transpose()
    }
}

// ─── Audit trail ("Riwayat Perubahan") for edit-on-behalf actions ──────────────────

pub struct PostgresAuditLogRepository {
    pool: PgPool,
}

impl PostgresAuditLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AuditLogRow {
    id: Uuid,
    actor_id: Uuid,
    actor_name: String,
    actor_role: String,
    student_id: Uuid,
    action: String,
    entity_type: String,
    summary: String,
    created_at: DateTime<Utc>,
}

impl From<AuditLogRow> for AuditLogEntry {
    fn from(r: AuditLogRow) -> Self {
        AuditLogEntry {
            id: r.id,
            actor_id: r.actor_id,
            actor_name: r.actor_name,
            actor_role: r.actor_role,
            student_id: r.student_id,
            action: r.action,
            entity_type: r.entity_type,
            summary: r.summary,
            created_at: r.created_at,
        }
    }
}

#[async_trait]
impl AuditLogRepository for PostgresAuditLogRepository {
    async fn record(&self, entry: NewAuditLogEntry) -> AppResult<AuditLogEntry> {
        let row = sqlx::query_as::<_, AuditLogRow>(
            r#"INSERT INTO audit_log (actor_id, actor_name, actor_role, student_id, action, entity_type, summary)
               VALUES ($1,$2,$3,$4,$5,$6,$7)
               RETURNING id, actor_id, actor_name, actor_role, student_id, action, entity_type, summary, created_at"#,
        )
        .bind(entry.actor_id)
        .bind(&entry.actor_name)
        .bind(&entry.actor_role)
        .bind(entry.student_id)
        .bind(&entry.action)
        .bind(&entry.entity_type)
        .bind(&entry.summary)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn list_by_student(&self, student_id: Uuid, limit: i64) -> AppResult<Vec<AuditLogEntry>> {
        let rows = sqlx::query_as::<_, AuditLogRow>(
            "SELECT id, actor_id, actor_name, actor_role, student_id, action, entity_type, summary, created_at
             FROM audit_log WHERE student_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(student_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(AuditLogEntry::from).collect())
    }
}
