use serde::{Deserialize, Serialize};

use crate::ai::Model;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Backend {
    //Ollama(String),
    OpenAI(crate::ai::openai::OpenAIModel),
}

impl Model for Backend {
    async fn reply(&self, conversation: &Conversation) -> anyhow::Result<String> {
        match self {
            Backend::OpenAI(model) => model.reply(conversation).await,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Role {
    Assistant,
    User(String), // name
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub content: String,
    pub from: Role,
}
impl ChatMessage {
    pub fn new(content: String, from: Option<String>) -> Self {
        Self {
            content,
            from: from.map_or(Role::Assistant, Role::User),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Conversation {
    pub messages: Vec<ChatMessage>,
    pub system: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum State {
    #[default]
    Start,
    ChatDialogue {
        backend: Backend,
        conversation: Conversation,
    },
}
