//! Interfaces layer — the outermost layer. Translates HTTP <-> application layer:
//! routing, request/response DTOs, auth middleware. Nothing outside this layer should
//! know that Axum exists.

pub mod http;
