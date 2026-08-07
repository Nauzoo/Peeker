use axum::{
    Json, Router,
    extract::{Multipart, Path, Query, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};

use serde::{Deserialize, Serialize};
use tower_http::services::{ServeDir, ServeFile};

use tower_cookies::CookieManagerLayer;

use tower::ServiceExt;

mod auth;
use crate::auth::auth::{AppState, Claims, get_login_status, login, register};

use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, ModelTrait, Set};

mod entities;

use std::fs;
use tokio::fs::File as tokio_file;
use tokio::io::AsyncWriteExt;

use migration::{
    Migrator, MigratorTrait, prelude::serde_json::Value as j_val, prelude::serde_json::json,
};

use uuid::Uuid;

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

fn validate_path(
    base_path: &std::path::Path,
    child_path: &str,
) -> Result<std::path::PathBuf, std::io::Error> {
    /* Sanatization function, checks whether the accessed path is part of the base path, preventing path traversal. */

    let full_path = base_path.join(child_path);

    let canon_base_path = std::fs::canonicalize(base_path)?;

    let canon_full_path = std::fs::canonicalize(full_path)?;

    if !canon_full_path.starts_with(canon_base_path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Access denied!",
        ));
    }

    Ok(canon_full_path)
}

async fn read_file(
    _token: Claims,
    Path(file_path): Path<String>,
    http_request: Request,
) -> Result<Response, StatusCode> {
    /* returns a file stream from a path.*/

    // TODO : move base_path" to a env. variable.
    let base_path = std::env::current_dir().unwrap();

    match validate_path(&base_path, &file_path) {
        Ok(file_found) => {
            let service = tower_http::services::ServeFile::new(file_found);
            let answer = service.oneshot(http_request).await.unwrap();

            Ok(answer.into_response())
        }
        Err(_err) => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Deserialize)]
pub struct PagingQuerry {
    pub page: Option<usize>, // Option<usize> handles nullable values safelly
    pub amount: Option<usize>,
}

#[derive(Serialize)]
pub struct DataInfo {
    pub id: String,
    pub name: String,
}

async fn get_files_batch(_token: Claims, Query(query): Query<PagingQuerry>) -> Json<Vec<DataInfo>> {
    let page = query.page.unwrap_or(1);
    let amount = query.amount.unwrap_or(5);

    let base_path = std::env::current_dir().unwrap().join("test_files");

    let mut files_list = Vec::new();

    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            if let Ok(type_) = entry.file_type() {
                if type_.is_file() {
                    let file_name = entry.file_name().to_string_lossy().into_owned();
                    files_list.push(DataInfo {
                        id: file_name.clone(),
                        name: file_name,
                    });
                }
            }
        }
    }

    files_list.sort_by(|a, b| a.name.cmp(&b.name));

    let start = (page.saturating_sub(1)) * amount;

    let batch: Vec<DataInfo> = files_list.into_iter().skip(start).take(amount).collect();

    // Retorna a lista como JSON
    Json(batch)
}

pub async fn upload_file(
    State(state): State<AppState>,
    token: Claims,
    mut multipart: Multipart,
) -> Result<Json<j_val>, StatusCode> {
    let uploads_dir = "./test_files";

    // CORREÇÃO 1: create_dir_all não falha se a pasta já existir
    tokio::fs::create_dir_all(uploads_dir)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        // CORREÇÃO 3: file_name() pega o nome real do arquivo (ex: video.mp4)
        let original_name = field.file_name().unwrap_or("unnamed_file").to_string();

        if original_name.is_empty() {
            continue;
        }

        let unique_id = Uuid::new_v4().to_string();
        let save_path = format!("{}/{}", uploads_dir, unique_id);

        let mut file = tokio_file::create(&save_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            file.write_all(&chunk)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        let author_id = token.sub.parse().unwrap_or(0);

        let new_file = entities::files::ActiveModel {
            name: Set(original_name.clone()),
            path: Set(save_path),
            creator: Set(author_id.to_string()),
            ..Default::default()
        };

        new_file
            .insert(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(Json(json!({
            "message": "Upload OK",
            "file_name": original_name,
            "id": unique_id
        })));
    }

    Err(StatusCode::BAD_REQUEST)
}

async fn delete_file(
    State(state): State<AppState>,
    token: Claims,
    Path(file_id): Path<i32>,
) -> Result<Json<j_val>, StatusCode> {
    if token.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    let file_record = entities::files::Entity::find_by_id(file_id)
        .one(&state.db)
        .await
        .map_err(|erro| {
            eprintln!("Error while searching file {}", erro);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let path_to_delete = file_record.path.clone();

    file_record.delete(&state.db).await.map_err(|erro| {
        eprintln!("Error while deleting file {}", erro);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tokio::fs::remove_file(&path_to_delete)
        .await
        .map_err(|erro| {
            eprintln!("Error while deleting physical media {}", erro);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(json!({
        "message" : "File successfully deleted",
        "id": file_id
    })))
}

#[tokio::main]
async fn main() {
    let db_url = "sqlite://server_data.db?mode=rwc"; // TODO : mover para env

    let db = Database::connect(db_url)
        .await
        .expect("Não foi possível conectar ao banco SQLite");

    initialize_database(&db).await;

    let state = AppState { db };

    let public_path = concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/dist");

    let app = Router::new()
        .route("/api/auth/", get(get_login_status))
        .route("/api/files", get(get_files_batch))
        .route(
            "/api/login",
            //get_service(ServeFile::new(format!("{public_path}/login.html")))
            post(login),
        )
        .route(
            "/api/register",
            //get_service(ServeFile::new(format!("{public_path}/register.html")))
            post(register),
        )
        .route("/api/files/{*path}", get(read_file))
        .route("/api/upload", post(upload_file))
        .route("/api/files/{*path}", delete(delete_file))
        .layer(CookieManagerLayer::new())
        .fallback_service(
            ServeDir::new(public_path)
                .not_found_service(ServeFile::new(format!("{public_path}/index.html"))),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
