use axum::{
    Json, Router, extract::{Path, Request, State}, http::StatusCode, response::{IntoResponse, Response }, routing::{get, post}
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use tower::ServiceExt;
mod auth;
use crate::auth::{Claims, LoginRequest, LoginResponse, RegisterRequest, generate_token};
mod entities;

use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter};
use sea_orm::{ActiveModelTrait, Set};

#[derive(Clone)] // O Estado PRECISA ser clonável para o Axum
pub struct AppState {
    pub db: DatabaseConnection,
}

use sea_orm::{ConnectionTrait, Schema, DbBackend};
// Importe sua entidade aqui (assumindo que está no entities.rs) 
use crate::entities::users;

async fn inicializar_banco(db: &DatabaseConnection) {
    let schema = Schema::new(DbBackend::Sqlite);

    // 1. Criamos o molde da tabela diretamente da sua Entity
    let mut stmt = schema.create_table_from_entity(users::Entity);
    stmt.if_not_exists();

    // 2. Entregamos a instrução direto para o banco executar
    db.execute(&stmt)
        .await
        .expect("Falha ao criar a tabela de usuários");
    
    println!("✅ Banco de dados e tabelas verificados com sucesso!");
}

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
    _token: Claims,
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

async fn login(
    State(state): State<AppState>, 
    Json(payload): Json<LoginRequest>)
     -> Result<Json<LoginResponse>, StatusCode> {
    
    let user_found = users::Entity::find().filter(users::Column::Name.eq(&payload.username))
    .one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?; 


    let db_hash = user_found.password;

    let parsed_hash = PasswordHash::new(&db_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verificamos se a senha digitada (payload.senha) gera o mesmo hash do banco
    let is_valid = Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !is_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let generated_token = generate_token(&user_found.id.to_string(), &user_found.role);

    Ok(Json(auth::LoginResponse {
        token: generated_token,
    }))
}

async fn register(
    State(state): State<AppState>, 
    Json(payload): Json<RegisterRequest>)
    -> Result<StatusCode, StatusCode> {

        // 1. Criptografia: Geramos um salt aleatório e criamos o hash da senha
    
    let salt = SaltString::generate(&mut OsRng);
    
    let senha_hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    // 2. Preparando os dados para o SeaORM
    // Usamos ActiveModel quando queremos inserir ou atualizar dados.
    // O `Set()` avisa ao SeaORM quais campos estamos alterando.
    let novo_usuario = users::ActiveModel {
        name: Set(payload.username),
        password: Set(senha_hash),
        role: Set(payload.role),
        ..Default::default() // Ignora o `id` para que o SQLite gere automaticamente (auto-increment)
    };
    
    // 3. Salvando no banco de dados
    match novo_usuario.insert(&state.db).await {
        Ok(_) => Ok(StatusCode::CREATED), // Retorna 201 Created se der certo
        Err(_) => Err(StatusCode::CONFLICT), // Retorna 409 Conflict se o email já existir
    }
    

}

#[tokio::main]
async fn main() {

    let db_url = "sqlite://server_data.db?mode=rwc";

    let db = Database::connect(db_url)
        .await
        .expect("Não foi possível conectar ao banco SQLite");

    inicializar_banco(&db).await;

    let state = AppState { db };

    let app = Router::new()
    .route("/files/{*path}", get(read_file))
    .route("/login", post(login))
    .route("/register", post(register))
    .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

}
