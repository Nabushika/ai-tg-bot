use crate::models::Conversation;

pub mod openai;

pub trait Model {
    async fn reply(&self, conversation: &Conversation) -> anyhow::Result<String>;
    async fn description(&self, conversation: &Conversation) -> anyhow::Result<String>;
}
