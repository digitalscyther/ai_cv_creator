mod openai;
mod user;
mod db;
mod ask;
mod dialogue;
mod message;


use std::{env, io};
use serde_json::json;
use std::error::Error;
use async_openai::error::OpenAIError;
use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs, ChatCompletionResponseMessage, Role};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing::{info};
use crate::ask::Asker;
use crate::dialogue::Dialogue;
use crate::message::Message;
use crate::openai::{get_response, Request};
use crate::user::User;


async fn user_test() {
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

    u.set_resume("u r good. go to work.");
    info!("{:?}", u.need());

    u.reset();
    info!("{:?}", u.need());

    // u.save().await.expect("failed save user");
}

async fn dialogue_test() {
    let u = User::get_user(50).await.expect("failed get user");
    let a = Asker::new(
        env::var("OPENAI_API_KEY").expect("foo"),
        Some(300),
        None,
        None,
    );

    let mut dialogue = Dialogue::new(u, a);

    println!("Start...");
    loop {
        println!(">>> ");

        let mut text = String::new();

        io::stdin().read_line(&mut text).expect("Failed to read line");

        let mut response = dialogue.process_message(Some(text.trim())).await;

        while response.is_none() {
            response = dialogue.process_message(match response {
                Some(ref t) => Some(&t),
                _ => None
            }).await;
        }
        dialogue.save_user().await;

        println!("{:?}", response);
    }
}

fn custom_message_test() {
    // Wrap them in the enum
    let messages = vec![
        ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessageArgs::default().build().unwrap()),
        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessageArgs::default().build().unwrap()),
        ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessageArgs::default().build().unwrap()),
        ChatCompletionRequestMessage::Tool(ChatCompletionRequestToolMessageArgs::default().build().unwrap()),
        ChatCompletionRequestMessage::Function(ChatCompletionRequestFunctionMessageArgs::default().build().unwrap()),
    ];

    // Serialize and deserialize each message
    for msg in messages {
        let message_wrapper = Message::from_original(msg);
        let serialized = serde_json::to_string(&message_wrapper).unwrap();
        println!("Serialized: {}", serialized);

        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        println!("Deserialized: {:?}", deserialized);
    }
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

    // custom_message_test();

    Ok(())
}