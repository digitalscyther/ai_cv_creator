use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageContent};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use crate::db;
use crate::message::Message;

#[derive(Debug)]
pub enum Need {
    Profession,
    Questions,
    Answers,
    Resume,
    None,
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug, Clone, Default)]
struct Question {
    index: u8,
    question: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
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
    resume: Option<String>,
    messages: Vec<ChatCompletionRequestMessage>,
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug, Default)]
pub struct UserWithCustomMessages {
    pub id: u64,
    profession: Option<String>,
    questions: Option<Vec<Question>>,
    resume: Option<String>,
    messages: Vec<Message>,
}

impl UserWithCustomMessages {
    pub fn from_original(user: &User) -> Self {
        let messages = user.messages.iter().map(|msg| Message::from_original(msg.clone())).collect();

        UserWithCustomMessages {
            id: user.id,
            profession: user.profession.clone(),
            questions: user.questions.clone(),
            resume: user.resume.clone(),
            messages,
        }
    }

    pub fn into_original(self) -> User {
        let messages = self.messages.into_iter().map(|msg| msg.into_original()).collect();

        User {
            id: self.id,
            profession: self.profession,
            questions: self.questions,
            resume: self.resume,
            messages,
        }
    }
}

impl User {
    pub async fn get_user(id: u64) -> Result<User, &'static str> {
        match db::load_user(id).await {
            Some(u) => Ok(u.into_original()),
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
        if self.resume.is_some() {
            return Need::None;
        }

        if let Some(questions) = &self.questions {
            return match questions.iter().all(|q| q.answer.is_some()) {
                true => Need::Resume,
                false => Need::Answers,
            };
        }

        if self.profession.is_some() {
            return Need::Questions;
        }

        Need::Profession
    }

    pub async fn save(&self) -> Result<(), &'static str> {
        db::save_user(UserWithCustomMessages::from_original(self)).await.map_err(|e| e)?;
        Ok(())
    }

    pub fn set_profession(&mut self, profession: &str) {
        self.profession = Some(profession.to_string());
    }

    pub fn set_questions(&mut self, questions: Vec<String>) {
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
            if let Some(q) = questions.get_mut(ind as usize) {
                q.set_answer(answer);
                return None;
            }

            return Some("invalid question index");
        };

        return Some("no questions");
    }

    pub fn set_resume(&mut self, resume: &str) {
        self.resume = Some(resume.to_string());
    }

    pub fn get_resume(&self) -> Option<String> {
        self.resume.clone()
    }

    pub fn reset(&mut self) {
        *self = User::new(self.id);
    }

    pub fn get_messages(&self, limit: Option<usize>) -> Vec<ChatCompletionRequestMessage> {
        if !limit.is_some() {
            return self.messages.clone();
        }

        let limit = limit.unwrap();
        let mut messages = vec![];

        let mut counter = 0;
        for m in self.messages.iter().rev() {
            let content = match m.clone() {
                ChatCompletionRequestMessage::System(sm) => sm.content,
                ChatCompletionRequestMessage::User(um) => match um.content {
                    ChatCompletionRequestUserMessageContent::Text(t) => t,
                    ChatCompletionRequestUserMessageContent::Array(_) => "".to_string(),
                },
                ChatCompletionRequestMessage::Assistant(am) => am.content.unwrap_or("".to_string()),
                ChatCompletionRequestMessage::Tool(tm) => tm.content,
                ChatCompletionRequestMessage::Function(fm) => fm.content.unwrap_or("".to_string()),
            };
            counter = counter + content.len();

            if counter > limit {
                break;
            }
            messages.push(m.clone());
        }
        messages.reverse();

        return messages;
    }

    pub fn add_message(&mut self, message: ChatCompletionRequestMessage) {
        self.messages.push(message);
    }

    pub fn add_func_success(&mut self, call_id: &str, _: &str) {
        self.add_message(
            ChatCompletionRequestMessage::Tool(
                ChatCompletionRequestToolMessageArgs::default()
                    .tool_call_id(call_id)
                    .content("success")
                    .build().unwrap()
            )
        );
    }

    pub fn get_answers_as_json_str(&self) -> Option<String> {
        if let Some(questions) = &self.questions {
            if let Ok(json_string) = serde_json::to_string(&questions) {
                return Some(json_string);
            }
        }
        None
    }
}