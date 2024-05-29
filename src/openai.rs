use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::{
    ChatCompletionRequestMessage,
    ChatCompletionResponseMessage,
    ChatCompletionTool,
    ChatCompletionToolArgs,
    CreateChatCompletionRequestArgs,
    FunctionObjectArgs,
};
use derivative::Derivative;
use serde_json::{Value};


const DEFAULT_MODEL: &str = "gpt-3.5-turbo";

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct Request {
    api_key: String,
    #[derivative(Default(value = "512"))]
    max_tokens: u16,
    #[derivative(Default(value = "DEFAULT_MODEL.to_string()"))]
    model: String,
    messages: Vec<ChatCompletionRequestMessage>,
    tool_calls: Option<Vec<ChatCompletionTool>>,
}

impl Request {
    pub fn new(
        api_key: String,
        messages: Vec<ChatCompletionRequestMessage>,
        max_tokens: Option<u16>,
        model: Option<String>,
        raw_functions: Vec<(&str, &str, Value)>,
    ) -> Self {
        let tool_calls: Vec<ChatCompletionTool> = raw_functions
            .into_iter()
            .map(|(name, description, parameters)| {
                ChatCompletionToolArgs::default()
                    .function(
                        FunctionObjectArgs::default()
                            .name(name)
                            .description(description)
                            .parameters(parameters)
                            .build().expect("FunctionObjectArgs didn't build"),
                    ).build().expect("ChatCompletionToolArgs didn't build")
            })
            .collect();

        let tool_calls: Option<Vec<ChatCompletionTool>> = match !tool_calls.is_empty() {
            true => Some(tool_calls),
            false => None
        };

        Request {
            api_key,
            max_tokens: max_tokens.unwrap_or(512),
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            messages,
            tool_calls,
        }
    }
}

pub struct ChatResponse {
    pub message: ChatCompletionResponseMessage,
    pub tokens_spent: u32,
}

pub async fn get_response(request: Request) -> Result<ChatResponse, OpenAIError> {
    let config = OpenAIConfig::new().with_api_key(&request.api_key);
    let client = Client::with_config(config);

    let mut args = CreateChatCompletionRequestArgs::default();
    let request_builder = args
        .max_tokens(request.max_tokens)
        .model(request.model)
        .messages(request.messages);

    if let Some(tool_calls) = request.tool_calls {
        request_builder.tools(tool_calls);
    };
    let request = request_builder.build()?;

    let response = client.chat()
        .create(request)
        .await?;

    Ok(
        ChatResponse {
            message: response.choices.get(0).unwrap().message.clone(),
            tokens_spent: match response.usage {
                Some(u) => u.total_tokens,
                _ => 0
            },
        }
    )
}