use std::sync::{Arc, Mutex};

use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bcrypt::{hash, verify};
use duckdb::{params, Connection};
use serde_json::json;
use crate::{jwt::create_jwt, models::{ReqLogin, ReqUser}, response_model::{ResponseModel, ResponseUser}};

pub async fn add_user(conn: Extension<Arc<Mutex<Connection>>>, req_user: Json<ReqUser>) -> Result<Response, (StatusCode, String)> {
    let pass = hash_password(req_user.password.clone()).unwrap();
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
    let token = create_jwt().unwrap();
    let mut stmt = conn.prepare("INSERT INTO user (id, username, password, token) VALUES (?,?,?,?)")
       .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    stmt.execute(params![req_user.id, req_user.username, pass, token])
       .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, Json(ResponseUser {
        username: req_user.username.clone(),
        id: req_user.id,
        token: token.to_string()
    })).into_response())
}

pub async fn login_user (
    conn: Extension<Arc<Mutex<Connection>>>, 
    req_login: Json<ReqLogin>) -> Result<Response, (StatusCode, String)> {
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
    
    let mut stmt_user = conn.prepare("SELECT password FROM user WHERE username = ?")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    struct ResultRow {password: String}
    let rows = stmt_user.query_map(params![req_login.username], |row| {
        Ok(ResultRow {
            password: row.get(0)?
        })
    }).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut result_user = ResultRow {
        password: "".to_string()
    };
    for user in rows {
        match user {
            Ok(res) => {
                result_user.password = res.password
            }
            Err(e) => {
                return Err((StatusCode::UNAUTHORIZED, e.to_string()));
            }
        }
    }

    // Validasi username dan password
    if result_user.password.is_empty() { // menangkap ketika username salah, sehingga result_user.password kosong
        return Err((StatusCode::UNAUTHORIZED, "Password atau username salah".to_string()));
    }
    // menangkap jika username benar dan password salah
    if !verify_password(req_login.password.clone(), &result_user.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? {
            return Err((StatusCode::UNAUTHORIZED, "Password atau username salah".to_string()));
    };

    // Update token dulu
    let token = create_jwt().unwrap();
    let mut stmt = conn.prepare("UPDATE user SET token = ? WHERE username = ?")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let _ = stmt.execute(params![token, req_login.username]);

    // ambil value id, username dan token dari table user
    stmt = conn.prepare("SELECT * FROM user WHERE username =?")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let rows = stmt.query_map(params![req_login.username], |row| {
        Ok(ResponseUser {
            id: row.get(0)?,
            username: row.get(1)?,
            token: row.get(3)?,
        })
    }).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // inisialisasi struct default
    let mut result = ResponseModel { 
        kode: 404,
        status: "Gagal".to_string(),
        data: None
    };
    // mengubah tipe data dari var "rows" menjadi struct ResponseUser berbentuk result
    for user in rows {
        match user { // mengubah result menjadi struct ResponseUser
            Ok(user) => {
                result = ResponseModel {
                    kode: 200,
                    status: "Sukses".to_string(),
                    data: Some(user) // masukan struct ResponseUser ke struct ResponseModel.data
                };
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    };

    // mengecek apakah struct ResponseModel.data ada isinya atau tidak
    match &result.data {
        Some(_) => {
            Ok((StatusCode::OK, Json(result)).into_response())
        }
        None => {
            Ok((StatusCode::NOT_FOUND, Json(result)).into_response())
        }
    }
}

pub async fn logout_user(
    conn: Extension<Arc<Mutex<Connection>>>,
    user: Extension<ResponseUser>
) -> Result<Response, (StatusCode, String)> {
    println!("dari fn logout {:?}", user.token);
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
    let mut stmt = conn.prepare("UPDATE user SET token = NULL WHERE token = ?")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let result = stmt.execute(params![user.token]);
    match result {
        Ok(res) => {
            if res == 0 {
                Ok((StatusCode::UNAUTHORIZED, Json(json!({"status": "Unauthorized" }))).into_response())
            } else {
                Ok((StatusCode::OK, Json(json!({"status": "sukses" }))).into_response())
            }
        }
        Err(e) => {
            Ok((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())
        }
    }
}

fn hash_password(password: String) -> Result<String, StatusCode> {
    hash(password, 12).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn verify_password(password: String, hash: &str) -> Result<bool, StatusCode> {
    verify(password, hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}