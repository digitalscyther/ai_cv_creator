use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs};
use crate::ask::{Asker, Response};
use crate::user::{Need, User};

const MAX_HISTORY: usize = 5_000;

pub struct Dialogue {
    user: User,
    asker: Asker,
}

impl Dialogue {
    pub fn new(user: User, asker: Asker) -> Self {
        Self { user, asker }
    }

    pub async fn process_message(&mut self, text: Option<&str>) -> Option<String> {
        if let Some(text) = text {
            self.user.add_message(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(text)
                    .build()
                    .expect("failed build ChatCompletionRequestUserMessageArgs")
                    .into()
            )
        }

        let messages = self.user.get_messages(Some(MAX_HISTORY));

        match self.user.need() {
            Need::Profession => {
                match self.asker.get_profession(messages).await {
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
                    _ => panic!("Profession case _")
                }
            }
            Need::Questions => {
                match self.asker.get_questions(messages).await {
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
                    _ => panic!("Response case _")
                }
            }
            Need::Answers => {
                match self.asker.get_answers(self.answer_with_messages(messages)).await {
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
                    _ => panic!("Response case _")
                }
            }
            Need::Resume => {
                match self.asker.get_resume(self.answer_with_messages(messages)).await {
                    Response::Resume(tool_call, resume) => {
                        self.user.add_message(tool_call.request_message.unwrap());
                        self.user.add_func_success(&tool_call.call_id, &tool_call.function_name);
                        self.user.set_resume(&resume);
                        self.user.get_resume()
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
                    _ => panic!("Profession case _")
                }
            }
            Need::None => {
                match text {
                    Some(text) if text == "resume" => self.user.get_resume(),
                    _ => Some("the end".to_string())
                }
            }
        }
    }

    pub async fn save_user(&mut self) {
        self.user.save().await.expect("dialogue user save failed")
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

