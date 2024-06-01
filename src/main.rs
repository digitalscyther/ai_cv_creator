mod openai;
mod user;
mod db;
mod ask;
mod dialogue;
mod message;


use std::{env};
use std::time::Duration;
use async_openai::error::OpenAIError;
use axum::error_handling::HandleErrorLayer;
use axum::{BoxError, Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing::{info};
use crate::ask::Asker;
use crate::db::create_pool;
use crate::dialogue::Dialogue;


async fn get_answer(pool: &Pool<Postgres>, user: user::User, message: UserMessage) -> Result<String, &'static str> {
    let default_api_key = env::var("OPENAI_API_KEY").expect("foo");
    let default_max_tokens = Some(300);
    let a = match message.open_ai {
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

    let mut dialogue = Dialogue::new(user, a, message.max_history, message.max_tokens);

    let mut response = dialogue.process_message(Some(message.text.trim())).await;

    while response.is_none() {
        response = dialogue.process_message(match response {
            Some(ref t) => Some(&t),
            _ => None
        }).await;
    }

    dialogue.save_user(pool).await;

    Ok(response.unwrap())
}

#[tokio::main]
async fn main() -> Result<(), OpenAIError> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    info!("Started...");

    let pool = create_pool().await;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await.expect("failed migrations");

    let app = Router::new()
        .route("/users", post(user_create))
        .route("/users/:id", get(user_get))
        .route("/users/:id/message", post(user_message))
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
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(pool);

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

async fn user_create(State(pool): State<Pool<Postgres>>) -> impl IntoResponse {
    let u = user::User::create_user(&pool).await.expect("todo");

    let user = User { id: u.id };

    (StatusCode::CREATED, Json(user))
}

#[derive(Debug, Serialize, Clone)]
struct User {
    id: u64,
}

async fn user_get(Path(id): Path<i32>, State(pool): State<Pool<Postgres>>) -> impl IntoResponse {
    let u = user::User::get_user(&pool, id).await;

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

async fn user_message(Path(id): Path<i32>, State(pool): State<Pool<Postgres>>, Json(message): Json<UserMessage>) -> impl IntoResponse {
    if let Ok(Some(u)) = user::User::get_user(&pool, id).await {
        if let Ok(answer) = get_answer(&pool, u, message).await {
            return Ok(Json(answer));
        }
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}