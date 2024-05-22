mod openai;
mod user;
mod db;


use std::env;
use serde_json::json;
use std::error::Error;
use async_openai::error::OpenAIError;
use async_openai::types::{ChatCompletionResponseMessage, Role};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing::{info};
use crate::openai::{Functions, get_response, Request};
use crate::user::User;


async fn void() -> Result<ChatCompletionResponseMessage, OpenAIError> {
    let api_key = env::var("OPENAI_API_KEY").expect("failed get openai_api_key");

    let functions = Functions::new(
        vec![
            ("get_current_weather", "Get the current weather in a given location", json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA",
                    },
                    "unit": { "type": "string", "enum": ["celsius", "fahrenheit"] },
                },
                "required": ["location"],
            }))
        ],
        None,
    );

    let req: Request = Request::new(
        api_key.to_string(),
        vec![
            (Role::System, "Before answering ask user name"),
            (Role::User, "What's the weather like in Boston?")
        ],
        None,
        None,
        Some(functions),
        // None,
    );

    return get_response(req).await
}


#[tokio::main]
async fn main() -> Result<(), OpenAIError> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    info!("hi");

    // let response_message = void().await;
    // info!("{:?}", response_message);

    let mut u = User::get_user(23).await.expect("failed get user");
    u.set_profession("accountant");
    u.save().await.expect("failed save user");
    info!("{:?}, {:?}", &u.need(), u);

    Ok(())
}