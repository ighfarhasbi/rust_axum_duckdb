use std::sync::{Arc, Mutex};

use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use duckdb::{params, Connection};
use serde_json::json;
use crate::{models::{ReqLogin, ReqUser}, response_model::{ResponseModel, ResponseUser}};

pub async fn add_user(conn: Extension<Arc<Mutex<Connection>>>, req_user: Json<ReqUser>) -> Result<Response, (StatusCode, String)> {
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
    let token = "tokentokentokentoken";
    let mut stmt = conn.prepare("INSERT INTO user (id, username, password, token) VALUES (?,?,?,?)")
       .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    stmt.execute(params![req_user.id, req_user.username, req_user.password, token])
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
    
    // Update token dulu
    let token = "token4";
    let mut stmt = conn.prepare("UPDATE user SET token = ? WHERE username = ? AND password =?")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let _ = stmt.execute(params![token, req_login.username, req_login.password]);

    // Login User 
    stmt = conn.prepare("SELECT * FROM user WHERE username =? AND password =?")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let rows = stmt.query_map(params![req_login.username, req_login.password], |row| {
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