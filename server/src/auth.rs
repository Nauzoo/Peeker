use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts}
};

use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, EncodingKey, Header, decode, DecodingKey, Validation};
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    //pub dvc_id: String,
    pub role: String,
    pub iat: usize,
    pub exp: usize
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub device: String
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    //pub device: String,
    pub role: String
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
}


const SECRET_KEY: &[u8] = b"dabadoo_77985?"; // REMOVER ESSA INFORMAÇÃO DO CÓDIGO, PASSAR PARA UMA VARIAVEL DE AMBIENTE

impl<S> FromRequestParts<S> for Claims  
where
    S: Send + Sync, 
{
    type Rejection = StatusCode;

    async fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {

        let auth_header = parts.headers.get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = &auth_header[7..];

        let decoded_token = decode::<Claims>(
            token, 
            &DecodingKey::from_secret(SECRET_KEY),
            &Validation::default(),
        ).map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(decoded_token.claims)
    }
}


pub fn generate_token(user_id: &str, level: &str) -> String {
    

    let current_momment = Utc::now().timestamp() as usize;
    let expiration = current_momment + (Duration::hours(24).num_seconds() as usize);

    let my_claim = Claims {
        sub: user_id.to_owned(),
        // dvc_id: device_id.to_owned(),
        role: level.to_owned(),
        iat: current_momment,
        exp: expiration
    };

    let header = Header::default();
    
    encode(
        &header,
        &my_claim,
        &EncodingKey::from_secret(SECRET_KEY)
    ).expect("Panick! Failed to generate JWT token.")

}