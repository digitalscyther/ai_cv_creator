use std::env;
use sqlx::{Postgres, Pool, Error};
use sqlx::postgres::PgPoolOptions;

use crate::user::UserWithCustomMessages;

pub async fn create_pool() -> Pool<Postgres> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}

pub async fn load_user(pool: &Pool<Postgres>, id: i32) -> Result<Option<UserWithCustomMessages>, Error> {
    let query = sqlx::query_as!(
        UserWithCustomMessages,
        r#"
        SELECT id, profession, questions, resume, messages, tokens_spent
        FROM users
        WHERE id = $1
        "#,
        id
    )
        .fetch_one(pool)
        .await;

    match query {
        Ok(user) => Ok(Some(user)),
        Err(_) => Ok(None)
    }
}

pub async fn save_user(pool: &Pool<Postgres>, user: UserWithCustomMessages) -> Result<(), &'static str> {
    let query = sqlx::query!(
        r#"
        INSERT INTO users (id, profession, questions, resume, messages, tokens_spent)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO UPDATE
        SET profession = EXCLUDED.profession,
            questions = EXCLUDED.questions,
            resume = EXCLUDED.resume,
            messages = EXCLUDED.messages,
            tokens_spent = EXCLUDED.tokens_spent
        "#,
        user.id,
        user.profession,
        user.questions,
        user.resume,
        user.messages,
        user.tokens_spent,
    )
        .execute(pool)
        .await;

    match query {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to save user data"),
    }
}

pub async fn new_user(pool: &Pool<Postgres>) -> Result<u64, &'static str> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO users ( tokens_spent )
        VALUES ( $1 )
        RETURNING id
        "#,
        0
    )
        .fetch_one(pool)
        .await;

    match rec {
        Ok(rec) => Ok(rec.id as u64),
        Err(_) => Err("Failed to create new user"),
    }
}