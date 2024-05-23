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

async fn user_test(){
    let mut u = User::get_user(23).await.expect("failed get user");
    info!("{:?}", &u.need());

    u.set_profession("accountant");
    info!("{:?}", u.need());

    u.set_questions(vec!["foo", "bar", "baz"]);
    info!("{:?}", u.need());

    u.set_answer(0, "foo");
    u.set_answer(1, "boo");
    u.set_answer(2, "boo");
    info!("{:?}", u.need());

    u.set_result("u r good. go to work.");
    info!("{:?}", u.need());

    u.reset();
    info!("{:?}", u.need());

    // u.save().await.expect("failed save user");
}


#[tokio::main]
async fn main() -> Result<(), OpenAIError> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    info!("Started...");

    // let response_message = void().await;
    // info!("{:?}", response_message);

    user_test().await;

    Ok(())
}