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

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, ExprTrait, JoinType,
    ModelTrait, QueryFilter, QuerySelect, RelationTrait, Set,
};

mod entities;

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
    pub id: i64,
    pub name: String,
    pub path: String,
}

async fn get_files_batch(
    State(state): State<AppState>,
    _token: Claims,
    Query(query): Query<PagingQuerry>,
) -> Result<Json<Vec<DataInfo>>, StatusCode> {
    let page = query.page.unwrap_or(1);
    let amount = query.amount.unwrap_or(5);

    let offset = (page.saturating_sub(1) * amount) as u64;

    let files = entities::files::Entity::find()
        .offset(offset)
        .limit(amount as u64)
        .all(&state.db)
        .await
        .map_err(|erro| {
            eprintln!("Error querying files from database: {}", erro);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let files_list: Vec<DataInfo> = files
        .into_iter()
        .map(|file| DataInfo {
            id: file.id,
            name: file.name,
            path: file.path,
        })
        .collect();

    Ok(Json(files_list))
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
    Path(file_id): Path<i64>,
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

#[derive(Deserialize)]
pub struct TagRequest {
    pub tag_name: String,
}

async fn create_tag(
    State(state): State<AppState>,
    _token: Claims,
    Json(payload): Json<TagRequest>,
) -> Result<Json<j_val>, StatusCode> {
    let new_tag = entities::tags::ActiveModel {
        name: Set(payload.tag_name),
        ..Default::default()
    };

    new_tag
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "message" : "Tag successfully created"
    })))
}

#[derive(Deserialize)]
pub struct AttachTagRequest {
    pub file_id: i64,
    pub tag_id: i64,
}

async fn attach_tag_to_file(
    State(state): State<AppState>,
    _token: Claims,
    Json(payload): Json<AttachTagRequest>,
) -> Result<Json<j_val>, StatusCode> {
    // 1. (Opcional) Verificar se o arquivo e a tag existem
    let file_exists = entities::files::Entity::find_by_id(payload.file_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();

    let tag_exists = entities::tags::Entity::find_by_id(payload.tag_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();

    if !file_exists || !tag_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    // 2. Cria a relação inserindo na tabela intermediária
    let new_relation = entities::file_tags::ActiveModel {
        file_id: Set(payload.file_id),
        tag_id: Set(payload.tag_id),
    };

    new_relation.insert(&state.db).await.map_err(|erro| {
        eprintln!("Error linking tag to file: {}", erro);
        // Se já existir a chave primária composta duplicada, o SQLite retorna erro de constraint
        StatusCode::BAD_REQUEST
    })?;

    Ok(Json(json!({
        "message": "Tag successfully linked to file",
        "file_id": payload.file_id,
        "tag_id": payload.tag_id
    })))
}

async fn detach_tag_from_file(
    State(state): State<AppState>,
    _token: Claims,
    Json(payload): Json<AttachTagRequest>,
) -> Result<Json<j_val>, StatusCode> {
    let relation = entities::file_tags::Entity::find_by_id((payload.file_id, payload.tag_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    relation.delete(&state.db).await.map_err(|erro| {
        eprintln!("Error detaching tag from file: {}", erro);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(json!({
        "message": "Tag successfully detached from file",
        "file_id": payload.file_id,
        "tag_id": payload.tag_id
    })))
}

#[derive(Deserialize)]
pub struct SearchByNameRequest {
    pub file_name: String,
}

async fn search_by_filename(
    State(state): State<AppState>,
    _token: Claims,
    Json(payload): Json<SearchByNameRequest>,
) -> Result<Json<j_val>, StatusCode> {
    let files = entities::files::Entity::find()
        .filter(entities::files::Column::Name.contains(&payload.file_name))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let files_list: Vec<DataInfo> = files
        .into_iter()
        .map(|file| DataInfo {
            id: file.id,
            name: file.name,
            path: file.path,
        })
        .collect();

    Ok(Json(json!(files_list)))
}

#[derive(Deserialize)]
pub struct SearchByTagRequest {
    pub tag_ids: Vec<i64>,
}

async fn search_by_tags(
    State(state): State<AppState>,
    _token: Claims,
    Json(payload): Json<SearchByTagRequest>,
) -> Result<Json<j_val>, StatusCode> {
    if payload.tag_ids.is_empty() {
        return Ok(Json(json!([])));
    }

    let required_tag_count = payload.tag_ids.len() as i64;

    let files = entities::files::Entity::find()
        .join(
            JoinType::InnerJoin,
            entities::files::Relation::FileTags.def(),
        )
        .filter(entities::file_tags::Column::TagId.is_in(payload.tag_ids))
        .group_by(entities::files::Column::Id)
        .having(
            sea_orm::sea_query::Expr::col(entities::file_tags::Column::TagId)
                .count()
                .eq(required_tag_count),
        )
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let files_list: Vec<DataInfo> = files
        .into_iter()
        .map(|file| DataInfo {
            id: file.id,
            name: file.name,
            path: file.path,
        })
        .collect();

    Ok(Json(json!(files_list)))
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
        .route("/api/login", post(login))
        .route("/api/register", post(register))
        .route("/api/files/{*path}", get(read_file))
        .route("/api/upload", post(upload_file))
        .route("/api/delete/{id}", delete(delete_file))
        .route("/api/tag/create", post(create_tag))
        .route("/api/tag/attach", post(attach_tag_to_file))
        .route("/api/tag/detach", post(detach_tag_from_file))
        .route("/api/search/filename", post(search_by_filename))
        .route("/api/search/tag", post(search_by_tags))
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
