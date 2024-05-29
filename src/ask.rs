use std::fs::read_to_string;
use async_openai::types::{ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionResponseMessage};
use serde_json::{json, Value};
use crate::openai::{ChatResponse, get_response, Request};


#[derive(Debug)]
pub enum Response {
    Text(String),
    Error(String),
    Profession(ToolCallRequest, String),
    Questions(ToolCallRequest, Vec<String>),
    Answers(ChatCompletionRequestMessage, Vec<(ToolCallRequest, (u8, String))>),
    Resume(ToolCallRequest, String),
}

#[derive(Debug)]
pub struct PayableResponse {
    pub response: Response,
    pub tokens_spent: u32,
}

impl PayableResponse {
    fn new(response: Response, tokens_spent: u32) -> Self {
        Self { response, tokens_spent }
    }
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

    pub async fn get_profession(&self, messages: Vec<ChatCompletionRequestMessage>) -> PayableResponse {
        self.get_string(
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
            "profession",
            Response::Profession,
        ).await
    }

    async fn get_string<F>(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        raw_functions: Vec<(&str, &str, Value)>,
        file_path: &str,
        result_field_name: &str,
        response_type: F,
    ) -> PayableResponse
        where
            F: Fn(ToolCallRequest, String) -> Response,
    {
        return self.abstract_get(
            messages,
            raw_functions,
            file_path,
            |tool_calls, response_message| {
                if let Some(tool_call) = tool_calls.first() {
                    let arguments: Value = tool_call.function.arguments.parse().unwrap();
                    return response_type(
                        ToolCallRequest::new(
                            tool_call.id.clone(),
                            tool_call.function.name.clone(),
                            Some(to_request(response_message)),
                        ),
                        arguments[result_field_name].to_string(),
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
    ) -> PayableResponse
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

        let (tokens_spent, response) = match get_result {
            Ok(chat_response) => {
                (chat_response.tokens_spent, match chat_response.message.tool_calls {
                    Some(ref tool_calls) => custom_behavior(tool_calls, chat_response.message.clone()),
                    _ => Response::Text(chat_response.message.content.unwrap())
                })
            }
            Err(e) => (0, Response::Error(e.to_string())),
        };
        return PayableResponse::new(response, tokens_spent);
    }

    async fn get(&self, messages: Vec<ChatCompletionRequestMessage>, raw_functions: Vec<(&str, &str, Value)>) -> Result<ChatResponse, String> {
        let request = Request::new(
            self.api_key.clone(),
            messages,
            self.max_tokens,
            self.model.clone(),
            raw_functions,
        );

        return get_response(request).await.map_err(|err| format!("openai_error: {:?}", err));
    }

    pub async fn get_questions(&self, messages: Vec<ChatCompletionRequestMessage>) -> PayableResponse {
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

    pub async fn get_answers(&self, messages: Vec<ChatCompletionRequestMessage>) -> PayableResponse {
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

    pub async fn get_resume(&self, messages: Vec<ChatCompletionRequestMessage>) -> PayableResponse {
        return self.get_string(
            messages,
            vec![
                ("save_resume", "Save the resume", json!({
                "type": "object",
                "properties": {
                    "resume": {
                        "type": "string"
                    },
                },
                "required": ["resume"],
            }))
            ],
            "./src/data/prompt_resume.txt",
            "resume",
            Response::Resume,
        ).await;
    }
}

fn parse_json(json_str: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str(json_str)
}

fn to_request(response: ChatCompletionResponseMessage) -> ChatCompletionRequestMessage {
    let mut message_args = ChatCompletionRequestAssistantMessageArgs::default();

    if let Some(content) = response.content {
        message_args.content(content);
    }

    if let Some(tool_calls) = response.tool_calls {
        message_args.tool_calls(tool_calls);
    }

    let message = message_args.build().expect("Failed #4143141532467235");

    ChatCompletionRequestMessage::Assistant(message)
}
