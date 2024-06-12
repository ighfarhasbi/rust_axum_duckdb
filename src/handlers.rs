use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use duckdb::{params, Connection};
use serde_json::json;
use std::{fs, path::Path, sync::{Arc, Mutex}};
use crate::{models::Mahasiswa, response_model::ResponseModel};

pub async fn get_mahasiswa(
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
) -> Result<Response, (StatusCode, String)> {
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
    let mut stmt = conn.prepare("SELECT id, nama, tgl_lahir FROM data_mahasiswa")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mahasiswa_iter = stmt.query_map([], |row| {
        let tgl_lahir: i32 = row.get(2)?;
        let tgl_lahir_str = duckdb_date_to_string(tgl_lahir);

        Ok(Mahasiswa {
            id: row.get(0)?,
            nama: row.get(1)?,
            tgl_lahir: tgl_lahir_str,
        })
    }).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut mahasiswas = vec![];
    for mahasiswa in mahasiswa_iter {
        mahasiswas.push(mahasiswa.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?);
    }

    let result = ResponseModel {
        kode: 200,
        status: "Sukses".to_string(),
        data: Some(&mahasiswas)
    };

    Ok((StatusCode::OK, Json(result)).into_response())
}

pub async fn add_mahasiswa(
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    Json(item): Json<Mahasiswa>,
) -> Result<Response, (StatusCode, String)> {
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
    let mut stmt = conn.prepare("INSERT INTO data_mahasiswa (id, nama, tgl_lahir) VALUES (?, ?, ?)")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    stmt.execute(params![item.id, item.nama, item.tgl_lahir])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _exp = conn.execute("COPY data_mahasiswa TO 'coba_mahasiswa_exp.parquet' (FORMAT PARQUET);", [])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Persiapan path setelah export sukses
    let src_path = Path::new("coba_mahasiswa_exp.parquet");
    let dest_dir = Path::new("src/parquet");
    let dest_path = dest_dir.join("coba_mahasiswa_exp.parquet");

    // memastikan directory ada
    if !dest_dir.exists() {
        fs::create_dir_all(dest_dir).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // memindahkan file hasil export
    fs::rename(src_path, dest_path).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;


    Ok((StatusCode::OK, Json(json!({"status": "success"}))).into_response())
}

pub async fn export_mahasiswa (
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
) -> Result<Response, (StatusCode, String)> {
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
    let _exp = conn.execute("COPY data_mahasiswa TO 'coba_mahasiswa_exp.parquet' (FORMAT PARQUET);", [])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Persiapan path setelah export sukses
    let src_path = Path::new("coba_mahasiswa_exp.parquet");
    let dest_dir = Path::new("src/parquet");
    let dest_path = dest_dir.join("coba_mahasiswa_exp.parquet");

    // memastikan directory ada
    if !dest_dir.exists() {
        fs::create_dir_all(dest_dir).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // memindahkan file hasil export
    fs::rename(src_path, dest_path).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;


    Ok((StatusCode::OK, Json(json!({"status": "sukses" }))).into_response())
}

fn duckdb_date_to_string(date: i32) -> String {
    let epoch_days = date as i64;
    let unix_epoch = chrono::NaiveDate::from_ymd(1970, 1, 1);
    let date = unix_epoch + chrono::Duration::days(epoch_days);
    date.to_string()
}