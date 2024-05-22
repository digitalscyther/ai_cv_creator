use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::{ChatCompletionFunctionCall, ChatCompletionFunctions, ChatCompletionFunctionsArgs, ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs, ChatCompletionResponseMessage, CreateChatCompletionRequestArgs, Role};
use derivative::Derivative;
use serde_json::{Value};


const DEFAULT_MODEL: &str = "gpt-3.5-turbo";


#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct Functions {
    functions: Vec<ChatCompletionFunctions>,
    #[derivative(Default(value = "ChatCompletionFunctionCall::None"))]
    mode: ChatCompletionFunctionCall,
}

impl Functions {
    pub fn new(
        raw_functions: Vec<(&str, &str, Value)>,
        mode: Option<ChatCompletionFunctionCall>,
    ) -> Self {
        let functions: Vec<ChatCompletionFunctions> = raw_functions
            .into_iter()
            .map(|(name, description, parameters)| {
                ChatCompletionFunctionsArgs::default()
                    .name(name)
                    .description(description)
                    .parameters(parameters)
                    .build()
                    .unwrap()
            })
            .collect();

        Functions {
            functions,
            mode: mode.unwrap_or(ChatCompletionFunctionCall::Auto),
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct Request {
    api_key: String,
    #[derivative(Default(value = "512"))]
    max_tokens: u16,
    #[derivative(Default(value = "DEFAULT_MODEL.to_string()"))]
    model: String,
    messages: Vec<ChatCompletionRequestMessage>,
    function: Option<Functions>,
}

impl Request {
    pub fn new(
        api_key: String,
        raw_messages: Vec<(Role, &str)>,
        max_tokens: Option<u16>,
        model: Option<String>,
        function: Option<Functions>,
    ) -> Self {
        let messages: Vec<ChatCompletionRequestMessage> = raw_messages
            .into_iter()
            .map(|(role, content)| {
                ChatCompletionRequestUserMessageArgs::default()
                    .role(role)
                    .content(content)
                    .build()
                    .unwrap()
                    .into()
            })
            .collect();

        Request {
            api_key,
            max_tokens: max_tokens.unwrap_or(512),
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            messages,
            function,
        }
    }
}


pub async fn get_response(request: Request) -> Result<ChatCompletionResponseMessage, OpenAIError> {
    let config = OpenAIConfig::new().with_api_key(&request.api_key);
    let client = Client::with_config(config);

    let mut args = CreateChatCompletionRequestArgs::default();
    let mut request_builder = args
        .max_tokens(request.max_tokens)
        .model(request.model)
        .messages(request.messages);

    if let Some(function) = request.function {
        request_builder
            .functions(function.functions)
            .function_call(function.mode);
    };
    let request = request_builder.build()?;

    Ok(client.chat()
        .create(request)
        .await?
        .choices
        .get(0)
        .unwrap()
        .message
        .clone())
}