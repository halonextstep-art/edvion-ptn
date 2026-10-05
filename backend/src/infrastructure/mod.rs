//! Infrastructure layer — concrete adapters for outside-world concerns: the Postgres
//! connection pool and repository implementations that fulfil the `domain::repository`
//! traits. This is the only layer allowed to depend on `sqlx`.

pub mod db;
pub mod docx_reader;
pub mod encoding;
pub mod repositories;
pub mod storage;
