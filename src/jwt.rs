use axum::response::{IntoResponse, Response};
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

pub fn create_jwt() -> Result<String, StatusCode> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp_duration = Duration::seconds(30); // duration for the token to expire
    println!("{}", exp_duration);
    let exp = (now + exp_duration).timestamp() as usize;

    let claims = Claims {
        exp,
        iat,
    };

    let secret = dotenv!("JWT_SECRET_KEY");
    let key = &EncodingKey::from_secret(secret.as_bytes());
    encode(&Header::default(), &claims, &key)
        .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn is_valid(token: &str) -> Result<bool, Response> {
    let secret = dotenv!("JWT_SECRET_KEY");
    let key = DecodingKey::from_secret(secret.as_bytes());
    // println!("{:?}", key);

    decode::<Claims>(token, &key, &Validation::new(Algorithm::HS256))
        .map_err(|err| match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => StatusCode::UNAUTHORIZED.into_response(),
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        })?;

    // println!("{:?}", result);
        
    Ok(true)
}