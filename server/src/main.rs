use axum::{
    Json, Router, extract::{Path, Request}, http::StatusCode, response::{IntoResponse, Response }, routing::{get, post}
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use tower::ServiceExt;

use crate::auth::{LoginRequest, LoginResponse, generate_token};
mod auth;


fn validate_path(base_path: &std::path::Path, child_path: &str) -> Result<std::path::PathBuf, std::io::Error> {
    
    let full_path = base_path.join(child_path);

    let canon_base_path = std::fs::canonicalize(base_path)?;

    let canon_full_path = std::fs::canonicalize(full_path)?;

    if !canon_full_path.starts_with(canon_base_path){
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Acesso negado"));
    }
    
    Ok(canon_full_path)
}

/*async fn hello() -> &'static str {
    "Hello from server!"
}*/


async fn read_file(
    _token: auth::Claims,
    Path(file_path) : Path<String>, 
    http_request: Request) -> Result<Response, StatusCode> {

    // TODO : MOVER "base_path" PARA UMA VARIAVEL DE AMBIENTE.
    let base_path = std::env::current_dir().unwrap();

    match validate_path(&base_path, &file_path) {
        Ok(file_found) => {
            let service = tower_http::services::ServeFile::new(file_found);
            let answer = service.oneshot(http_request).await.unwrap();

            Ok(answer.into_response())
        },
        Err(_err) => Err(StatusCode::NOT_FOUND),
        
    }
}

async fn login(Json(payload): Json<LoginRequest>) -> Result<Json<LoginResponse>, StatusCode> {
    
    // CÓDIGO DE TESTE!!
    // TODO : IMPLEMENTAR A LEITURA DOS DADOS DIRETO DO BANCO DE DADOS
    let db_id = "nauzoo";
    let db_role = "admin";

    let salt = SaltString::generate(&mut OsRng);


    let db_hash = Argon2::default()
    .hash_password(b"senha_super_segura", &salt)
    .expect("Erro ao gerar o hash")
    .to_string();

    let parsed_hash = PasswordHash::new(&db_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verificamos se a senha digitada (payload.senha) gera o mesmo hash do banco
    let is_valid = Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !is_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let generated_token = generate_token(db_id, &payload.device, db_role);

    Ok(Json(auth::LoginResponse {
        token: generated_token,
    }))
}

#[tokio::main]
async fn main() {

    let app = Router::new()
    .route("/files/{*path}", get(read_file))
    .route("/login", post(login));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

}
