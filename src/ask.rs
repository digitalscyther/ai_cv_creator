use async_openai::types::{ChatCompletionMessageToolCall, ChatCompletionResponseMessage, Role};
use serde_json::{json, Value};
use crate::openai::{get_response, Request};

#[derive(Debug)]
pub enum Response {
    Text(String),
    Error(String),
    Profession(String),
    Questions(Vec<String>),
    Answers(Vec<(u8, String)>),
}

pub struct Asker {
    api_key: String,
    max_tokens: Option<u16>,
    model: Option<String>,
}

impl Asker {
    pub fn new(api_key: String, max_tokens: Option<u16>, model: Option<String>) -> Self {
        Asker { api_key, max_tokens, model }
    }

    pub async fn get_profession(&self, messages: Vec<(Role, &str)>) -> Response {
        return self.abstract_get(
            messages,
            vec![
                ("set_profession", "Set the profession", json!({
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
            |tool_calls| {
                if let Some(tool_call) = tool_calls.first() {
                    let arguments: Value = tool_call.function.arguments.parse().unwrap();
                    return Response::Profession(arguments["profession"].to_string());
                };
                return Response::Error("Exception #4699740191".to_string());
            },
        ).await;
    }

    async fn abstract_get<F>(
        &self,
        messages: Vec<(Role, &str)>,
        raw_functions: Vec<(&str, &str, Value)>,
        custom_behavior: F,
    ) -> Response
        where
            F: Fn(&Vec<ChatCompletionMessageToolCall>) -> Response,
    {
        let get_result = self.get(
            messages,
            raw_functions,
        ).await;

        match get_result {
            Ok(message) => {
                if let Some(tool_calls) = message.tool_calls {
                    return custom_behavior(&tool_calls);
                };
                return Response::Text(message.content.unwrap());
            }
            Err(e) => Response::Error(e.to_string()),
        }
    }

    async fn get(&self, messages: Vec<(Role, &str)>, raw_functions: Vec<(&str, &str, Value)>) -> Result<ChatCompletionResponseMessage, &'static str> {
        let request = Request::new(
            self.api_key.clone(),
            messages,
            self.max_tokens,
            self.model.clone(),
            raw_functions,
        );

        return get_response(request).await.map_err(|_| "openai_error");
    }

    pub async fn get_questions(&self, messages: Vec<(Role, &str)>) -> Response {
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
            |tool_calls| {
                let mut questions = vec![];

                for tool_call in tool_calls {
                    if let Ok(args) = parse_json(&tool_call.function.arguments) {
                        if let Some(qs) = args["questions"].as_array() {
                            for q in qs {
                                if let Some(question) = q.as_str() {
                                    questions.push(question.to_string());
                                }
                            }
                        }
                    }
                }

                if !questions.is_empty() {
                    Response::Questions(questions)
                } else {
                    Response::Error("Exception #6407321013".to_string())
                }
            },
        ).await;
    }

    pub async fn get_answers(&self, messages: Vec<(Role, &str)>) -> Response {
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
            |tool_calls| {
                let mut answers = vec![];

                for tool_call in tool_calls {
                    if let Ok(args) = parse_json(&tool_call.function.arguments) {
                        let index = args["index"].as_u64().unwrap() as u8;
                        let answer = args["answer"].as_str().unwrap();
                        answers.push((index, answer.to_string()))
                    }
                }

                if !answers.is_empty() {
                    Response::Answers(answers)
                } else {
                    Response::Error("Exception #6407321013".to_string())
                }
            },
        ).await;
    }
}

fn parse_json(json_str: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str(json_str)
}
