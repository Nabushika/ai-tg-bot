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

// new model with characters n conversations n stuff

// TODO: probably shouldn't have to be `Clone`
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserState {
    //pub backend: Option<Backend>,
    pub conversations: Vec<Conversation>,
    pub current_conversation: Option<usize>,
    //pub characters: Vec<Character>,
    pub ui_state: UIState,
}

impl UserState {
    pub fn get_current_conversation(&mut self) -> Option<&mut Conversation> {
        self.current_conversation
            .and_then(|idx| self.conversations.get_mut(idx))
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub enum UIState {
    #[default]
    Chatting,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub name: String,
    pub messages: Vec<ChatMessage>,
    pub system: Option<String>,
    //pub character: Option<String>,
}

impl Default for Conversation {
    fn default() -> Self {
        Self {
            name: format!(
                "Conversation from {}",
                chrono::Utc::now().format("%d/%m/%Y %H:%M")
            ),
            messages: vec![],
            system: None,
        }
    }
}

pub struct Character {
    pub name: String,
}
