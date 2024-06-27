use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs};
use sqlx::{Pool, Postgres};
use crate::ask::{Asker, Response};
use crate::user::{Need, User};

const MAX_HISTORY: usize = 5_000;
const MAX_TOKENS: u32 = 50_000;

pub struct Dialogue {
    user: User,
    asker: Asker,
    max_history: usize,
    max_tokens: u32,
}

pub enum Instruction {
    SaveResume,
    DeleteResume(String),
    None,
}

impl Dialogue {
    pub fn new(user: User, asker: Asker, max_history: Option<usize>, max_tokens: Option<u32>) -> Self {
        let max_history = max_history.unwrap_or(MAX_HISTORY);
        let max_tokens = max_tokens.unwrap_or(MAX_TOKENS);
        Self { user, asker, max_history, max_tokens }
    }

    pub async fn set_resume(&mut self, name: &str) -> Result<(), &'static str> {
        self.user.set_resume(name);
        Ok(())
    }

    pub async fn process_message(&mut self, text: Option<&str>) -> (Option<String>, Instruction) {
        if let Some(text) = text {
            if text == "reset" {
                let instruction = match self.user.get_resume() {
                    Some(name) => Instruction::DeleteResume(name),
                    None => Instruction::None
                };
                self.user.reset();
                return (Some("Data reset".to_string()), instruction)
            }

            self.user.add_message(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(text)
                    .build()
                    .expect("failed build ChatCompletionRequestUserMessageArgs")
                    .into()
            )
        }

        let messages = self.user.get_messages(Some(self.max_history));

        match self.user.need() {
            Need::None => {
                (match text {
                    Some(text) if text == "resume" => self.user.get_resume(),
                    _ => Some("the end".to_string())
                }, Instruction::None)
            }
            others => {
                if self.user.not_enough_tokens(self.max_tokens) {
                    return (Some("Limit exceed".to_string()), Instruction::None);
                }

                return match others {
                    Need::None => unimplemented!(),
                    Need::Profession => {
                        let payable_response = self.asker.get_profession(messages).await;
                        self.user.add_tokens_spent(payable_response.tokens_spent);
                        (match payable_response.response {
                            Response::Profession(tool_call, profession) => {
                                self.user.add_message(tool_call.request_message.unwrap());
                                self.user.add_func_success(&tool_call.call_id, &tool_call.function_name);
                                self.user.set_profession(&profession);
                                None
                            }
                            Response::Text(text) => {
                                self.user.add_message(
                                    ChatCompletionRequestMessage::Assistant(
                                        ChatCompletionRequestAssistantMessageArgs::default()
                                            .content(&text)
                                            .build().unwrap()
                                    )
                                );
                                Some(text.to_string())
                            }
                            smt => panic!("Profession case _: {:?}", smt)
                        }, Instruction::None)
                    }
                    Need::Questions => {
                        let payable_response = self.asker.get_questions(messages).await;
                        self.user.add_tokens_spent(payable_response.tokens_spent);
                        (match payable_response.response {
                            Response::Questions(tool_call, questions) => {
                                self.user.add_message(tool_call.request_message.unwrap());
                                self.user.add_func_success(&tool_call.call_id, &tool_call.function_name);
                                self.user.set_questions(questions);
                                None
                            }
                            Response::Text(text) => {
                                self.user.add_message(
                                    ChatCompletionRequestMessage::Assistant(
                                        ChatCompletionRequestAssistantMessageArgs::default()
                                            .content(&text)
                                            .build().unwrap()
                                    )
                                );
                                Some(text.to_string())
                            }
                            Response::Error(text) => panic!("Error case {:?}", text),
                            smt => panic!("Questions case _: {:?}", smt)
                        }, Instruction::None)
                    }
                    Need::Answers => {
                        let payable_response = self.asker.get_answers(self.answer_with_messages(messages)).await;
                        self.user.add_tokens_spent(payable_response.tokens_spent);
                        (match payable_response.response {
                            Response::Answers(
                                func_request_message, answers
                            ) => {
                                self.user.add_message(func_request_message);
                                for (tool_call, (index, answer)) in answers {
                                    self.user.add_func_success(&tool_call.call_id, &tool_call.function_name);
                                    self.user.set_answer(index, &answer);
                                }
                                None
                            }
                            Response::Text(text) => {
                                self.user.add_message(
                                    ChatCompletionRequestMessage::Assistant(
                                        ChatCompletionRequestAssistantMessageArgs::default()
                                            .content(&text)
                                            .build().unwrap()
                                    )
                                );
                                Some(text.to_string())
                            }
                            Response::Error(text) => panic!("Error case {:?}", text),
                            smt => panic!("Answers case _: {:?}", smt)
                        }, Instruction::None)
                    }
                    Need::Resume => {
                        let payable_response = self.asker.clone_with_max_tokens(
                            4_000   // TODO better
                        ).get_resume(self.answer_with_messages(vec![])).await;
                        self.user.add_tokens_spent(payable_response.tokens_spent);
                        match payable_response.response {
                            Response::Resume(tool_call, resume) => {
                                self.user.add_message(tool_call.request_message.unwrap());
                                self.user.add_func_success(&tool_call.call_id, &tool_call.function_name);
                                (Some(resume), Instruction::SaveResume)
                            }
                            Response::Text(text) => {
                                self.user.add_message(
                                    ChatCompletionRequestMessage::Assistant(
                                        ChatCompletionRequestAssistantMessageArgs::default()
                                            .content(&text)
                                            .build().unwrap()
                                    )
                                );
                                (Some(text.to_string()), Instruction::None)
                            }
                            smt => panic!("Resume case _: {:?}", smt)
                        }
                    }
                };
            }
        }
    }

    pub async fn save_user(&mut self, pool: &Pool<Postgres>) {
        self.user.save(pool).await.expect("dialogue user save failed")
    }

    fn answer_with_messages(&self, messages: Vec<ChatCompletionRequestMessage>) -> Vec<ChatCompletionRequestMessage> {
        match self.user.get_answers_as_json_str() {
            Some(answers) => merge_messages(
                vec![
                    ChatCompletionRequestMessage::System(
                        ChatCompletionRequestSystemMessageArgs::default()
                            .content(answers)
                            .build().unwrap()
                    )
                ],
                messages,
            ),
            None => messages
        }
    }
}

fn merge_messages(messages0: Vec<ChatCompletionRequestMessage>, messages1: Vec<ChatCompletionRequestMessage>) -> Vec<ChatCompletionRequestMessage> {
    let mut merged_messages = Vec::with_capacity(messages0.len() + messages1.len());
    merged_messages.extend(messages0.into_iter());
    merged_messages.extend(messages1.into_iter());
    merged_messages
}

