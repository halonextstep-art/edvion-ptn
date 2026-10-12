//! Postgres implementation of the Rasionalisasi SNBT/SNBP repositories.
//!
//! `ptn.institutions`/`ptn.ptn_programs` are a shared reference catalogue (no `tenant_id`/
//! RLS — see the migration's own comment), so `PostgresPtnProgramRepository` stays a plain
//! pool-query repository, no `begin_tx`/GUC ceremony. Their domain structs (`PtnProgram`)
//! still expose `id: Uuid` though, so SELECTs read `public_id AS id`-equivalent columns.
//!
//! Every other table here (`student_rapor_scores`, `student_ptn_targets`,
//! `student_achievements`, `student_snbt_tracking`, `snbp_participation`,
//! `school_alumni_benchmarks`, `alumni_rapor_scores`, `school_snbp_eligibility`,
//! `audit_log`) carries `tenant_id`/RLS, so each method opens its own `begin_tx` + GUC
//! transaction (same pattern as `postgres_user_repository::PostgresUserRepository::begin_tx`).
//! `student_id`/`actor_id` columns are bigint FK to `core.akun` (resolved via
//! `core.akun.public_id`); `school_id` columns are bigint FK to `core.tenant` (resolved via
//! `core.tenant.public_id`).

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
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
use crate::domain::tenant::TenantCtx;
use crate::error::{AppError, AppResult};

async fn begin_tx(pool: &PgPool, ctx: &TenantCtx) -> AppResult<Transaction<'static, Postgres>> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)")
        .bind(ctx.tenant_id.to_string())
        .bind(if ctx.is_platform_admin { "true" } else { "false" })
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

// ─── PTN Program catalog (shared reference catalogue — no tenant_id/RLS) ───────

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
    public_id: Uuid,
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
            id: r.public_id,
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
    SELECT public_id, kode, nama_ptn, nama_prodi, provinsi, kota, singkatan, rumpun, mapel_syarat,
           daya_tampung_snbp, peminat_snbp, daya_tampung_snbt, peminat_snbt, pg_snbt, pg_snbp,
           jenjang, has_official_stats, created_at, updated_at
    FROM ptn.ptn_programs
"#;

#[async_trait]
impl PtnProgramRepository for PostgresPtnProgramRepository {
    async fn upsert_by_kode(&self, n: NewPtnProgram) -> AppResult<PtnProgram> {
        let row = sqlx::query_as::<_, PtnProgramRow>(&format!(
            r#"INSERT INTO ptn.ptn_programs
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
               RETURNING public_id, kode, nama_ptn, nama_prodi, provinsi, kota, singkatan, rumpun, mapel_syarat,
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
        let row = sqlx::query_as::<_, PtnProgramRow>(&format!("{SELECT_PTN_PROGRAM} WHERE public_id = $1"))
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
        // atau "Teknik Informatika ITB") tetap ketemu. Tiap kata WAJIB muncul di nama_ptn,
        // nama_prodi, ATAU singkatan (AND antar-kata, OR antar-kolom per kata).
        let words: Vec<String> = filter
            .search
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .split_whitespace()
            .map(|w| format!("%{w}%"))
            .collect();

        let mut qb = QueryBuilder::new(format!("{SELECT_PTN_PROGRAM} WHERE 1=1"));
        let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM ptn.ptn_programs WHERE 1=1");
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
            r#"UPDATE ptn.ptn_programs SET
                nama_ptn = $2, nama_prodi = $3, provinsi = $4, kota = $5, singkatan = $6,
                rumpun = $7, mapel_syarat = $8, daya_tampung_snbp = $9, peminat_snbp = $10,
                daya_tampung_snbt = $11, peminat_snbt = $12, pg_snbt = $13, pg_snbp = $14,
                jenjang = $15, has_official_stats = $16, updated_at = now()
               WHERE public_id = $1"#,
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
        let result = sqlx::query("DELETE FROM ptn.ptn_programs WHERE public_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::from_sqlx_delete(e, "program masih menjadi target siswa aktif"))?;
        Ok(result.rows_affected() > 0)
    }

    async fn count_all(&self) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ptn.ptn_programs")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    async fn distinct_rumpun(&self) -> AppResult<Vec<String>> {
        let rows: Vec<String> =
            sqlx::query_scalar("SELECT DISTINCT rumpun FROM ptn.ptn_programs ORDER BY rumpun ASC")
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
    public_id: Uuid,
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
            id: r.public_id,
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

const SELECT_RAPOR_SCORE: &str = r#"
    SELECT rs.public_id, a.public_id AS student_id, rs.semester, rs.subject, rs.score, rs.is_minat,
           rs.created_at, rs.updated_at
    FROM ptn.student_rapor_scores rs
    JOIN core.akun a ON a.id = rs.student_id
"#;

#[async_trait]
impl RaporScoreRepository for PostgresRaporScoreRepository {
    async fn upsert(&self, student_id: Uuid, entry: RaporScoreUpsert, ctx: &TenantCtx) -> AppResult<RaporScore> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let student_pk: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(student_id)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.student_rapor_scores (student_id, semester, subject, score, is_minat)
               VALUES ($1,$2,$3,$4,$5)
               ON CONFLICT (student_id, semester, subject) DO UPDATE SET
                 score = EXCLUDED.score, is_minat = EXCLUDED.is_minat, updated_at = now()
               RETURNING public_id"#,
        )
        .bind(student_pk)
        .bind(entry.semester)
        .bind(&entry.subject)
        .bind(entry.score)
        .bind(entry.is_minat)
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, RaporScoreRow>(&format!("{SELECT_RAPOR_SCORE} WHERE rs.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(row.into())
    }

    async fn list_by_student(&self, student_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<RaporScore>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, RaporScoreRow>(&format!(
            "{SELECT_RAPOR_SCORE} WHERE a.public_id = $1 ORDER BY rs.semester ASC, rs.subject ASC"
        ))
        .bind(student_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(RaporScore::from).collect())
    }

    async fn delete(&self, student_id: Uuid, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query(
            r#"DELETE FROM ptn.student_rapor_scores
               WHERE public_id = $1 AND student_id = (SELECT id FROM core.akun WHERE public_id = $2)"#,
        )
        .bind(id)
        .bind(student_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
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
    public_id: Uuid,
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
            id: r.public_id,
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

const SELECT_PTN_TARGET: &str = r#"
    SELECT t.public_id, a.public_id AS student_id, p.public_id AS ptn_program_id, t.track, t.priority,
           t.sort_order, t.created_at
    FROM ptn.student_ptn_targets t
    JOIN core.akun a ON a.id = t.student_id
    JOIN ptn.ptn_programs p ON p.id = t.ptn_program_id
"#;

#[async_trait]
impl PtnTargetRepository for PostgresPtnTargetRepository {
    async fn create(&self, n: NewPtnTarget, ctx: &TenantCtx) -> AppResult<PtnTarget> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let student_pk: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(n.student_id)
            .fetch_one(&mut *tx)
            .await?;
        let program_pk: i64 = sqlx::query_scalar("SELECT id FROM ptn.ptn_programs WHERE public_id = $1")
            .bind(n.ptn_program_id)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.student_ptn_targets (student_id, ptn_program_id, track, priority, sort_order)
               VALUES ($1,$2,$3,$4,$5)
               ON CONFLICT (student_id, ptn_program_id, track) DO UPDATE SET
                 priority = EXCLUDED.priority, sort_order = EXCLUDED.sort_order
               RETURNING public_id"#,
        )
        .bind(student_pk)
        .bind(program_pk)
        .bind(n.track.as_str())
        .bind(n.priority.as_str())
        .bind(n.sort_order)
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, PtnTargetRow>(&format!("{SELECT_PTN_TARGET} WHERE t.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        PtnTarget::try_from(row)
    }

    async fn list_by_student(&self, student_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<PtnTarget>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, PtnTargetRow>(&format!(
            "{SELECT_PTN_TARGET} WHERE a.public_id = $1 ORDER BY t.sort_order ASC, t.created_at ASC"
        ))
        .bind(student_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows.into_iter().map(PtnTarget::try_from).collect()
    }

    async fn set_priority(&self, student_id: Uuid, id: Uuid, priority: Priority, ctx: &TenantCtx) -> AppResult<Option<PtnTarget>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query(
            r#"UPDATE ptn.student_ptn_targets SET priority = $3
               WHERE public_id = $1 AND student_id = (SELECT id FROM core.akun WHERE public_id = $2)"#,
        )
        .bind(id)
        .bind(student_id)
        .bind(priority.as_str())
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, PtnTargetRow>(&format!("{SELECT_PTN_TARGET} WHERE t.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        PtnTarget::try_from(row).map(Some)
    }

    async fn delete(&self, student_id: Uuid, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query(
            r#"DELETE FROM ptn.student_ptn_targets
               WHERE public_id = $1 AND student_id = (SELECT id FROM core.akun WHERE public_id = $2)"#,
        )
        .bind(id)
        .bind(student_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
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
    public_id: Uuid,
    school_id: Uuid,
    alumni_name: String,
    graduation_year: i32,
    track: String,
    nama_ptn: String,
    nama_prodi: String,
    benchmark_score: f64,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
}

impl TryFrom<AlumniBenchmarkRow> for AlumniBenchmark {
    type Error = AppError;
    fn try_from(r: AlumniBenchmarkRow) -> Result<Self, Self::Error> {
        Ok(AlumniBenchmark {
            id: r.public_id,
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
            created_by: r.created_by.unwrap_or_else(Uuid::nil),
            created_at: r.created_at,
        })
    }
}

const SELECT_ALUMNI_BENCHMARK: &str = r#"
    SELECT ab.public_id, t.public_id AS school_id, ab.alumni_name, ab.graduation_year, ab.track,
           ab.nama_ptn, ab.nama_prodi, ab.benchmark_score,
           (SELECT public_id FROM core.akun WHERE core.akun.id = ab.created_by) AS created_by,
           ab.created_at
    FROM ptn.school_alumni_benchmarks ab
    JOIN core.tenant t ON t.id = ab.school_id
"#;

#[async_trait]
impl AlumniBenchmarkRepository for PostgresAlumniBenchmarkRepository {
    async fn create(&self, n: NewAlumniBenchmark, ctx: &TenantCtx) -> AppResult<AlumniBenchmark> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let school_pk: i64 = sqlx::query_scalar("SELECT id FROM core.tenant WHERE public_id = $1")
            .bind(n.school_id)
            .fetch_one(&mut *tx)
            .await?;
        let created_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(n.created_by)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.school_alumni_benchmarks
                (school_id, alumni_name, graduation_year, track, nama_ptn, nama_prodi, benchmark_score, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
               RETURNING public_id"#,
        )
        .bind(school_pk)
        .bind(&n.alumni_name)
        .bind(n.graduation_year)
        .bind(n.track.as_str())
        .bind(&n.nama_ptn)
        .bind(&n.nama_prodi)
        .bind(n.benchmark_score)
        .bind(created_by_id)
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, AlumniBenchmarkRow>(&format!("{SELECT_ALUMNI_BENCHMARK} WHERE ab.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        AlumniBenchmark::try_from(row)
    }

    async fn list_by_school(&self, school_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<AlumniBenchmark>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, AlumniBenchmarkRow>(&format!(
            "{SELECT_ALUMNI_BENCHMARK} WHERE t.public_id = $1 ORDER BY ab.graduation_year DESC, ab.alumni_name ASC"
        ))
        .bind(school_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows.into_iter().map(AlumniBenchmark::try_from).collect()
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<AlumniBenchmark>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let row = sqlx::query_as::<_, AlumniBenchmarkRow>(&format!("{SELECT_ALUMNI_BENCHMARK} WHERE ab.public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(AlumniBenchmark::try_from).transpose()
    }

    async fn delete(&self, school_id: Uuid, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query(
            r#"DELETE FROM ptn.school_alumni_benchmarks
               WHERE public_id = $1 AND school_id = (SELECT id FROM core.tenant WHERE public_id = $2)"#,
        )
        .bind(id)
        .bind(school_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn upsert_rapor(&self, alumni_id: Uuid, entry: AlumniRaporScoreUpsert, ctx: &TenantCtx) -> AppResult<AlumniRaporScore> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let alumni_pk: i64 = sqlx::query_scalar("SELECT id FROM ptn.school_alumni_benchmarks WHERE public_id = $1")
            .bind(alumni_id)
            .fetch_one(&mut *tx)
            .await?;
        let row = sqlx::query_as::<_, AlumniRaporScoreRow>(
            r#"INSERT INTO ptn.alumni_rapor_scores (alumni_id, subject, score)
               VALUES ($1, $2, $3)
               ON CONFLICT (alumni_id, subject) DO UPDATE SET score = EXCLUDED.score, updated_at = now()
               RETURNING public_id, alumni_id, subject, score, created_at, updated_at"#,
        )
        .bind(alumni_pk)
        .bind(&entry.subject)
        .bind(entry.score)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(AlumniRaporScore { id: row.public_id, alumni_id, subject: row.subject, score: row.score, created_at: row.created_at, updated_at: row.updated_at })
    }

    async fn list_rapor(&self, alumni_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<AlumniRaporScore>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let alumni_pk: i64 = sqlx::query_scalar("SELECT id FROM ptn.school_alumni_benchmarks WHERE public_id = $1")
            .bind(alumni_id)
            .fetch_one(&mut *tx)
            .await?;
        let rows = sqlx::query_as::<_, AlumniRaporScoreRow>(
            "SELECT public_id, alumni_id, subject, score, created_at, updated_at
             FROM ptn.alumni_rapor_scores WHERE alumni_id = $1 ORDER BY subject ASC",
        )
        .bind(alumni_pk)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(|r| AlumniRaporScore {
            id: r.public_id,
            alumni_id,
            subject: r.subject,
            score: r.score,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect())
    }

    async fn delete_rapor(&self, alumni_id: Uuid, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query(
            r#"DELETE FROM ptn.alumni_rapor_scores
               WHERE public_id = $1 AND alumni_id = (SELECT id FROM ptn.school_alumni_benchmarks WHERE public_id = $2)"#,
        )
        .bind(id)
        .bind(alumni_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }
}

/// Raw row shape shared by `upsert_rapor`/`list_rapor` — note `alumni_id` here is the
/// bigint internal FK, remapped to the `Uuid` alumni public_id by the caller (which already
/// has it in scope), not re-resolved from the DB on every row.
#[derive(sqlx::FromRow)]
struct AlumniRaporScoreRow {
    public_id: Uuid,
    #[allow(dead_code)]
    alumni_id: i64,
    subject: String,
    score: f64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
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
    public_id: Uuid,
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
            id: r.public_id,
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

const SELECT_SNBT_TRACKING: &str = r#"
    SELECT st.public_id, a.public_id AS student_id, st.actual_score, st.exam_date, st.status, st.notes, st.updated_at
    FROM ptn.student_snbt_tracking st
    JOIN core.akun a ON a.id = st.student_id
"#;

#[async_trait]
impl SnbtTrackingRepository for PostgresSnbtTrackingRepository {
    async fn upsert(&self, student_id: Uuid, entry: SnbtTrackingUpsert, ctx: &TenantCtx) -> AppResult<SnbtTracking> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let student_pk: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(student_id)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.student_snbt_tracking (student_id, actual_score, exam_date, status, notes)
               VALUES ($1,$2,$3,$4,$5)
               ON CONFLICT (student_id) DO UPDATE SET
                 actual_score = EXCLUDED.actual_score, exam_date = EXCLUDED.exam_date,
                 status = EXCLUDED.status, notes = EXCLUDED.notes, updated_at = now()
               RETURNING public_id"#,
        )
        .bind(student_pk)
        .bind(entry.actual_score)
        .bind(entry.exam_date)
        .bind(entry.status.as_str())
        .bind(&entry.notes)
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, SnbtTrackingRow>(&format!("{SELECT_SNBT_TRACKING} WHERE st.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        SnbtTracking::try_from(row)
    }

    async fn get_by_student(&self, student_id: Uuid, ctx: &TenantCtx) -> AppResult<Option<SnbtTracking>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let row = sqlx::query_as::<_, SnbtTrackingRow>(&format!("{SELECT_SNBT_TRACKING} WHERE a.public_id = $1"))
            .bind(student_id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
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
    public_id: Uuid,
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
            id: r.public_id,
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

const SELECT_SNBP_PARTICIPATION: &str = r#"
    SELECT sp.public_id, a.public_id AS student_id, t.public_id AS school_id, sp.year, sp.konsultan,
           sp.status, sp.aktif, sp.updated_at
    FROM ptn.snbp_participation sp
    JOIN core.akun a ON a.id = sp.student_id
    JOIN core.tenant t ON t.id = sp.school_id
"#;

#[async_trait]
impl SnbpParticipationRepository for PostgresSnbpParticipationRepository {
    async fn upsert(&self, student_id: Uuid, entry: SnbpParticipationUpsert, ctx: &TenantCtx) -> AppResult<SnbpParticipation> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let student_pk: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(student_id)
            .fetch_one(&mut *tx)
            .await?;
        let school_pk: i64 = sqlx::query_scalar("SELECT id FROM core.tenant WHERE public_id = $1")
            .bind(entry.school_id)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.snbp_participation (student_id, school_id, year, konsultan, status, aktif)
               VALUES ($1,$2,$3,$4,$5,$6)
               ON CONFLICT (student_id) DO UPDATE SET
                 year = EXCLUDED.year, konsultan = EXCLUDED.konsultan,
                 status = EXCLUDED.status, aktif = EXCLUDED.aktif, updated_at = now()
               RETURNING public_id"#,
        )
        .bind(student_pk)
        .bind(school_pk)
        .bind(entry.year)
        .bind(&entry.konsultan)
        .bind(entry.status.as_str())
        .bind(entry.aktif)
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, SnbpParticipationRow>(&format!("{SELECT_SNBP_PARTICIPATION} WHERE sp.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        SnbpParticipation::try_from(row)
    }

    async fn get_by_student(&self, student_id: Uuid, ctx: &TenantCtx) -> AppResult<Option<SnbpParticipation>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let row = sqlx::query_as::<_, SnbpParticipationRow>(&format!("{SELECT_SNBP_PARTICIPATION} WHERE a.public_id = $1"))
            .bind(student_id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(SnbpParticipation::try_from).transpose()
    }

    async fn list_by_school(&self, school_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<SnbpParticipation>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, SnbpParticipationRow>(&format!("{SELECT_SNBP_PARTICIPATION} WHERE t.public_id = $1"))
            .bind(school_id)
            .fetch_all(&mut *tx)
            .await?;
        tx.commit().await?;
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
    public_id: Uuid,
    school_id: Uuid,
    year: i32,
    eligible_count: i32,
    updated_at: DateTime<Utc>,
}

impl From<SchoolEligibilityRow> for SchoolEligibility {
    fn from(r: SchoolEligibilityRow) -> Self {
        SchoolEligibility { id: r.public_id, school_id: r.school_id, year: r.year, eligible_count: r.eligible_count, updated_at: r.updated_at }
    }
}

const SELECT_SCHOOL_ELIGIBILITY: &str = r#"
    SELECT se.public_id, t.public_id AS school_id, se.year, se.eligible_count, se.updated_at
    FROM ptn.school_snbp_eligibility se
    JOIN core.tenant t ON t.id = se.school_id
"#;

#[async_trait]
impl SchoolEligibilityRepository for PostgresSchoolEligibilityRepository {
    async fn upsert(&self, school_id: Uuid, entry: SchoolEligibilityUpsert, ctx: &TenantCtx) -> AppResult<SchoolEligibility> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let school_pk: i64 = sqlx::query_scalar("SELECT id FROM core.tenant WHERE public_id = $1")
            .bind(school_id)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.school_snbp_eligibility (school_id, year, eligible_count)
               VALUES ($1,$2,$3)
               ON CONFLICT (school_id, year) DO UPDATE SET
                 eligible_count = EXCLUDED.eligible_count, updated_at = now()
               RETURNING public_id"#,
        )
        .bind(school_pk)
        .bind(entry.year)
        .bind(entry.eligible_count)
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, SchoolEligibilityRow>(&format!("{SELECT_SCHOOL_ELIGIBILITY} WHERE se.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(row.into())
    }

    async fn list_by_school(&self, school_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<SchoolEligibility>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, SchoolEligibilityRow>(&format!(
            "{SELECT_SCHOOL_ELIGIBILITY} WHERE t.public_id = $1 ORDER BY se.year DESC"
        ))
        .bind(school_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
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
    public_id: Uuid,
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
            id: r.public_id,
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

const SELECT_ACHIEVEMENT: &str = r#"
    SELECT ach.public_id, a.public_id AS student_id, ach.nama, ach.tingkat, ach.tahun, ach.juara,
           ach.certificate_url, ach.created_at
    FROM ptn.student_achievements ach
    JOIN core.akun a ON a.id = ach.student_id
"#;

#[async_trait]
impl AchievementRepository for PostgresAchievementRepository {
    async fn create(&self, n: NewAchievement, ctx: &TenantCtx) -> AppResult<Achievement> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let student_pk: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(n.student_id)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.student_achievements (student_id, nama, tingkat, tahun, juara)
               VALUES ($1,$2,$3,$4,$5)
               RETURNING public_id"#,
        )
        .bind(student_pk)
        .bind(&n.nama)
        .bind(n.tingkat.as_str())
        .bind(n.tahun)
        .bind(&n.juara)
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, AchievementRow>(&format!("{SELECT_ACHIEVEMENT} WHERE ach.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Achievement::try_from(row)
    }

    async fn list_by_student(&self, student_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<Achievement>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, AchievementRow>(&format!(
            "{SELECT_ACHIEVEMENT} WHERE a.public_id = $1 ORDER BY ach.tahun DESC, ach.created_at DESC"
        ))
        .bind(student_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows.into_iter().map(Achievement::try_from).collect()
    }

    async fn delete(&self, student_id: Uuid, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query(
            r#"DELETE FROM ptn.student_achievements
               WHERE public_id = $1 AND student_id = (SELECT id FROM core.akun WHERE public_id = $2)"#,
        )
        .bind(id)
        .bind(student_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_certificate(&self, student_id: Uuid, id: Uuid, certificate_url: Option<String>, ctx: &TenantCtx) -> AppResult<Option<Achievement>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query(
            r#"UPDATE ptn.student_achievements SET certificate_url = $3
               WHERE public_id = $1 AND student_id = (SELECT id FROM core.akun WHERE public_id = $2)"#,
        )
        .bind(id)
        .bind(student_id)
        .bind(&certificate_url)
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, AchievementRow>(&format!("{SELECT_ACHIEVEMENT} WHERE ach.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Achievement::try_from(row).map(Some)
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
    public_id: Uuid,
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
            id: r.public_id,
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

const SELECT_AUDIT_LOG: &str = r#"
    SELECT al.public_id, actor.public_id AS actor_id, al.actor_name, al.actor_role,
           student.public_id AS student_id, al.action, al.entity_type, al.summary, al.created_at
    FROM ptn.audit_log al
    JOIN core.akun actor ON actor.id = al.actor_id
    JOIN core.akun student ON student.id = al.student_id
"#;

#[async_trait]
impl AuditLogRepository for PostgresAuditLogRepository {
    async fn record(&self, entry: NewAuditLogEntry, ctx: &TenantCtx) -> AppResult<AuditLogEntry> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let actor_pk: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(entry.actor_id)
            .fetch_one(&mut *tx)
            .await?;
        let student_pk: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(entry.student_id)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.audit_log (actor_id, actor_name, actor_role, student_id, action, entity_type, summary)
               VALUES ($1,$2,$3,$4,$5,$6,$7)
               RETURNING public_id"#,
        )
        .bind(actor_pk)
        .bind(&entry.actor_name)
        .bind(&entry.actor_role)
        .bind(student_pk)
        .bind(&entry.action)
        .bind(&entry.entity_type)
        .bind(&entry.summary)
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, AuditLogRow>(&format!("{SELECT_AUDIT_LOG} WHERE al.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(row.into())
    }

    async fn list_by_student(&self, student_id: Uuid, limit: i64, ctx: &TenantCtx) -> AppResult<Vec<AuditLogEntry>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, AuditLogRow>(&format!(
            "{SELECT_AUDIT_LOG} WHERE student.public_id = $1 ORDER BY al.created_at DESC LIMIT $2"
        ))
        .bind(student_id)
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(AuditLogEntry::from).collect())
    }
}
