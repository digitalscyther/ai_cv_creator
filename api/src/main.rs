mod openai;
mod user;
mod db;
mod ask;
mod dialogue;
mod message;
mod pdf;
mod storage;


use std::{env};
use std::time::Duration;
use async_openai::error::OpenAIError;
use aws_sdk_s3::Client;
use axum::error_handling::HandleErrorLayer;
use axum::{BoxError, Json, Router};
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tempfile::NamedTempFile;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing::{info};
use uuid::Uuid;
use crate::ask::Asker;
use crate::db::create_pool;
use crate::dialogue::{Dialogue, Instruction};
use crate::storage::{create_client, delete, load, save};


enum Answer {
    Message(String),
    Generated,
}

fn get_bucket_name() -> String {
    env::var("MINIO_BUCKET_NAME").expect("MINIO_BUCKET_NAME must be set")
}


async fn get_answer(app_state: AppState, user: user::User, message: UserMessage) -> Result<Answer, &'static str> {
    let default_api_key = env::var("OPENAI_API_KEY").expect("foo");
    let bucket_name = get_bucket_name();

    let default_max_tokens = Some(1000);
    let asker = match message.open_ai {
        Some(open_ai) => Asker::new(
            open_ai.api_key.unwrap_or(default_api_key),
            open_ai.max_tokens.or(default_max_tokens),
            open_ai.model,
            None,
        ),
        None => Asker::new(
            default_api_key,
            default_max_tokens,
            None,
            None,
        )
    };

    let mut dialogue = Dialogue::new(user, asker, message.max_history, message.max_tokens);

    let text = message.text.trim();

    if text.len() > dialogue.get_max_message_length() {
        return Ok(Answer::Message("Invalid message (to long)".to_string()))
    }

    let (mut response, mut instruction) = dialogue.process_message(Some(text)).await;

    while response.is_none() {
        (response, instruction) = dialogue.process_message(match response {
            Some(ref t) => Some(&t),
            _ => None
        }).await;
    }

    match instruction {
        Instruction::SaveResume => {
            let resume_temp = NamedTempFile::new().unwrap();
            pdf::generate_pdf(&response.clone().unwrap(), &resume_temp).await.expect("Failed generate pdf");

            let resume_temp_filepath = resume_temp.path().to_str().unwrap().to_string();
            let resume_name = format!("{}.pdf", Uuid::new_v4().to_string());
            save(&app_state.s3_client, &bucket_name, &resume_temp_filepath, &resume_name).await.expect("failed save_s3");
            dialogue.set_resume(&resume_name).await.expect("Failed set resume for user");
            dialogue.save_user(&app_state.pool).await;
            return Ok(Answer::Generated)
        }
        Instruction::DeleteResume(name) => {
            delete(&app_state.s3_client, &bucket_name, &name).await.expect("failed delete_s3");
        }
        Instruction::None => { }
    }

    dialogue.save_user(&app_state.pool).await;

    Ok(Answer::Message(response.unwrap()))
}

#[derive(Derivative, Debug)]
#[derivative(Clone)]
struct AppState {
    pool: Pool<Postgres>,
    s3_client: Client,
}

#[tokio::main]
async fn main() -> Result<(), OpenAIError> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    info!("Started...");

    let s3_client = create_client().await.expect("Failed create s3 client");

    let pool = create_pool().await;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await.expect("failed migrations");

    let app_state = AppState { pool, s3_client };

    let app = Router::new()
        .route("/users", post(user_create))
        .route("/users/:id", get(user_get))
        .route("/users/:id/message", post(user_message))
        .route("/users/:id/cv", get(user_cv))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(60))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(app_state);

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let bind_address = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn user_create(State(app_state): State<AppState>) -> impl IntoResponse {
    let u = user::User::create_user(&app_state.pool).await.expect("todo");

    let user = User { id: u.id };

    (StatusCode::CREATED, Json(user))
}

#[derive(Debug, Serialize, Clone)]
struct User {
    id: u64,
}

async fn user_get(Path(id): Path<i32>, State(app_state): State<AppState>) -> impl IntoResponse {
    let u = user::User::get_user(&app_state.pool, id).await;

    match u {
        Ok(Some(u)) => Ok(Json(u)),
        _ => Err(StatusCode::NOT_FOUND)
    }
}

#[derive(Debug, Deserialize)]
struct OpenAI {
    api_key: Option<String>,
    max_tokens: Option<u16>,
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserMessage {
    text: String,
    open_ai: Option<OpenAI>,
    max_history: Option<usize>,
    max_tokens: Option<u32>,
}

async fn user_message(Path(id): Path<i32>, State(app_state): State<AppState>, Json(message): Json<UserMessage>) -> impl IntoResponse {
    if let Ok(Some(user)) = user::User::get_user(&app_state.pool, id).await {
        return match get_answer(app_state, user, message).await {
            Ok(Answer::Message(text)) => Ok(Json(text)),
            Ok(Answer::Generated) => Ok(Json("generated".to_string())),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
    Err(StatusCode::NOT_FOUND)
}

async fn user_cv(Path(id): Path<i32>, State(app_state): State<AppState>) -> impl IntoResponse {
    if let Ok(Some(u)) = user::User::get_user(&app_state.pool, id).await {
        return match u.get_resume() {
            Some(name) => {
                let bucket_name = get_bucket_name();
                let bytes = load(&app_state.s3_client, &bucket_name, &name).await.expect("Failed s3_load");

                let headers = [
                    (header::CONTENT_TYPE, "application/pdf; charset=utf-8"),
                    (header::CONTENT_DISPOSITION, "attachment; filename=\"cv.pdf\""),
                ];

                Ok((headers, bytes))
            },
            None => Err(StatusCode::BAD_REQUEST)
        }
    }
    Err(StatusCode::NOT_FOUND)
}