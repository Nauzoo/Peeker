use axum::{
    Json, extract::{Extension, FromRequestParts, State}, http::{StatusCode, request::Parts}
};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use tower_cookies::{Cookie, Cookies};

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, ActiveModelTrait, Set};

use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, EncodingKey, Header, DecodingKey, Validation};
use chrono::{Utc, Duration};

use crate::entities::users;


#[derive(Clone)]
pub struct AppState { // Global AppState, contains usefull information for many app routines. 
    pub db: DatabaseConnection,
}

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
    // pub device: String
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    //pub device: String,
    pub role: String
}


const SECRET_KEY: &[u8] = b"dabadoo_77985?"; // TODO: Remove this hardcoded information, save it to an env. variable  

impl<S> FromRequestParts<S> for Claims  
where
    S: Send + Sync, 
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        
        // Extracting cookies using tower-cookie
        let cookies = parts
            .extensions
            .get::<Cookies>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // Searches for "auth_token" from the cookies list 
        let token_string = cookies
            .get("auth_token")
            .map(|c| c.value().to_string())
            .ok_or(StatusCode::UNAUTHORIZED)?; // Returns 401 if cookie not found (user is not logged in correctly)
        

        // Decoodes the token data using the server's secret key
        let decoded_token = decode::<Claims>(
            &token_string, 
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
    ).expect("Panic! Failed to generate JWT token.")

}

pub async fn login(
    State(state): State<AppState>, 
    Extension(cookies): Extension<Cookies>,
    Json(payload): Json<LoginRequest>)
     -> Result<StatusCode, StatusCode> {
    

    let user_found = users::Entity::find().filter(users::Column::Username.eq(&payload.username))
    .one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?; 


    let db_hash = user_found.password_hash;

    let parsed_hash = PasswordHash::new(&db_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verifies if the provided password (payload.password) generates the same hash as the one saved on db.
    let is_valid = Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !is_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let generated_token = generate_token(&user_found.id.to_string(), &user_found.role);

    let mut cookie = Cookie::new("auth_token", generated_token);
    cookie.set_http_only(true);
    cookie.set_path("/");

    cookies.add(cookie);

    Ok(StatusCode::OK)
}

// Note related to axum extractors : Extractors are a kind of handler that recives data from an HTTP request and converts it into
// native rust struct
pub async fn register(
    State(state): State<AppState>, 
    Json(payload): Json<RegisterRequest>) // <-- This weird @ss syntax Json(payload) is an extractor
    -> Result<StatusCode, StatusCode> {
    /* Registration handler : Recives a JSON payloand and uses an extractor to access its' information */
    
    let salt = SaltString::generate(&mut OsRng);
    
    let password_hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    // Preparing the data for SeaORM
    // ActiveModel is used when inserting or updating data.
    // "Set()" tells SeaORM which fields were updated.
    let new_user = users::ActiveModel {
        username: Set(payload.username),
        password_hash: Set(password_hash),
        role: Set(payload.role),
        ..Default::default() // This quirk makes that SQLite automatically generates the entity "id" by ignoring it
        // PS: It will also ignore new fields added to the model
    };
    
    // Saving the user in the databank
    match new_user.insert(&state.db).await {
        Ok(_) => Ok(StatusCode::CREATED),    // Returns 201 Created if everything goes fine
        Err(_) => Err(StatusCode::CONFLICT), // Returns 409 Conflict if the user already exisists
    }

}