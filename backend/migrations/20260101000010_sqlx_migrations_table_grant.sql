-- `GRANT CREATE ON SCHEMA public` (see 20260101000001) only lets `lms_app` create
-- `_sqlx_migrations` if it doesn't exist yet. In practice this migration set is always
-- applied the first time via a superuser connection (DDL on `core.*`/`CREATE SCHEMA ptn`
-- requires it), so Postgres makes that superuser the table's owner — `lms_app` then has
-- no actual row-level rights on it, and every subsequent app boot (which re-runs
-- `sqlx::migrate!` against `DATABASE_URL`, i.e. as `lms_app`) fails with "permission
-- denied for table _sqlx_migrations". Idempotent and safe to run as either role.
GRANT SELECT, INSERT, UPDATE, DELETE ON public._sqlx_migrations TO lms_app;
