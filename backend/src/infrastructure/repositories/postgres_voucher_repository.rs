use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::repository::{NewVoucher, VoucherRepository};
use crate::domain::tenant::TenantCtx;
use crate::domain::voucher::{DiscountType, Voucher, VoucherType};
use crate::error::AppResult;

pub struct PostgresVoucherRepository {
    pool: PgPool,
}

impl PostgresVoucherRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn begin_tx(&self, ctx: &TenantCtx) -> AppResult<Transaction<'static, Postgres>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)")
            .bind(ctx.tenant_id.to_string())
            .bind(if ctx.is_platform_admin { "true" } else { "false" })
            .execute(&mut *tx)
            .await?;
        Ok(tx)
    }
}

#[derive(sqlx::FromRow)]
struct VoucherRow {
    public_id: Uuid,
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
            id: r.public_id,
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
    SELECT v.public_id, v.code, v.voucher_type, p.public_id AS package_id, p.name AS package_name,
           v.discount_type, v.discount_value, v.max_uses, v.used_count, v.active,
           v.expires_at, v.note, v.school_name, v.referrer_name, v.referrer_commission,
           (SELECT public_id FROM core.akun WHERE core.akun.id = v.created_by) AS created_by,
           v.created_at
    FROM ptn.vouchers v
    JOIN ptn.packages p ON p.id = v.package_id
"#;

#[async_trait]
impl VoucherRepository for PostgresVoucherRepository {
    async fn create(&self, n: NewVoucher, ctx: &TenantCtx) -> AppResult<Voucher> {
        let mut tx = self.begin_tx(ctx).await?;
        let public_id = insert_one(&mut tx, ctx, &n).await?;
        let row = sqlx::query_as::<_, VoucherRow>(&format!("{SELECT_VOUCHER} WHERE v.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Voucher::try_from(row)
    }

    async fn create_batch(&self, new_vouchers: Vec<NewVoucher>, ctx: &TenantCtx) -> AppResult<Vec<Voucher>> {
        let mut tx = self.begin_tx(ctx).await?;
        let mut public_ids = Vec::with_capacity(new_vouchers.len());
        for n in &new_vouchers {
            public_ids.push(insert_one(&mut tx, ctx, n).await?);
        }
        let mut out = Vec::with_capacity(public_ids.len());
        for public_id in public_ids {
            let row = sqlx::query_as::<_, VoucherRow>(&format!("{SELECT_VOUCHER} WHERE v.public_id = $1"))
                .bind(public_id)
                .fetch_optional(&mut *tx)
                .await?;
            if let Some(row) = row {
                out.push(Voucher::try_from(row)?);
            }
        }
        tx.commit().await?;
        Ok(out)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<Voucher>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, VoucherRow>(&format!("{SELECT_VOUCHER} WHERE v.public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(Voucher::try_from).transpose()
    }

    async fn list(&self, ctx: &TenantCtx) -> AppResult<Vec<Voucher>> {
        let mut tx = self.begin_tx(ctx).await?;
        let rows = sqlx::query_as::<_, VoucherRow>(&format!("{SELECT_VOUCHER} ORDER BY v.created_at DESC"))
            .fetch_all(&mut *tx)
            .await?;
        tx.commit().await?;
        rows.into_iter().map(Voucher::try_from).collect()
    }

    async fn set_active(&self, id: Uuid, active: bool, ctx: &TenantCtx) -> AppResult<Option<Voucher>> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("UPDATE ptn.vouchers SET active = $2 WHERE public_id = $1")
            .bind(id)
            .bind(active)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, VoucherRow>(&format!("{SELECT_VOUCHER} WHERE v.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Voucher::try_from(row).map(Some)
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.vouchers WHERE public_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn find_by_code(&self, code: &str, ctx: &TenantCtx) -> AppResult<Option<Voucher>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, VoucherRow>(&format!("{SELECT_VOUCHER} WHERE upper(v.code) = upper($1)"))
            .bind(code)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(Voucher::try_from).transpose()
    }

    async fn redeem(&self, voucher_id: Uuid, student_id: Uuid, ctx: &TenantCtx) -> AppResult<Voucher> {
        let mut tx = self.begin_tx(ctx).await?;

        let voucher_pk: i64 = sqlx::query_scalar("SELECT id FROM ptn.vouchers WHERE public_id = $1")
            .bind(voucher_id)
            .fetch_one(&mut *tx)
            .await?;
        let student_pk: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(student_id)
            .fetch_one(&mut *tx)
            .await?;

        sqlx::query("INSERT INTO ptn.voucher_redemptions (voucher_id, student_id) VALUES ($1, $2)")
            .bind(voucher_pk)
            .bind(student_pk)
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

        sqlx::query("UPDATE ptn.vouchers SET used_count = used_count + 1 WHERE id = $1")
            .bind(voucher_pk)
            .execute(&mut *tx)
            .await?;

        let row = sqlx::query_as::<_, VoucherRow>(&format!("{SELECT_VOUCHER} WHERE v.public_id = $1"))
            .bind(voucher_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("voucher {voucher_id} not found")))?;

        tx.commit().await?;
        Voucher::try_from(row)
    }

    async fn student_has_redemption(&self, student_id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = self.begin_tx(ctx).await?;
        let exists: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(
                SELECT 1 FROM ptn.voucher_redemptions vr
                JOIN core.akun a ON a.id = vr.student_id
                WHERE a.public_id = $1
               )"#,
        )
        .bind(student_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(exists)
    }

    async fn student_redeemed_package_ids(&self, student_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<Uuid>> {
        let mut tx = self.begin_tx(ctx).await?;
        let ids: Vec<Uuid> = sqlx::query_scalar(
            r#"SELECT DISTINCT p.public_id
               FROM ptn.voucher_redemptions vr
               JOIN ptn.vouchers v ON v.id = vr.voucher_id
               JOIN ptn.packages p ON p.id = v.package_id
               JOIN core.akun a ON a.id = vr.student_id
               WHERE a.public_id = $1"#,
        )
        .bind(student_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(ids)
    }
}

async fn insert_one(tx: &mut Transaction<'static, Postgres>, ctx: &TenantCtx, n: &NewVoucher) -> AppResult<Uuid> {
    let package_id: i64 = sqlx::query_scalar("SELECT id FROM ptn.packages WHERE public_id = $1")
        .bind(n.package_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| crate::error::AppError::Validation("paket yang dipilih tidak ditemukan".to_string()))?;
    let created_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
        .bind(n.created_by)
        .fetch_one(&mut **tx)
        .await?;

    let public_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO ptn.vouchers
            (tenant_id, code, voucher_type, package_id, discount_type, discount_value, max_uses,
             expires_at, note, school_name, referrer_name, referrer_commission, created_by)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
           RETURNING public_id"#,
    )
    .bind(ctx.tenant_id)
    .bind(&n.code)
    .bind(n.voucher_type.as_str())
    .bind(package_id)
    .bind(n.discount_type.as_str())
    .bind(n.discount_value)
    .bind(n.max_uses)
    .bind(n.expires_at)
    .bind(&n.note)
    .bind(&n.school_name)
    .bind(&n.referrer_name)
    .bind(n.referrer_commission)
    .bind(created_by_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.code().as_deref() == Some("23505") {
                return crate::error::AppError::Conflict("kode voucher sudah digunakan, coba lagi".to_string());
            }
        }
        crate::error::AppError::Database(e)
    })?;
    Ok(public_id)
}
