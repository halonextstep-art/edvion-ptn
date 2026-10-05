use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::voucher::{DiscountType, Voucher, VoucherType};
use crate::domain::repository::{NewVoucher, VoucherRepository};
use crate::error::AppResult;

pub struct PostgresVoucherRepository {
    pool: PgPool,
}

impl PostgresVoucherRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct VoucherRow {
    id: Uuid,
    code: String,
    voucher_type: String,
    package_id: Uuid,
    package_name: String,
    discount_type: String,
    discount_value: i32,
    max_uses: i32,
    used_count: i32,
    active: bool,
    expires_at: NaiveDate,
    note: String,
    school_name: Option<String>,
    referrer_name: Option<String>,
    referrer_commission: Option<i32>,
    created_by: Uuid,
    created_at: DateTime<Utc>,
}

impl TryFrom<VoucherRow> for Voucher {
    type Error = crate::error::AppError;
    fn try_from(r: VoucherRow) -> Result<Self, Self::Error> {
        Ok(Voucher {
            id: r.id,
            code: r.code,
            voucher_type: r
                .voucher_type
                .parse::<VoucherType>()
                .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("corrupt voucher_type: {e}")))?,
            package_id: r.package_id,
            package_name: r.package_name,
            discount_type: r
                .discount_type
                .parse::<DiscountType>()
                .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("corrupt discount_type: {e}")))?,
            discount_value: r.discount_value,
            max_uses: r.max_uses,
            used_count: r.used_count,
            active: r.active,
            expires_at: r.expires_at,
            note: r.note,
            school_name: r.school_name,
            referrer_name: r.referrer_name,
            referrer_commission: r.referrer_commission,
            created_by: r.created_by,
            created_at: r.created_at,
        })
    }
}

const SELECT_VOUCHER: &str = r#"
    SELECT v.id, v.code, v.voucher_type, v.package_id, p.name AS package_name,
           v.discount_type, v.discount_value, v.max_uses, v.used_count, v.active,
           v.expires_at, v.note, v.school_name, v.referrer_name, v.referrer_commission,
           v.created_by, v.created_at
    FROM vouchers v
    JOIN packages p ON p.id = v.package_id
"#;

#[async_trait]
impl VoucherRepository for PostgresVoucherRepository {
    async fn create(&self, n: NewVoucher) -> AppResult<Voucher> {
        let id = Uuid::new_v4();
        insert_one(&self.pool, id, &n).await?;
        self.find_by_id(id)
            .await?
            .ok_or_else(|| crate::error::AppError::Internal(anyhow::anyhow!("voucher vanished right after insert")))
    }

    async fn create_batch(&self, new_vouchers: Vec<NewVoucher>) -> AppResult<Vec<Voucher>> {
        let mut ids = Vec::with_capacity(new_vouchers.len());
        for n in &new_vouchers {
            let id = Uuid::new_v4();
            insert_one(&self.pool, id, n).await?;
            ids.push(id);
        }
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(v) = self.find_by_id(id).await? {
                out.push(v);
            }
        }
        Ok(out)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Voucher>> {
        let row = sqlx::query_as::<_, VoucherRow>(&format!("{SELECT_VOUCHER} WHERE v.id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(Voucher::try_from).transpose()
    }

    async fn list(&self) -> AppResult<Vec<Voucher>> {
        let rows = sqlx::query_as::<_, VoucherRow>(&format!("{SELECT_VOUCHER} ORDER BY v.created_at DESC"))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Voucher::try_from).collect()
    }

    async fn set_active(&self, id: Uuid, active: bool) -> AppResult<Option<Voucher>> {
        let result = sqlx::query("UPDATE vouchers SET active = $2 WHERE id = $1")
            .bind(id)
            .bind(active)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM vouchers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn find_by_code(&self, code: &str) -> AppResult<Option<Voucher>> {
        let row = sqlx::query_as::<_, VoucherRow>(&format!("{SELECT_VOUCHER} WHERE upper(v.code) = upper($1)"))
            .bind(code)
            .fetch_optional(&self.pool)
            .await?;
        row.map(Voucher::try_from).transpose()
    }

    async fn redeem(&self, voucher_id: Uuid, student_id: Uuid) -> AppResult<Voucher> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("INSERT INTO voucher_redemptions (voucher_id, student_id) VALUES ($1, $2)")
            .bind(voucher_id)
            .bind(student_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                if let sqlx::Error::Database(ref db_err) = e {
                    if db_err.code().as_deref() == Some("23505") {
                        return crate::error::AppError::Conflict("kamu sudah pernah menukarkan voucher ini".to_string());
                    }
                }
                crate::error::AppError::Database(e)
            })?;

        sqlx::query("UPDATE vouchers SET used_count = used_count + 1 WHERE id = $1")
            .bind(voucher_id)
            .execute(&mut *tx)
            .await?;

        let row = sqlx::query_as::<_, VoucherRow>(&format!("{SELECT_VOUCHER} WHERE v.id = $1"))
            .bind(voucher_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("voucher {voucher_id} not found")))?;

        tx.commit().await?;
        Voucher::try_from(row)
    }

    async fn student_has_redemption(&self, student_id: Uuid) -> AppResult<bool> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM voucher_redemptions WHERE student_id = $1)")
                .bind(student_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(exists)
    }

    async fn student_redeemed_package_ids(&self, student_id: Uuid) -> AppResult<Vec<Uuid>> {
        let ids: Vec<Uuid> = sqlx::query_scalar(
            r#"SELECT DISTINCT v.package_id
               FROM voucher_redemptions vr
               JOIN vouchers v ON v.id = vr.voucher_id
               WHERE vr.student_id = $1"#,
        )
        .bind(student_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(ids)
    }
}

async fn insert_one(pool: &PgPool, id: Uuid, n: &NewVoucher) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO vouchers
            (id, code, voucher_type, package_id, discount_type, discount_value, max_uses,
             expires_at, note, school_name, referrer_name, referrer_commission, created_by)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)"#,
    )
    .bind(id)
    .bind(&n.code)
    .bind(n.voucher_type.as_str())
    .bind(n.package_id)
    .bind(n.discount_type.as_str())
    .bind(n.discount_value)
    .bind(n.max_uses)
    .bind(n.expires_at)
    .bind(&n.note)
    .bind(&n.school_name)
    .bind(&n.referrer_name)
    .bind(n.referrer_commission)
    .bind(n.created_by)
    .execute(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.code().as_deref() == Some("23503") {
                return crate::error::AppError::Validation("paket yang dipilih tidak ditemukan".to_string());
            }
            if db_err.code().as_deref() == Some("23505") {
                return crate::error::AppError::Conflict("kode voucher sudah digunakan, coba lagi".to_string());
            }
        }
        crate::error::AppError::Database(e)
    })?;
    Ok(())
}
