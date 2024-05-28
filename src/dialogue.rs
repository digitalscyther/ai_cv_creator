use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs};
use crate::ask::{Asker, Response};
use crate::user::{Need, User};

pub struct Dialogue {
    user: User,
    asker: Asker,
}

impl Dialogue {
    pub fn new(user: User, asker: Asker) -> Self {
        Self { user, asker }
    }

    pub async fn process_message(&mut self, text: Option<String>) -> Option<String> {
        if let Some(text) = text {
            self.user.add_message(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(text)
                    .build()
                    .expect("failed build ChatCompletionRequestUserMessageArgs")
                    .into()
            )
        }

        let messages = self.user.get_messages();

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
                        Some(text)
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
                        Some(text)
                    }
                    _ => panic!("Response case _")
                }
            }
            Need::Answers => {
                match self.asker.get_answers(messages).await {
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
                        Some(text)
                    }
                    _ => panic!("Response case _")
                }
            }
            Need::Result => {
                let result = "result #foo";
                self.user.set_result(&result);
                Some(result.to_string())
            }
            Need::None => {
                Some("the end".to_string())
            }
        }
    }

    pub async fn save_user(&mut self) {
        self.user.save().await.expect("dialogue user save failed")
    }
}

