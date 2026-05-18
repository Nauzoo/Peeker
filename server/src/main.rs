use axum::{
    Json, Router, 
    extract::{Path, Request, State}, 
    http::StatusCode, 
    response::{IntoResponse, Response }, routing::{get, post}
};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2
};

use tower_cookies::{CookieManagerLayer};

use tower::ServiceExt;

mod auth;
use crate::auth::auth::{Claims, RegisterRequest, AppState, login};

use sea_orm::{Database, DatabaseConnection, ActiveModelTrait, Set, ConnectionTrait, Schema, DbBackend};

mod entities;
use crate::entities::users;


async fn initialize_database(db: &DatabaseConnection) {
    /* Creates a new database, if it doesn't exist, and returns its connection. */
    
    let schema = Schema::new(DbBackend::Sqlite); // Choosing the DB backend
   
    let mut stmt = schema.create_table_from_entity(users::Entity); // Uses user Model to build the table
    stmt.if_not_exists();

    db.execute(&stmt)
        .await
        .expect("! Failed to create users entity to the database.");
    
    println!("✅ Database and entities verified successfully!");
}

fn validate_path(base_path: &std::path::Path, child_path: &str) -> Result<std::path::PathBuf, std::io::Error> {
    /* Sanatization function, checks whether the accessed path is part of the base path, preventing path traversal. */

    let full_path = base_path.join(child_path);

    let canon_base_path = std::fs::canonicalize(base_path)?;

    let canon_full_path = std::fs::canonicalize(full_path)?;

    if !canon_full_path.starts_with(canon_base_path){
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied!"));
    }
    
    Ok(canon_full_path)
}

async fn read_file(
    _token: Claims,
    Path(file_path) : Path<String>, 
    http_request: Request) -> Result<Response, StatusCode> {
    /* returns a file stream from a path.*/

    // TODO : move base_path" to a env. variable.
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

async fn register(
    State(state): State<AppState>, 
    Json(payload): Json<RegisterRequest>)
    -> Result<StatusCode, StatusCode> {
    
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

    initialize_database(&db).await;

    let state = AppState { db };

    let app = Router::new()
    .route("/files/{*path}", get(read_file))
    .route("/login", post(login))
    .route("/register", post(register))
    .layer(CookieManagerLayer::new())
    .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

}
