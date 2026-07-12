use axum::{
    Json, Router, extract::{Path, Query, Request}, http::StatusCode, response::{IntoResponse, Response }, routing::{get, post}
};

use serde::{ Serialize, Deserialize };
use tower_http::{services::{ServeDir, ServeFile}};

use tower_cookies::{CookieManagerLayer};

use tower::ServiceExt;

mod auth;
use crate::auth::auth::{Claims, AppState, login, register};

use sea_orm::{Database, DatabaseConnection};

mod entities;

use std::fs;

use migration::{Migrator, MigratorTrait};

async fn initialize_database(db: &DatabaseConnection) {
    /* 
       Aplica todas as migrations pendentes.
       Se o banco for novo, roda tudo. Se já estiver atualizado, passa direto.
    */
    Migrator::up(db, None)
        .await
        .expect("! Failed to run database migrations.");
    
    println!("✅ Database migrations applied successfully!");
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

#[derive(Deserialize)]
pub struct PagingQuerry {
    pub page : Option<usize>, // Option<usize> handles nullable values safelly
    pub amount : Option<usize>
}

#[derive(Serialize)]
pub struct DataInfo {
    pub id : String,
    pub name: String
}

async  fn get_files_batch(
    _token: Claims,
    Query(query): Query<PagingQuerry>,
) -> Json<Vec<DataInfo>> {

    let page = query.page.unwrap_or(1);
    let amount = query.amount.unwrap_or(5);

    let base_path = std::env::current_dir().unwrap().join("test_files");

    let mut files_list = Vec::new();

    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            if let Ok(type_) = entry.file_type() {
                if type_.is_file() {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                files_list.push(
                    DataInfo {
                        id: file_name.clone(),
                        name: file_name
                    }
                );
               } 
            } 
        }
    }

    files_list.sort_by(|a, b| a.name.cmp(&b.name));

    let start = (page.saturating_sub(1)) * amount;

    let batch: Vec<DataInfo> = files_list
        .into_iter()
        .skip(start)
        .take(amount)
        .collect();

    // Retorna a lista como JSON
    Json(batch)
    
}

async fn get_login_status(
    _token :Claims,
) -> StatusCode {
    StatusCode::OK
} 

#[tokio::main]
async fn main() {

    let db_url = "sqlite://server_data.db?mode=rwc";

    let db = Database::connect(db_url)
        .await
        .expect("Não foi possível conectar ao banco SQLite");

    initialize_database(&db).await;

    let state = AppState { db };

    let public_path = concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/dist");

    let app = Router::new()

        .route("/api/auth/", get(get_login_status))
        .route("/api/files", get(get_files_batch))

        .route("/api/files/{*path}", get(read_file))
        
        .route(
            "/api/login", 
            //get_service(ServeFile::new(format!("{public_path}/login.html")))
            post(login)
        )
        
        .route(
            "/api/register",
            //get_service(ServeFile::new(format!("{public_path}/register.html")))
            post(register))
        
        .layer(CookieManagerLayer::new())
        .fallback_service(ServeDir::new(public_path)
        .not_found_service(ServeFile::new(format!("{public_path}/index.html"))))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

}
