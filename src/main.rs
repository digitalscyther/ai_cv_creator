mod openai;
mod user;
mod db;
mod ask;


use std::env;
use serde_json::json;
use std::error::Error;
use async_openai::error::OpenAIError;
use async_openai::types::{ChatCompletionResponseMessage, Role};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing::{info};
use crate::ask::Asker;
use crate::openai::{get_response, Request};
use crate::user::User;


async fn void() -> Result<ChatCompletionResponseMessage, OpenAIError> {
    let api_key = env::var("OPENAI_API_KEY").expect("failed get openai_api_key");

    let raw_functions = vec![
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
        ];

    let req: Request = Request::new(
        api_key.to_string(),
        vec![
            (Role::System, "Before answering ask user name"),
            (Role::User, "What's the weather like in Boston?")
        ],
        None,
        None,
        raw_functions,
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


async fn get_some() {
    let asker = Asker::new(
        env::var("OPENAI_API_KEY").expect("failed get openai_api_key"),
        None,
        None
    );

    // let profession = asker.get_profession(
    //     vec![
    //         (Role::System, "find out from the user what profession is interesting to him and save it"),
    //         (Role::User, "I'm interested in the profession of Rust developer"),
    //     ]
    // ).await;
    // info!("{:?}", profession);

    // let questions = asker.get_questions(
    //     vec![
    //         (Role::System, "Generate questions, the answers to which are needed to create a resume for Rust developer. Save them."),
    //     ]
    // ).await;
    // info!("{:?}", questions);

    let answers = asker.get_answers(
        vec![
            (Role::System, "You need to fill out a form asking the user questions. Form(+ - mean answered question):\
            [0] Full Name [ ]\
            [1] Age [+] 18\
            [2] Sex [ ]\
            \
            Ask questions and fill form"),
            (Role::User, "Hello"),
            (Role::Assistant, "Hi! I'm here to help you with filling out a form. Let's get started. What is your full name?"),
            (Role::User, "Alex Black"),
            (Role::Assistant, "Great! Thank you, Alex. How old are you?"),
            (Role::User, "25"),
        ]
    ).await;
    info!("{:?}", answers);
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

    // user_test().await;

    get_some().await;

    Ok(())
}