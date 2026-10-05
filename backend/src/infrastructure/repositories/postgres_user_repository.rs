use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::repository::{NewUser, UserFilter, UserRepository, UserUpdate};
use crate::domain::user::{Role, User, UserStatus};
use crate::error::{AppError, AppResult};

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    name: String,
    email: String,
    password_hash: String,
    role: String,
    school_id: Option<Uuid>,
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
        let role = Role::from_str(&row.role)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt role in db: {e}")))?;
        let status = UserStatus::from_str(&row.status)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt user status in db: {e}")))?;
        Ok(User {
            id: row.id,
            name: row.name,
            email: row.email,
            password_hash: row.password_hash,
            role,
            school_id: row.school_id,
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

// `st` is a LEFT JOIN because nisn/grade only exist for role = 'student' (via the 1:1
// `students` profile row) — every other role reads them back as NULL.
const SELECT_USER: &str = r#"
    SELECT u.id, u.name, u.email, u.password_hash, u.role, u.school_id,
           s.name AS school_name, u.status, u.last_login, u.phone,
           st.nisn, st.grade, st.nis, st.gender, st.birth_date, st.enrolled_at,
           st.rombel_code, st.username, u.avatar_url, u.minat_jurusan, u.created_at
    FROM users u
    LEFT JOIN schools s ON s.id = u.school_id
    LEFT JOIN students st ON st.user_id = u.id
"#;

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, new_user: NewUser) -> AppResult<User> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO users (id, name, email, password_hash, role, school_id, status, phone)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(id)
        .bind(&new_user.name)
        .bind(&new_user.email)
        .bind(&new_user.password_hash)
        .bind(new_user.role.as_str())
        .bind(new_user.school_id)
        .bind(new_user.status.as_str())
        .bind(&new_user.phone)
        .execute(&self.pool)
        .await?;

        // If this is a student, also create the student profile row (nisn/grade/etc. live
        // here). nis/gender/birth_date/enrolled_at/rombel_code/username are create-only (no
        // `UserUpdate` field for them — see domain::repository::UserUpdate doc comment) so
        // the general admin "Edit User" flow, which never sends these, can't silently null
        // them back out on an unrelated edit.
        if matches!(new_user.role, Role::Student) {
            sqlx::query(
                r#"INSERT INTO students (user_id, school_id, nisn, grade, nis, gender, birth_date, enrolled_at, rombel_code, username)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
            )
            .bind(id)
            .bind(new_user.school_id)
            .bind(&new_user.nisn)
            .bind(&new_user.grade)
            .bind(&new_user.nis)
            .bind(&new_user.gender)
            .bind(new_user.birth_date)
            .bind(new_user.enrolled_at)
            .bind(&new_user.rombel_code)
            .bind(&new_user.username)
            .execute(&self.pool)
            .await?;
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("user vanished right after insert")))
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(&format!("{SELECT_USER} WHERE u.id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(User::try_from).transpose()
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(&format!("{SELECT_USER} WHERE u.email = $1"))
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        row.map(User::try_from).transpose()
    }

    async fn list(&self, filter: UserFilter) -> AppResult<Vec<User>> {
        let mut sql = SELECT_USER.to_string();
        let mut conditions: Vec<String> = Vec::new();
        let mut idx = 1;
        if filter.role.is_some() {
            conditions.push(format!("u.role = ${idx}"));
            idx += 1;
        }
        if filter.status.is_some() {
            conditions.push(format!("u.status = ${idx}"));
            idx += 1;
        }
        if filter.search.is_some() {
            conditions.push(format!("(u.name ILIKE ${idx} OR u.email ILIKE ${idx})"));
            idx += 1;
        }
        if filter.school_id.is_some() {
            conditions.push(format!("u.school_id = ${idx}"));
        }
        if let Some(has_school) = filter.has_school {
            conditions.push(if has_school { "u.school_id IS NOT NULL".to_string() } else { "u.school_id IS NULL".to_string() });
        }
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY u.created_at DESC");

        let mut query = sqlx::query_as::<_, UserRow>(&sql);
        if let Some(role) = filter.role {
            query = query.bind(role.as_str());
        }
        if let Some(status) = filter.status {
            query = query.bind(status.as_str());
        }
        if let Some(search) = &filter.search {
            query = query.bind(format!("%{search}%"));
        }
        if let Some(school_id) = filter.school_id {
            query = query.bind(school_id);
        }
        let rows = query.fetch_all(&self.pool).await?;
        rows.into_iter().map(User::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: UserUpdate) -> AppResult<Option<User>> {
        let mut tx = self.pool.begin().await?;

        let updated = if let Some(password_hash) = &u.password_hash {
            sqlx::query(
                r#"UPDATE users SET name = $2, email = $3, role = $4, school_id = $5, password_hash = $6,
                       status = $7, phone = $8, minat_jurusan = $9
                   WHERE id = $1"#,
            )
            .bind(id)
            .bind(&u.name)
            .bind(&u.email)
            .bind(u.role.as_str())
            .bind(u.school_id)
            .bind(password_hash)
            .bind(u.status.as_str())
            .bind(&u.phone)
            .bind(&u.minat_jurusan)
            .execute(&mut *tx)
            .await?
        } else {
            sqlx::query(
                r#"UPDATE users SET name = $2, email = $3, role = $4, school_id = $5, status = $6, phone = $7,
                       minat_jurusan = $8
                   WHERE id = $1"#,
            )
            .bind(id)
            .bind(&u.name)
            .bind(&u.email)
            .bind(u.role.as_str())
            .bind(u.school_id)
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

        // Keep the `students` 1:1 profile row (including nisn/grade) in sync with the role.
        if matches!(u.role, Role::Student) {
            sqlx::query(
                r#"INSERT INTO students (user_id, school_id, nisn, grade) VALUES ($1, $2, $3, $4)
                   ON CONFLICT (user_id) DO UPDATE SET school_id = EXCLUDED.school_id, nisn = EXCLUDED.nisn, grade = EXCLUDED.grade"#,
            )
            .bind(id)
            .bind(u.school_id)
            .bind(&u.nisn)
            .bind(&u.grade)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query("DELETE FROM students WHERE user_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        self.find_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::from_sqlx_delete(
                    e,
                    "cannot delete this user: they have created questions or tryout sessions — reassign or delete that content first",
                )
            })?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_status(&self, id: Uuid, status: UserStatus) -> AppResult<Option<User>> {
        let result = sqlx::query("UPDATE users SET status = $2 WHERE id = $1")
            .bind(id)
            .bind(status.as_str())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn touch_last_login(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE users SET last_login = now() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_avatar(&self, id: Uuid, avatar_url: Option<String>) -> AppResult<Option<User>> {
        let result = sqlx::query("UPDATE users SET avatar_url = $2 WHERE id = $1")
            .bind(id)
            .bind(&avatar_url)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }
}
