mod openai;
mod user;
mod db;
mod ask;
mod dialogue;


use std::env;
use serde_json::json;
use std::error::Error;
use async_openai::error::OpenAIError;
use async_openai::types::{ChatCompletionResponseMessage, Role};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing::{info};
use crate::ask::Asker;
use crate::dialogue::Dialogue;
use crate::openai::{get_response, Request};
use crate::user::User;


async fn user_test(){
    let mut u = User::get_user(23).await.expect("failed get user");
    info!("{:?}", &u.need());

    u.set_profession("accountant");
    info!("{:?}", u.need());

    u.set_questions(vec!["foo", "bar", "baz"].iter().map(|s| s.to_string()).collect());
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

async fn dialogue_test() {
    let mut u = User::get_user(50).await.expect("failed get user");
    let a = Asker::new(
        env::var("OPENAI_API_KEY").expect("foo"),
        Some(300),
        None,
        None
    );

    let mut dialogue = Dialogue::new(u, a);

    let response = dialogue.process_message(Some("hello".to_string())).await.expect("foo");

    info!("{:?}", response);
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

    dialogue_test().await;

    Ok(())
}