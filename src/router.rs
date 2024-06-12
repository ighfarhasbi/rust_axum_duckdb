// src/router.rs

use axum::{
    routing::{delete, get, post, put}, Extension, Router
};
use std::sync::{Arc, Mutex};
use crate::handlers::{get_mahasiswa, add_mahasiswa, export_mahasiswa, update_mahasiswa, delete_mahasiswa};
use duckdb::Connection;

pub fn create_router(conn: Arc<Mutex<Connection>>) -> Router {
    Router::new()
        .route("/mahasiswa", get(get_mahasiswa))
        .route("/mahasiswa", post(add_mahasiswa))
        .route("/mahasiswa/:id", put(update_mahasiswa))
        .route("/mahasiswa/:id", delete(delete_mahasiswa))
        .route("/export-mahasiswa/:nama_file", get(export_mahasiswa))
        .layer(Extension(conn))
}
