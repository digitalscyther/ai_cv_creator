use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::env;
use teloxide::{prelude::*};
use teloxide::utils::command::BotCommands;
use chrono::{Utc, DateTime};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing::{info};
use uuid::Uuid;


fn get_api_url() -> String {
    env::var("API_URL").expect("API_URL must be set")
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    code: String,
    creator: i64,
    api_user_id: i32,
    created: DateTime<Utc>,
    chat_id: Option<i64>,
    registered: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiMessage {
    text: String,
}

#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    /// display this text.
    Help,
    Start,
    ShowMyInfo,
    #[command(description = "generate an invite link.")]
    GenerateInvite,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiUser {
    id: i32
}

async fn create_user(client: &Client) -> Result<i32, reqwest::Error> {
    let api_url = get_api_url();
    let response = client.post(format!("{api_url}/users")).send().await?;
    let user: ApiUser = response.json().await?;
    Ok(user.id)
}

async fn get_user_info(client: &Client, user_id: i32) -> Result<Value, reqwest::Error> {
    let api_url = get_api_url();
    let response = client.get(format!("{api_url}/users/{}", user_id)).send().await?;
    let data: Value = response.json().await?;
    Ok(data)
}

async fn send_message(client: &Client, user_id: i32, text: &str) -> Result<String, reqwest::Error> {
    let api_url = get_api_url();
    let message = ApiMessage { text: text.to_string() };
    let response = client.post(format!("{api_url}/users/{}/message", user_id))
        .json(&message)
        .send().await?;
    let reply: String = response.json().await?;
    Ok(reply)
}

async fn get_user_id(pool: &Pool<Postgres>, chat_id: i64) -> Result<Option<i32>, &'static str> {
    let api_user_id: Option<i32> = sqlx::query_scalar!("SELECT api_user_id FROM users WHERE chat_id = $1", chat_id)
        .fetch_optional(pool).await.unwrap();
    Ok(api_user_id)
}

async fn handle_message(
    params: ConfigParameters,
    bot: Bot, msg: Message
) -> Result<(), teloxide::RequestError> {
    let chat_id = msg.chat.id;

    let api_user_id = get_user_id(&params.pool, chat_id.0).await.expect("foo");

    let user_id = match api_user_id {
        Some(id) => id,
        None => {
            bot.send_message(
                chat_id,
                "You are not registered. Please contact with an admin to register."
            ).await.unwrap();
            return Ok(());
        }
    };

    let text = msg.text().unwrap();
    let reply = send_message(&params.client, user_id, text).await.unwrap();
    bot.send_message(chat_id, reply).await.unwrap();

    Ok(())
}

async fn handle_invite_link(params: ConfigParameters, bot: Bot, msg: &Message, invite_code: &str) -> Result<(), teloxide::RequestError> {
    let chat_id = msg.chat.id;

    let invite_exists: Option<i32> = sqlx::query_scalar!("SELECT id FROM users WHERE code = $1 AND registered IS NULL", invite_code)
        .fetch_optional(&params.pool).await.unwrap();

    if invite_exists.is_none() {
        return Ok(());
    }

    let id = invite_exists.unwrap();
    let now = Utc::now();

    sqlx::query!("UPDATE users SET chat_id = $1, registered = $2 WHERE id = $3", chat_id.0, now, id)
        .execute(&params.pool).await.unwrap();  // TODO fix unwrap without result stop app

    bot.send_message(chat_id, "You have successfully registered!").await.unwrap();

    Ok(())
}

async fn handle_command(
    params: ConfigParameters,
    bot: Bot, msg: Message,
    command: Command,
) -> Result<(), teloxide::RequestError> {
    match command {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await.expect("foo");
        },
        Command::Start => {
            let code = &msg.text().unwrap()[7..];
            if code.len() > 0 {
                handle_invite_link(params, bot, &msg, code).await.expect("foo");
            }
        },
        Command::ShowMyInfo => {
            match get_user_id(&params.pool, msg.chat.id.0).await.expect("foo") {
                Some(user_id) => {
                    let user_info = get_user_info(&params.client, user_id).await;
                    bot.send_message(
                        msg.chat.id,
                        serde_json::to_string(&user_info.unwrap()).unwrap()
                    ).await.expect("foo");
                },
                None => {
                    bot.send_message(
                        msg.chat.id,
                        "You are not registered. Please contact with an admin to register."
                    ).await.unwrap();
                }
            }
        }
        Command::GenerateInvite => {
            let invite_code = Uuid::new_v4().to_string();
            let api_user_id = create_user(&params.client).await.or_else(|e| Err(format!("Failed create new user:\n{:?}", e))).unwrap();
            let now = Utc::now();

            sqlx::query!(
                "INSERT INTO users (api_user_id, creator, code, created) VALUES ($1, $2, $3, $4)",
                api_user_id,
                msg.chat.id.0,
                invite_code,
                now
            )
                .execute(&params.pool).await.unwrap();

            let bot_name = env::var("BOT_NAME").expect("BOT_NAME must be set");
            let invite_link = format!("https://t.me/{bot_name}?start={}", invite_code);
            bot.send_message(msg.chat.id, format!("Your invite link: {}", invite_link)).await.expect("foo");
        }
    };
    Ok(())
}

#[derive(Clone)]
struct ConfigParameters {
    pool: Pool<Postgres>,
    client: Client,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    info!("Started...");

    let db_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let bot_token: String = env::var("BOT_TOKEN").expect("BOT_TOKEN must be set");

    let bot = Bot::new(bot_token);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url).await.expect("Failed to create pool.");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await.expect("failed migrations");

    let client = Client::new();

    let parameters = ConfigParameters { pool, client };

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handle_command)
        )
        .branch(dptree::endpoint(handle_message));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![parameters])
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
