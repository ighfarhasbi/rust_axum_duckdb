use std::sync::{Arc, Mutex};

use axum::{
    extract::Extension, http::{Request, StatusCode}, middleware::Next, response::Response
    };
use axum_extra::headers::{authorization::Bearer, Authorization, HeaderMapExt};
use axum::body::Body;
use duckdb::{params, Connection};

use crate::{jwt::{is_valid, refresh_access_token}, response_model::ResponseLoginRefUser};

pub async fn guard_route (
    conn: Extension<Arc<Mutex<Connection>>>,
    mut request: Request<Body>,
    next: Next
) -> Result<Response, (StatusCode, String) >{

    let conn2 = conn.clone();
    // ambil token dulu
    let token = request
    .headers().typed_get::<Authorization<Bearer>>()
        .ok_or((StatusCode::BAD_REQUEST, "token tidak ditemukan".to_string()))?
        .token()
        .to_owned();
    // println!("ini tokennya, {:?}", token);

    let extension = request.extensions_mut();
    let user = cek_token(conn, token.clone());
    let mut ref_token_str = "".to_string();
    match user {
        Ok(result) => {
            // println!("ini usernya dari guard = {:?}", result);
            if result.token == "" {
                return Err((StatusCode::UNAUTHORIZED, "Token tidak valid".to_string())); // jika token tidak valid maka akan berhenti disini
            } else {
                ref_token_str = result.refresh_token.clone();
                extension.insert(result);
            }
        }
        Err(_) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Error pada server".to_string()));
        }
    }

    // cek apakah token valid atau tidak (expired atau belum)
    let dur = is_valid(&token);
    println!("ini dur = {:?}", dur);
    match dur {
        Ok(_) => {
            // tidak melakukan apapa jika token ada
        }
        Err(_e) => {
            let new_acc_token = refresh_access_token(&ref_token_str);
            match new_acc_token {
                Ok(new_access_token) => { 
                    // println!("access token baru => {:?}", new_acc_token);
                    let _ = update_token(conn2, new_access_token, token);
                    // disini harusnya fungsi rekursifnya, manggil si fn guard_route
                }
                Err(_) => {return Err((StatusCode::UNAUTHORIZED, "Token expired".to_string()));}  
            }
        }
    }
    let response = next.run(request).await;
    Ok(response)
}

fn cek_token(conn: Extension<Arc<Mutex<Connection>>>, token: String) -> Result<ResponseLoginRefUser, StatusCode> {
    
    // cek token dengan yang ada di db
    let conn = conn.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut stmt = conn.prepare("SELECT * FROM token_user WHERE access_token =?")
       .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result = stmt.query_map(params![token],
        |row| {
        Ok(ResponseLoginRefUser {
            id: row.get(1)?,
            username: row.get(4)?,
            token: row.get(2)?,
            refresh_token: row.get(3)?,
        })
    }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // inisisasi struct kosong
    let mut user_result = ResponseLoginRefUser {
        id: 0,
        username: "".to_string(),
        token: "".to_string(),
        refresh_token: "".to_string()
    };
    // mengubah tipe data dari MappedRows pada var user_id_result menjadi struck ResponseUser
    for user in result { //masuk for ini pasti data ada, atau stmt di execution hasilnya ada
        let user = user.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        // println!("ini usernya, {:?}", user);
        if user.token!= token { 
            return Err(StatusCode::UNAUTHORIZED); // Kondisi Err ini tidak pernah terjadi
        } else {
            user_result = ResponseLoginRefUser {
                id: user.id,
                username: user.username,
                token: user.token,
                refresh_token: user.refresh_token,
            };
        }
    }
    Ok(ResponseLoginRefUser {
        id: user_result.id,
        username: user_result.username,
        token: user_result.token,
        refresh_token: user_result.refresh_token
    })
}

fn update_token (conn: Extension<Arc<Mutex<Connection>>>, new_token: String, old_token: String) -> Result<(), (StatusCode, String)> {
    let conn = conn.lock().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock connection".to_string()))?;
    let mut stmt = conn.prepare("UPDATE token_user SET access_token = ? WHERE access_token = ? ")
       .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    stmt.execute(params![new_token, old_token])
       .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    println!("ini token baru = {}", new_token);
    Ok(())
}