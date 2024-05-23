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
struct Question {
    index: u8,
    question: String,
    answer: Option<String>,
}

impl Question {
    fn new(index: u8, question: &str) -> Self {
        Question {
            index,
            question: question.to_string(),
            answer: None,
        }
    }

    fn set_answer(&mut self, answer: &str) {
        self.answer = Some(answer.to_string());
    }
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug, Default)]
pub struct User {
    pub id: u64,
    profession: Option<String>,
    questions: Option<Vec<Question>>,
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

        if let Some(questions) = &self.questions {
            return match questions.iter().all(|q| q.answer.is_some()) {
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

    pub fn set_questions(&mut self, questions: Vec<&str>) {
        self.questions =
            Some(
                questions
                    .iter()
                    .enumerate()
                    .map(|(ind, q)| Question::new(ind as u8, q))
                    .collect()
            );
    }

    pub fn set_answer(&mut self, ind: u8, answer: &str) -> Option<&'static str> {
        if let Some(ref mut questions) = &mut self.questions {
            if let Some(mut q) = questions.get_mut(ind as usize) {
                q.set_answer(answer);
                return None;
            }

            return Some("invalid question index");
        };

        return Some("no questions");
    }

    pub fn set_result(&mut self, result: &str) {
        self.result = Some(result.to_string());
    }

    pub fn reset(&mut self) {
        *self = User::new(self.id);
    }
}