use axum::{
    extract::{Extension, Json, Path as JsonPath},
    http::StatusCode, 
    response::{IntoResponse, Response},
};
use duckdb::{params, Connection};
use serde_json::json;
use std::{fs, path::Path, sync::{Arc, Mutex, MutexGuard}};
use crate::{models::{CreateMahasiswa, Mahasiswa}, response_model::ResponseModel};

pub async fn get_mahasiswa(
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
) -> Result<Response, (StatusCode, String)> {
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
    let mut stmt = conn.prepare("SELECT id, nama, tgl_lahir, user_id FROM data_mahasiswa")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mahasiswa_iter = stmt.query_map([], |row| {
        let tgl_lahir: i32 = row.get(2)?;
        let tgl_lahir_str = duckdb_date_to_string(tgl_lahir);

        Ok(Mahasiswa {
            id: row.get(0)?,
            nama: row.get(1)?,
            tgl_lahir: tgl_lahir_str,
            user_id: row.get(3)?
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
    Json(item): Json<CreateMahasiswa>,
) -> Result<Response, (StatusCode, String)> {
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
    let mut stmt = conn.prepare("INSERT INTO data_mahasiswa (id, nama, tgl_lahir) VALUES (?, ?, ?)")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    stmt.execute(params![item.id, item.nama, item.tgl_lahir])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    to_parquet(&conn); // buat file parquet

    Ok((StatusCode::OK, Json(json!({"status": "success"}))).into_response())
}

pub async fn update_mahasiswa (conn: Extension<Arc<Mutex<Connection>>>, id: JsonPath<i32>, Json(item): Json<Mahasiswa>) -> Result<Response, (StatusCode, String)> {
    let conn = conn.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut stmt = conn.prepare("UPDATE data_mahasiswa SET nama = ?, tgl_lahir = ? WHERE id = ?")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    stmt.execute(params![item.nama, item.tgl_lahir, id.to_string()])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    to_parquet(&conn); // update file parquet

    Ok((StatusCode::OK, Json(json!({"status": "sukses" }))).into_response())

}

pub async fn delete_mahasiswa (conn: Extension<Arc<Mutex<Connection>>>, id: JsonPath<i32>) -> Result<Response, (StatusCode, String)> {
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
    let mut stmt = conn.prepare("DELETE FROM data_mahasiswa WHERE id =?")
       .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    stmt.execute(params![id.to_string()])
       .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    to_parquet(&conn); // update file parquet

    Ok((StatusCode::OK, Json(json!({"status": "sukses" }))).into_response())
}

pub async fn export_mahasiswa (
    Extension(conn): Extension<Arc<Mutex<Connection>>>,
    nama_file: JsonPath<String>
) -> Result<Response, (StatusCode, String)> {
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
     // Prepare the SQL statement
     let sql = format!("COPY data_mahasiswa TO '{}' (FORMAT PARQUET);", nama_file.to_string());
     let mut stmt = conn.prepare(&sql).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
     
     // Execute the statement
     stmt.execute([]).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
 
    

    // // Persiapan path setelah export sukses
    // let src_path = Path::new("coba_mahasiswa_exp.parquet");
    // let dest_dir = Path::new("src/parquet");
    // let dest_path = dest_dir.join("coba_mahasiswa_exp.parquet");

    // // memastikan directory ada
    // if !dest_dir.exists() {
    //     fs::create_dir_all(dest_dir).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    // }

    // // memindahkan file hasil export
    // fs::rename(src_path, dest_path).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;


    Ok((StatusCode::OK, Json(json!({"status": "sukses" }))).into_response())
}

fn duckdb_date_to_string(date: i32) -> String {
    let epoch_days = date as i64;
    let unix_epoch = chrono::NaiveDate::from_ymd(1970, 1, 1);
    let date = unix_epoch + chrono::Duration::days(epoch_days);
    date.to_string()
}

fn to_parquet(conn: &MutexGuard<Connection>) {
    let _exp = conn.execute("COPY data_mahasiswa TO 'coba_mahasiswa_exp.parquet' (FORMAT PARQUET);", [])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));

    let _set_key = conn.execute("PRAGMA add_parquet_key('key256', '01234567891123450123456789112345');", [])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    let _set_key = conn.execute("COPY coba_mahasiswa_exp.parquet TO 'coba_mahasiswa_exp_encrpt256.parquet' (ENCRYPTION_CONFIG {footer_key: 'key256'});", [])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));

    // Persiapan path setelah export sukses
    let src_path = Path::new("coba_mahasiswa_exp_encrpt256.parquet"); 
    let dest_dir = Path::new("src/parquet");
    let dest_path = dest_dir.join("coba_mahasiswa_exp_encrpt256.parquet");

    // jika folder src/parquet tidak ada, maka akan dibuat dulu
    if !dest_dir.exists() {
        let _add_dir = fs::create_dir_all(dest_dir).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    // jika folder src/parquet ada, maka pindahkan file .parquet
    let _rename = fs::rename(src_path, dest_path).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
}

// ini yang langsung ambil token dari request auth tanpa middleware
// pub async fn add_mahasiswa(
//     Extension(conn): Extension<Arc<Mutex<Connection>>>,
//     TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
//     Json(item): Json<CreateMahasiswa>,
// ) -> Result<Response, (StatusCode, String)> {
//     let token = authorization.token(); // amil token dari auth di postman
    
//     // mencari user yang melakukan method add ini
//     let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
//     let mut stmt = conn.prepare("SELECT id FROM user WHERE token =?")
//         .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
//     struct User {id: i32} // struct untuk id user terlogin 
//     let user_id_result = stmt.query_map(params![token],
//         |row| {
//         Ok(User {id: row.get(0)?})
//     }).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

//     // mengubah tipe data dari MappedRows pada var user_id_result menjadi struck User
//     let mut id_result = 0;
//     for id_user in user_id_result {
//         let user_id = id_user.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
//         id_result = user_id.id;
//         // println!("{}", id_result);
//     }

//     // cek apakah id user terlogin ada
//     if id_result != 0 {
//         stmt = conn.prepare("INSERT INTO data_mahasiswa (id, nama, tgl_lahir, user_id) VALUES (?, ?, ?, ?)")
//             .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
//         stmt.execute(params![item.id, item.nama, item.tgl_lahir, id_result])
//             .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

//         to_parquet(&conn); // buat file parquet

//         Ok((StatusCode::OK, Json(json!({"status": "success"}))).into_response())
//     } else {
//         Ok((StatusCode::UNAUTHORIZED, Json(json!({"status": "unauthorized"}))).into_response())
//     }

    
// }