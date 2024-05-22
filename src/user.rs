use derivative::Derivative;
use serde::{Deserialize, Serialize};
use crate::db;

#[derive(Debug)]
pub enum Need {
    Profession,
    Questions,
    Answers,
    Result,
    None,
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug, Default)]
struct Answer {
    index: u8,
    question: String,
    answer: Option<String>,
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug, Default)]
pub struct User {
    pub id: u64,
    profession: Option<String>,
    answers: Option<Vec<Answer>>,
    result: Option<String>,
}

impl User {
    pub async fn get_user(id: u64) -> Result<User, &'static str> {
        match db::load_user(id).await {
            Some(user) => Ok(user),
            None => {
                Ok(User::new(id))
            }
        }
    }

    fn new(id: u64) -> Self {
        let mut u = User::default();
        u.id = id;

        u
    }

    pub fn need(&self) -> Need {
        if self.result.is_some() {
            return Need::None;
        }

        if let Some(answers) = &self.answers {
            return match answers.iter().all(|a| a.answer.is_some()) {
                true => Need::Result,
                false => Need::Answers,
            };
        }

        if self.profession.is_some() {
            return Need::Questions;
        }

        Need::Profession
    }

    pub async fn save(&self) -> Result<(), &'static str> {
        db::save_user(&self).await.map_err(|e| e)?;
        Ok(())
    }

    pub fn set_profession(&mut self, profession: &str) {
        self.profession = Some(profession.to_string());
    }
}