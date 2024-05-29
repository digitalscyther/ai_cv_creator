use async_openai::types::{ChatCompletionRequestAssistantMessage, ChatCompletionRequestFunctionMessage, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestToolMessage, ChatCompletionRequestUserMessage};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeStruct;
use serde_json::Value;

#[derive(Debug)]
pub struct Message(ChatCompletionRequestMessage);

impl Message {
    pub fn from_original(enum_val: ChatCompletionRequestMessage) -> Self {
        Message(enum_val)
    }

    pub fn into_original(self) -> ChatCompletionRequestMessage {
        self.0
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Message", 2)?;
        match &self.0 {
            ChatCompletionRequestMessage::System(msg) => {
                state.serialize_field("type", "system")?;
                state.serialize_field("content", &serde_json::to_string(&msg).map_err(|_| "Serialization error").unwrap())?;
            }
            ChatCompletionRequestMessage::User(msg) => {
                state.serialize_field("type", "user")?;
                state.serialize_field("content", &serde_json::to_string(&msg).map_err(|_| "Serialization error").unwrap())?;
            }
            ChatCompletionRequestMessage::Assistant(msg) => {
                state.serialize_field("type", "assistant")?;
                state.serialize_field("content", &serde_json::to_string(&msg).map_err(|_| "Serialization error").unwrap())?;
            }
            ChatCompletionRequestMessage::Tool(msg) => {
                state.serialize_field("type", "tool")?;
                state.serialize_field("content", &serde_json::to_string(&msg).map_err(|_| "Serialization error").unwrap())?;
            }
            ChatCompletionRequestMessage::Function(msg) => {
                state.serialize_field("type", "function")?;
                state.serialize_field("content", &serde_json::to_string(&msg).map_err(|_| "Serialization error").unwrap())?;
            }
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        let msg_type = value.get("type").and_then(Value::as_str).ok_or_else(|| {
            serde::de::Error::custom("Missing or invalid `type` field")
        })?;

        match msg_type {
            "system" => {
                let msg = serde_json::from_str::<ChatCompletionRequestSystemMessage>(value["content"].as_str().unwrap()).expect("Failed deserialize ChatCompletionRequestSystemMessage");
                Ok(Message(ChatCompletionRequestMessage::System(msg)))
            }
            "user" => {
                let msg = serde_json::from_str::<ChatCompletionRequestUserMessage>(value["content"].as_str().unwrap()).expect("Failed deserialize ChatCompletionRequestUserMessage");
                Ok(Message(ChatCompletionRequestMessage::User(msg)))
            }
            "assistant" => {
                let msg = serde_json::from_str::<ChatCompletionRequestAssistantMessage>(value["content"].as_str().unwrap()).expect("Failed deserialize ChatCompletionRequestAssistantMessage");
                Ok(Message(ChatCompletionRequestMessage::Assistant(msg)))
            }
            "tool" => {
                let msg = serde_json::from_str::<ChatCompletionRequestToolMessage>(value["content"].as_str().unwrap()).expect("Failed deserialize ChatCompletionRequestToolMessage");
                Ok(Message(ChatCompletionRequestMessage::Tool(msg)))
            }
            "function" => {
                let msg = serde_json::from_str::<ChatCompletionRequestFunctionMessage>(value["content"].as_str().unwrap()).expect("Failed deserialize ChatCompletionRequestFunctionMessage");
                Ok(Message(ChatCompletionRequestMessage::Function(msg)))
            }
            _ => Err(serde::de::Error::unknown_variant(msg_type, &["system", "user", "assistant", "tool", "function"])),
        }
    }
}

