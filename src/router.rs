// src/router.rs

use axum::{
    routing::{get, post},
    Router,
    Extension,
};
use std::sync::{Arc, Mutex};
use crate::handlers::{get_mahasiswa, add_mahasiswa, export_mahasiswa};
use duckdb::Connection;

pub fn create_router(conn: Arc<Mutex<Connection>>) -> Router {
    Router::new()
        .route("/mahasiswa", get(get_mahasiswa))
        .route("/mahasiswa", post(add_mahasiswa))
        .route("/export-mahasiswa", get(export_mahasiswa))
        .layer(Extension(conn))
}
