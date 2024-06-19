// src/router.rs

use axum::{
    routing::{delete, get, post, put}, Extension, Router
};
use std::sync::{Arc, Mutex};
use crate::{
    handlers::{add_mahasiswa, delete_mahasiswa, export_mahasiswa, get_mahasiswa, update_mahasiswa}, 
    user_handler::{add_user, login_user, logout_user}
    };
use duckdb::Connection;

pub fn create_router(conn: Arc<Mutex<Connection>>) -> Router {
    Router::new()
        .route("/mahasiswa", get(get_mahasiswa))
        .route("/mahasiswa", post(add_mahasiswa))
        .route("/mahasiswa/:id", put(update_mahasiswa))
        .route("/mahasiswa/:id", delete(delete_mahasiswa))
        .route("/user", post(add_user))
        .route("/user/login", post(login_user))
        .route("/user/logout", post(logout_user))
        .route("/export-mahasiswa/:nama_file", get(export_mahasiswa))
        .layer(Extension(conn))
}
