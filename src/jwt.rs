use chrono::{Duration, Utc};
use dotenvy_macro::dotenv;
use hyper::StatusCode;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]

pub struct Claims {
    exp: usize,
    iat: usize,
}

// Buat token dan waktu exp nya
pub fn create_jwt(exp_duration: Duration) -> Result<String, StatusCode> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + exp_duration).timestamp() as usize; // duration for the token to expire

    let claims = Claims {
        exp,
        iat,
    };

    let secret = dotenv!("JWT_SECRET_KEY");
    let key = &EncodingKey::from_secret(secret.as_bytes());
    encode(&Header::default(), &claims, key)
        .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)
}

// Pengecekan apakah token masih valid atau tidak dilihat dari waktu exp dan kevalidan token
pub fn is_valid(token: &str) -> Result<Claims, StatusCode> {
    let secret = dotenv!("JWT_SECRET_KEY");
    let key = &DecodingKey::from_secret(secret.as_bytes());
    decode::<Claims>(token, &key, &Validation::new(Algorithm::HS256))
        .map(|data| data.claims)
        .map_err(|err| match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })
}

pub fn refresh_access_token(refresh_token: &str) -> Result<String, StatusCode> {
    match is_valid(refresh_token) {
        Ok(claims) => {
            let now = Utc::now().timestamp() as usize;
            if claims.exp > now {
                create_access_token()
            } else {
                Err(StatusCode::UNAUTHORIZED) // Refresh token has expired
            }
        }
        Err(e) => Err(e), // Invalid refresh token
    }
}

pub fn create_access_token() -> Result<String, StatusCode> {
    create_jwt(Duration::seconds(30))
}

pub fn create_refresh_token() -> Result<String, StatusCode> {
    create_jwt(Duration::days(1))
}