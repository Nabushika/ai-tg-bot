use crate::models::{ChatMessage, Conversation};
use crate::{ai::Model, Role};

use anyhow::Context;
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use async_openai::{config::OpenAIConfig, Client};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OpenAIModel {
    #[serde(with = "client_ser")]
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAIModel {
    pub fn new(api_url: String, model: String) -> Self {
        let config = OpenAIConfig::new().with_api_base(api_url);
        Self {
            client: Client::with_config(config),
            model,
        }
    }
    pub fn new_with_token(api_url: String, model: String, token: String) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(api_url)
            .with_api_key(token);
        Self {
            client: Client::with_config(config),
            model,
        }
    }

    async fn reply_with_system(
        &self,
        system: Option<&str>,
        conversation: &[ChatMessage],
    ) -> anyhow::Result<String> {
        let mut msgs = Vec::with_capacity(conversation.len() + usize::from(system.is_some()));
        if let Some(system) = &system {
            msgs.push(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system.to_owned())
                    .build()
                    .unwrap()
                    .into(),
            );
        }
        msgs.extend(conversation.iter().map(|msg| {
            match &msg.from {
                Role::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(msg.content.clone())
                    .build()
                    .unwrap()
                    .into(),
                Role::User(name) => ChatCompletionRequestUserMessageArgs::default()
                    .content(msg.content.clone())
                    .name(name.clone())
                    .build()
                    .unwrap()
                    .into(),
            }
        }));
        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model.clone())
            .messages(msgs)
            .build()
            .unwrap();

        self.client
            .chat()
            .create(request)
            .await?
            .choices
            .into_iter()
            .nth(0)
            .and_then(|msg| msg.message.content)
            .context("OpenAI client returned empty response!")
    }
}

impl Model for OpenAIModel {
    async fn reply(&self, conversation: &Conversation) -> anyhow::Result<String> {
        self.reply_with_system(conversation.system.as_deref(), &conversation.messages)
            .await
    }
    async fn description(&self, conversation: &Conversation) -> anyhow::Result<String> {
        const DESCRIPTION_SYSTEM_MSG: &str = "Describe the following chat dialogue. Be as concise as possible, limiting your summary to one sentence if at all possible.";
        self.reply_with_system(Some(DESCRIPTION_SYSTEM_MSG), &conversation.messages)
            .await
    }
}

mod client_ser {
    use async_openai::config::Config;

    use super::{Client, OpenAIConfig};

    pub fn serialize<S>(client: &Client<OpenAIConfig>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use secrecy::ExposeSecret;
        let config = client.config();
        let to_ser = (config.api_base(), config.api_key().expose_secret());
        serde::Serialize::serialize(&to_ser, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Client<OpenAIConfig>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (api_base, api_key): (&str, &str) = serde::Deserialize::deserialize(deserializer)?;
        let config = OpenAIConfig::default()
            .with_api_base(api_base)
            .with_api_key(api_key);
        Ok(Client::with_config(config))
    }
}
