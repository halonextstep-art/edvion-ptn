use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::repository::{NewUser, UserFilter, UserRepository, UserUpdate};
use crate::domain::tenant::TenantCtx;
use crate::domain::user::{Role, User, UserStatus};
use crate::error::{AppError, AppResult};

pub struct PostgresUserRepository {
    pool: PgPool,
    /// Connected as the BYPASSRLS `ptn_auth_lookup` role — used only by `find_by_email`,
    /// the one method that must run before a `TenantCtx` is known. See
    /// `application::auth_service` module doc comment for why this exists.
    lookup_pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool, lookup_pool: PgPool) -> Self {
        Self { pool, lookup_pool }
    }

    /// Opens this call's transaction and sets the RLS GUCs on it — every method below
    /// (except `find_by_email`, the pre-auth lookup) opens one of these first. `true` =
    /// transaction-local (`SET LOCAL` semantics), so it can never leak to a later request
    /// that reuses this pooled connection. See `domain::tenant::TenantCtx` doc comment.
    async fn begin_tx(&self, ctx: &TenantCtx) -> AppResult<Transaction<'static, Postgres>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)",
        )
        .bind(ctx.tenant_id.to_string())
        .bind(if ctx.is_platform_admin { "true" } else { "false" })
        .execute(&mut *tx)
        .await?;
        Ok(tx)
    }

    /// Shared `WHERE`-clause builder for `list`/`list_page` — kept as one place so the two
    /// can never drift out of sync with each other (e.g. `list_page`'s `total` silently
    /// counting a different set of rows than the page it returns).
    fn build_list_query(filter: &UserFilter) -> QueryBuilder<'static, Postgres> {
        let mut qb = QueryBuilder::new(SELECT_USER);
        Self::push_list_conditions(&mut qb, filter);
        qb
    }

    fn build_list_count_query(filter: &UserFilter) -> QueryBuilder<'static, Postgres> {
        let mut qb = QueryBuilder::new(
            "SELECT COUNT(*) FROM core.akun a
             JOIN core.role r ON r.id = a.role_id
             LEFT JOIN core.tenant t ON t.id = a.tenant_id AND a.tenant_id <> 0",
        );
        Self::push_list_conditions(&mut qb, filter);
        qb
    }

    fn push_list_conditions(qb: &mut QueryBuilder<'static, Postgres>, filter: &UserFilter) {
        // `core.akun`/`core.role` are shared with academic/tahfiz/office/etc. — without
        // this, a plain unfiltered listing (e.g. admin's "all users" view) would try to
        // decode every account in the whole table, including roles PTN has never heard of
        // (GURU, BK, WAKA_*, ...), and `Role::from_core_role_kode` correctly rejects those
        // as a 500 rather than silently mis-mapping them. PTN must only ever see its own
        // four roles here, regardless of what the caller's filter asks for.
        qb.push(" WHERE r.kode IN ('ADMIN','SEKOLAH','SISWA','KONTEN')");
        if let Some(role) = filter.role {
            qb.push(" AND r.kode = ").push_bind(role.core_role_kode());
        }
        if let Some(status) = filter.status {
            qb.push(" AND a.status = ").push_bind(status.as_str());
        }
        if let Some(search) = &filter.search {
            let pattern = format!("%{search}%");
            qb.push(" AND (a.full_name ILIKE ").push_bind(pattern.clone())
                .push(" OR a.email ILIKE ").push_bind(pattern).push(")");
        }
        if let Some(school_id) = filter.school_id {
            qb.push(" AND t.public_id = ").push_bind(school_id);
        }
        if let Some(has_school) = filter.has_school {
            qb.push(if has_school { " AND a.tenant_id <> 0" } else { " AND a.tenant_id = 0" });
        }
    }
}

#[derive(sqlx::FromRow)]
struct LookupRow {
    public_id: Uuid,
    tenant_id: i64,
    role_kode: String,
}

#[derive(sqlx::FromRow)]
struct UserRow {
    public_id: Uuid,
    name: Option<String>,
    email: Option<String>,
    password_hash: String,
    role_kode: String,
    school_public_id: Option<Uuid>,
    school_name: Option<String>,
    status: String,
    last_login: Option<DateTime<Utc>>,
    phone: Option<String>,
    nisn: Option<String>,
    grade: Option<String>,
    nis: Option<String>,
    gender: Option<String>,
    birth_date: Option<NaiveDate>,
    enrolled_at: Option<NaiveDate>,
    rombel_code: Option<String>,
    username: Option<String>,
    avatar_url: Option<String>,
    minat_jurusan: Option<String>,
    created_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = AppError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let role = Role::from_core_role_kode(&row.role_kode)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt role in db: {e}")))?;
        let status = UserStatus::from_str(&row.status)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt user status in db: {e}")))?;
        Ok(User {
            id: row.public_id,
            name: row.name,
            email: row.email,
            password_hash: row.password_hash,
            role,
            school_id: row.school_public_id,
            school_name: row.school_name,
            status,
            last_login: row.last_login,
            phone: row.phone,
            nisn: row.nisn,
            grade: row.grade,
            nis: row.nis,
            gender: row.gender,
            birth_date: row.birth_date,
            enrolled_at: row.enrolled_at,
            rombel_code: row.rombel_code,
            username: row.username,
            avatar_url: row.avatar_url,
            minat_jurusan: row.minat_jurusan,
            created_at: row.created_at,
        })
    }
}

// `sp` (ptn.student_profile) is a LEFT JOIN because nisn/grade/etc. only exist for
// role = 'student'; every other role reads them back as NULL. `t` (core.tenant) is a LEFT
// JOIN because admin/content accounts live under the platform tenant but still want
// `school_name = NULL` rather than "Platform (System)" showing up as a school.
const SELECT_USER: &str = r#"
    SELECT a.public_id, a.full_name AS name, a.email, a.password_hash, r.kode AS role_kode,
           t.public_id AS school_public_id, t.name AS school_name, a.status, a.last_login_at AS last_login,
           a.telepon AS phone,
           sp.nisn, sp.grade, sp.nis, sp.gender, sp.birth_date, sp.enrolled_at,
           sp.rombel_code, sp.username, a.avatar_url, a.minat_jurusan, a.created_at
    FROM core.akun a
    JOIN core.role r ON r.id = a.role_id
    LEFT JOIN core.tenant t ON t.id = a.tenant_id AND a.tenant_id <> 0
    LEFT JOIN ptn.student_profile sp ON sp.akun_id = a.id
"#;

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, new_user: NewUser) -> AppResult<User> {
        // The right tenant for a NEW account depends on the account's own role/school_id,
        // not on whoever is calling — resolve it first (via the BYPASSRLS lookup pool,
        // since no tenant context exists yet and `core.tenant` carries no RLS policy
        // anyway, so this is just an ordinary read), then open the real RLS-scoped
        // transaction with that tenant.
        let tenant_id = match (new_user.role.is_platform_scoped(), new_user.school_id) {
            (true, _) => TenantCtx::PLATFORM_TENANT_ID,
            (false, Some(school_public_id)) => {
                sqlx::query_scalar::<_, i64>("SELECT id FROM core.tenant WHERE public_id = $1")
                    .bind(school_public_id)
                    .fetch_optional(&self.lookup_pool)
                    .await?
                    .ok_or_else(|| AppError::Validation("school_id does not match any known school".to_string()))?
            }
            // B2C (mandiri) student/no school — still needs *a* tenant row to satisfy
            // `core.akun.tenant_id NOT NULL`/RLS; the platform tenant with
            // `is_platform_admin = false` (plain `TenantCtx::school`, not
            // `TenantCtx::platform_admin()`) gives them no elevated access.
            (false, None) => TenantCtx::PLATFORM_TENANT_ID,
        };
        let ctx = TenantCtx::school(tenant_id);
        let mut tx = self.begin_tx(&ctx).await?;

        let role_id: i64 = sqlx::query_scalar("SELECT id FROM core.role WHERE kode = $1")
            .bind(new_user.role.core_role_kode())
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("role lookup failed: {e}")))?;

        let akun_id: i64 = sqlx::query_scalar(
            r#"INSERT INTO core.akun (tenant_id, username, email, password_hash, full_name, role_id, status, telepon, avatar_url, minat_jurusan)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, NULL)
               RETURNING id"#,
        )
        .bind(ctx.tenant_id)
        .bind(&new_user.email)
        .bind(&new_user.email)
        .bind(&new_user.password_hash)
        .bind(&new_user.name)
        .bind(role_id)
        .bind(new_user.status.as_str())
        .bind(&new_user.phone)
        .fetch_one(&mut *tx)
        .await?;

        // `core.akun` doesn't have a `status` column text-check matching PTN's; it's added
        // generically for every module, but PTN still needs its own lifecycle gate (see
        // `AuthService::login`) — stored as plain text same as before, just on `core.akun`.
        sqlx::query("UPDATE core.akun SET status = $2 WHERE id = $1")
            .bind(akun_id)
            .bind(new_user.status.as_str())
            .execute(&mut *tx)
            .await?;

        if matches!(new_user.role, Role::Student) {
            sqlx::query(
                r#"INSERT INTO ptn.student_profile (akun_id, tenant_id, nisn, grade, nis, gender, birth_date, enrolled_at, rombel_code, username)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
            )
            .bind(akun_id)
            .bind(ctx.tenant_id)
            .bind(&new_user.nisn)
            .bind(&new_user.grade)
            .bind(&new_user.nis)
            .bind(&new_user.gender)
            .bind(new_user.birth_date)
            .bind(new_user.enrolled_at)
            .bind(&new_user.rombel_code)
            .bind(&new_user.username)
            .execute(&mut *tx)
            .await?;
        }

        let row = sqlx::query_as::<_, UserRow>(&format!("{SELECT_USER} WHERE a.id = $1"))
            .bind(akun_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        User::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<User>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, UserRow>(&format!("{SELECT_USER} WHERE a.public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(User::try_from).transpose()
    }

    /// Pre-auth/uniqueness-check path — no `TenantCtx` exists yet, this is what resolves
    /// one. Runs the first lookup through the BYPASSRLS `lookup_pool` (we don't know the
    /// email's tenant, that's exactly what's being discovered), then re-fetches the full
    /// row through the normal RLS-scoped path once the tenant/role is known.
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, LookupRow>(
            r#"SELECT a.public_id, a.tenant_id, r.kode AS role_kode
               FROM core.akun a
               JOIN core.role r ON r.id = a.role_id
               WHERE a.email = $1"#,
        )
        .bind(email)
        .fetch_optional(&self.lookup_pool)
        .await?;
        let Some(row) = row else { return Ok(None) };
        let role = Role::from_core_role_kode(&row.role_kode)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt role in db: {e}")))?;
        let ctx = TenantCtx::for_role(role, row.tenant_id);
        self.find_by_id(row.public_id, &ctx).await
    }

    async fn list(&self, filter: UserFilter, ctx: &TenantCtx) -> AppResult<Vec<User>> {
        let mut tx = self.begin_tx(ctx).await?;
        let mut qb = Self::build_list_query(&filter);
        qb.push(" ORDER BY a.created_at DESC");
        let rows = qb.build_query_as::<UserRow>().fetch_all(&mut *tx).await?;
        tx.commit().await?;
        rows.into_iter().map(User::try_from).collect()
    }

    async fn list_page(&self, filter: UserFilter, ctx: &TenantCtx) -> AppResult<(Vec<User>, i64)> {
        let mut tx = self.begin_tx(ctx).await?;
        let page = filter.page.max(1);
        let page_size = filter.page_size.clamp(1, 100);
        let offset = (page - 1) * page_size;

        let mut count_qb = Self::build_list_count_query(&filter);
        let mut qb = Self::build_list_query(&filter);
        qb.push(" ORDER BY a.created_at DESC LIMIT ").push_bind(page_size).push(" OFFSET ").push_bind(offset);

        let rows = qb.build_query_as::<UserRow>().fetch_all(&mut *tx).await?;
        let total: i64 = count_qb.build_query_scalar().fetch_one(&mut *tx).await?;
        tx.commit().await?;
        let users = rows.into_iter().map(User::try_from).collect::<AppResult<Vec<_>>>()?;
        Ok((users, total))
    }

    async fn update(&self, id: Uuid, u: UserUpdate, ctx: &TenantCtx) -> AppResult<Option<User>> {
        let mut tx = self.begin_tx(ctx).await?;

        let role_id: i64 = sqlx::query_scalar("SELECT id FROM core.role WHERE kode = $1")
            .bind(u.role.core_role_kode())
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("role lookup failed: {e}")))?;

        let akun_id: Option<i64> = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        let Some(akun_id) = akun_id else {
            tx.rollback().await.ok();
            return Ok(None);
        };

        // `username` is deliberately NOT touched here — it has no corresponding field in
        // `UserUpdate` (PTN's own accounts only ever set it once, at creation, and legacy
        // cross-module accounts' `username` is their real login identifier, see
        // `AuthService::find_lookup_row_by_username`). An earlier version of this query
        // bound `username = $3` to the email value, which silently overwrote a legacy
        // account's username with its email on every profile edit/password change —
        // removed, not just redundant.
        let updated = if let Some(password_hash) = &u.password_hash {
            sqlx::query(
                r#"UPDATE core.akun SET full_name = $2, email = $3, role_id = $4, password_hash = $5,
                       status = $6, telepon = $7, minat_jurusan = $8
                   WHERE id = $1"#,
            )
            .bind(akun_id)
            .bind(&u.name)
            .bind(&u.email)
            .bind(role_id)
            .bind(password_hash)
            .bind(u.status.as_str())
            .bind(&u.phone)
            .bind(&u.minat_jurusan)
            .execute(&mut *tx)
            .await?
        } else {
            sqlx::query(
                r#"UPDATE core.akun SET full_name = $2, email = $3, role_id = $4, status = $5, telepon = $6,
                       minat_jurusan = $7
                   WHERE id = $1"#,
            )
            .bind(akun_id)
            .bind(&u.name)
            .bind(&u.email)
            .bind(role_id)
            .bind(u.status.as_str())
            .bind(&u.phone)
            .bind(&u.minat_jurusan)
            .execute(&mut *tx)
            .await?
        };

        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }

        if matches!(u.role, Role::Student) {
            sqlx::query(
                r#"INSERT INTO ptn.student_profile (akun_id, tenant_id, nisn, grade) VALUES ($1, $2, $3, $4)
                   ON CONFLICT (akun_id) DO UPDATE SET nisn = EXCLUDED.nisn, grade = EXCLUDED.grade"#,
            )
            .bind(akun_id)
            .bind(ctx.tenant_id)
            .bind(&u.nisn)
            .bind(&u.grade)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query("DELETE FROM ptn.student_profile WHERE akun_id = $1")
                .bind(akun_id)
                .execute(&mut *tx)
                .await?;
        }

        let row = sqlx::query_as::<_, UserRow>(&format!("{SELECT_USER} WHERE a.id = $1"))
            .bind(akun_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        User::try_from(row).map(Some)
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("DELETE FROM core.akun WHERE public_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                AppError::from_sqlx_delete(
                    e,
                    "cannot delete this user: they have created questions or tryout sessions — reassign or delete that content first",
                )
            })?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_status(&self, id: Uuid, status: UserStatus, ctx: &TenantCtx) -> AppResult<Option<User>> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("UPDATE core.akun SET status = $2 WHERE public_id = $1")
            .bind(id)
            .bind(status.as_str())
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, UserRow>(&format!("{SELECT_USER} WHERE a.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        User::try_from(row).map(Some)
    }

    async fn touch_last_login(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<()> {
        let mut tx = self.begin_tx(ctx).await?;
        sqlx::query("UPDATE core.akun SET last_login_at = now() WHERE public_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn update_avatar(&self, id: Uuid, avatar_url: Option<String>, ctx: &TenantCtx) -> AppResult<Option<User>> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("UPDATE core.akun SET avatar_url = $2 WHERE public_id = $1")
            .bind(id)
            .bind(&avatar_url)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, UserRow>(&format!("{SELECT_USER} WHERE a.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        User::try_from(row).map(Some)
    }
}
