use std::fs::read_to_string;
use async_openai::types::{ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionResponseMessage};
use serde_json::{json, Value};
use crate::openai::{get_response, Request};


#[derive(Debug)]
pub enum Response {
    Text(String),
    Error(String),
    Profession(ToolCallRequest, String),
    Questions(ToolCallRequest, Vec<String>),
    Answers(ChatCompletionRequestMessage, Vec<(ToolCallRequest, (u8, String))>),
}

#[derive(Debug)]
pub struct ToolCallRequest {
    pub call_id: String,
    pub function_name: String,
    pub request_message: Option<ChatCompletionRequestMessage>,
}

impl ToolCallRequest {
    fn new(call_id: String, function_name: String, request_message: Option<ChatCompletionRequestMessage>) -> Self {
        ToolCallRequest { call_id, function_name, request_message }
    }
}

pub struct Asker {
    api_key: String,
    max_tokens: Option<u16>,
    model: Option<String>,
    system_message: Option<String>,
}

impl Asker {
    pub fn new(api_key: String, max_tokens: Option<u16>, model: Option<String>, system_message: Option<String>) -> Self {
        Asker { api_key, max_tokens, model, system_message }
    }

    pub async fn get_profession(&self, messages: Vec<ChatCompletionRequestMessage>) -> Response {
        return self.abstract_get(
            messages,
            vec![
                ("save_profession", "Save the profession", json!({
                "type": "object",
                "properties": {
                    "profession": {
                        "type": "string",
                        "description": "Name of profession, e.g. Software Developer",
                    },
                },
                "required": ["profession"],
            }))
            ],
            "./src/data/prompt_profession.txt",
            |tool_calls, response_message| {
                if let Some(tool_call) = tool_calls.first() {
                    let arguments: Value = tool_call.function.arguments.parse().unwrap();
                    return Response::Profession(
                        ToolCallRequest::new(
                            tool_call.id.clone(),
                            tool_call.function.name.clone(),
                            Some(to_request(response_message)),
                        ),
                        arguments["profession"].to_string(),
                    );
                };
                return Response::Error("Exception #4699740191".to_string());
            },
        ).await;
    }

    async fn abstract_get<F>(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        raw_functions: Vec<(&str, &str, Value)>,
        default_prompt_filepath: &str,
        custom_behavior: F,
    ) -> Response
        where
            F: Fn(&Vec<ChatCompletionMessageToolCall>, ChatCompletionResponseMessage) -> Response,
    {
        let default_message = read_to_string(default_prompt_filepath)
            .expect(&format!("Failed to get default message from \"{default_prompt_filepath}\""));

        let mut all_messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(
                        match &self.system_message {
                            Some(message) => message,
                            None => &default_message
                        }
                    )
                    .build()
                    .unwrap()
            )
        ];
        all_messages.extend(messages);

        let get_result = self.get(
            all_messages, raw_functions,
        ).await;

        match get_result {
            Ok(message) => {
                if let Some(ref tool_calls) = message.tool_calls {
                    return custom_behavior(tool_calls, message.clone());
                };
                return Response::Text(message.content.unwrap());
            }
            Err(e) => Response::Error(e.to_string()),
        }
    }

    async fn get(&self, messages: Vec<ChatCompletionRequestMessage>, raw_functions: Vec<(&str, &str, Value)>) -> Result<ChatCompletionResponseMessage, &'static str> {
        let request = Request::new(
            self.api_key.clone(),
            messages,
            self.max_tokens,
            self.model.clone(),
            raw_functions,
        );

        return get_response(request).await.map_err(|_| "openai_error");
    }

    pub async fn get_questions(&self, messages: Vec<ChatCompletionRequestMessage>) -> Response {
        return self.abstract_get(
            messages,
            vec![
                ("add_questions", "Set a list of questions for creating a resume for a profession", json!({
                "type": "object",
                "properties": {
                    "questions": {
                        "type": "array",
                        "description": "A list of questions",
                        "items": {
                            "type": "string",
                            "description": "Question, e.g. “full name”, “gender”, “name”, “work experience”, “list of programming languages studied”, “knowledge of the Django framework and at what level”",
                        },
                        "minItems": 5,
                        "maxItems": 20,
                    },
                },
                "required": ["questions"],
            }))
            ],
            "./src/data/prompt_questions.txt",
            |tool_calls, response_message| {
                for tool_call in tool_calls {
                    if let Ok(args) = parse_json(&tool_call.function.arguments) {
                        if let Some(qs) = args["questions"].as_array() {
                            return Response::Questions(
                                ToolCallRequest::new(
                                    tool_call.id.clone(),
                                    tool_call.function.name.clone(),
                                    Some(to_request(response_message)),
                                ),
                                qs.iter()
                                    .map(move |x| x.to_string())
                                    .collect(),
                            );
                        }
                    }
                }
                Response::Error("Exception #6407321013".to_string())
            },
        ).await;
    }

    pub async fn get_answers(&self, messages: Vec<ChatCompletionRequestMessage>) -> Response {
        return self.abstract_get(
            messages,
            vec![
                ("set_answer", "Set answer to the survey question by index", json!({
                "type": "object",
                "properties": {
                    "index": {
                        "type": "integer",
                        "description": "index question from the survey",
                    },
                    "answer": {
                        "type": "string",
                        "description": "answer to the survey question",
                    },
                },
                "required": ["index", "answer"],
            }))
            ],
            "./src/data/prompt_answers.txt",
            |tool_calls, response_message| {
                let mut answers: Vec<(ToolCallRequest, (u8, String))> = vec![];

                for tool_call in tool_calls {
                    if let Ok(args) = parse_json(&tool_call.function.arguments) {
                        let index = args["index"].as_u64().unwrap() as u8;
                        let answer = args["answer"].as_str().unwrap();
                        answers.push(
                            (
                                ToolCallRequest::new(
                                    tool_call.id.clone(),
                                    tool_call.function.name.clone(),
                                    None,
                                ),
                                (index, answer.to_string())
                            )
                        );
                    }
                }

                return match answers.is_empty() {
                    true => Response::Error("Exception #6407321013".to_string()),
                    false => Response::Answers(to_request(response_message), answers)
                };
            },
        ).await;
    }
}

fn parse_json(json_str: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str(json_str)
}

fn to_request(response: ChatCompletionResponseMessage) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage::Assistant(
        ChatCompletionRequestAssistantMessageArgs::default()
            .content(response.content.unwrap())
            .tool_calls(response.tool_calls.unwrap())
            .build()
            .expect("Failed #4143141532467235")
    )
}
