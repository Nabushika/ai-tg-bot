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
    async fn description(&self, conversation: &Conversation) -> anyhow::Result<String> {
        match self {
            Backend::OpenAI(model) => model.description(conversation).await,
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
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
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
    pub fn get_or_create_conversation(&mut self) -> &mut Conversation {
        match self.current_conversation {
            Some(idx) if idx < self.conversations.len() => self.conversations.get_mut(idx).unwrap(),
            _ => {
                self.current_conversation = Some(self.conversations.len());
                self.conversations.push(Conversation::default());
                self.conversations.last_mut().unwrap()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub enum UIState {
    #[default]
    Chatting,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Conversation {
    pub name: String,
    pub messages: Vec<ChatMessage>,
    pub system: Option<String>,
    pub description: Option<String>,
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
            description: None,
        }
    }
}

impl std::fmt::Display for Conversation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(desc) = &self.description {
            write!(f, "{}: {}", self.name, desc)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

pub struct Character {
    pub name: String,
}
