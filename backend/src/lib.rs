//! EdvionPTN backend library root — re-exports the layered architecture modules so
//! both `main.rs` and integration tests can wire them up.
//!
//! Layers (dependency direction goes downward — outer layers depend on inner ones,
//! never the reverse):
//!   interfaces (HTTP/Axum)
//!     -> application (use-case services)
//!       -> domain (entities + repository traits)
//!   infrastructure (Postgres) implements domain::repository traits, and is wired
//!   in at the composition root (`main.rs`) only.

pub mod config;
pub mod error;
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interfaces;
