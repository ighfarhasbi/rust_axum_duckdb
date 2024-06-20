use std::sync::{Arc, Mutex};

use axum::{
    http::{Request, StatusCode}, 
    middleware::Next, 
    response::Response,
    extract::Extension
    };
use axum_extra::headers::{authorization::Bearer, Authorization, HeaderMapExt};
use axum::body::Body;
use duckdb::{params, Connection};

use crate::response_model::ResponseUser;

pub async fn guard_route (
    conn: Extension<Arc<Mutex<Connection>>>,
    mut request: Request<Body>,
    next: Next
) -> Result<Response, (StatusCode, String) >{
    // ambil token dulu
    let token = request
    .headers().typed_get::<Authorization<Bearer>>()
        .ok_or((StatusCode::BAD_REQUEST, "token tidak ditemukan".to_string()))?
        .token()
        .to_owned();
    println!("ini tokennya, {:?}", token);

    let extension = request.extensions_mut();
    let user = cek_token(conn, token);
    match user {
        Ok(result) => {
            println!("ini usernya dari guard = {:?}", result);
            if result.token == "" {
                return Err((StatusCode::UNAUTHORIZED, "Token tidak valid".to_string())); // jika token tidak valid maka akan berhenti disini
            } else {
                extension.insert(result);
            }
        }
        Err(_) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Error pada server".to_string()));
        }
    }
    let response = next.run(request).await;
    Ok(response)
}

fn cek_token(conn: Extension<Arc<Mutex<Connection>>>, token: String) -> Result<ResponseUser, StatusCode> {
    // cek token dengan yang ada di db
    let conn = conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut stmt = conn.prepare("SELECT * FROM user WHERE token =?")
       .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result = stmt.query_map(params![token],
        |row| {
        Ok(ResponseUser {
            id: row.get(0)?,
            username: row.get(1)?,
            token: row.get(3)?,
        })
    }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // inisisasi struct kosong
    let mut user_result = ResponseUser {
        id: 0,
        username: "".to_string(),
        token: "".to_string()
    };
    // mengubah tipe data dari MappedRows pada var user_id_result menjadi struck ResponseUser
    for user in result { //masuk for ini pasti data ada, atau stmt di execution hasilnya ada
        let user = user.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        println!("ini usernya, {:?}", user);
        if user.token!= token { 
            return Err(StatusCode::UNAUTHORIZED); // Kondisi Err ini tidak pernah terjadi
        } else {
            user_result = ResponseUser {
                id: user.id,
                username: user.username,
                token: user.token,
            };
        }
    }
    Ok(ResponseUser {
        id: user_result.id,
        username: user_result.username,
        token: user_result.token
    })
}