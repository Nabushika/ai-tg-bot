use crate::models::Conversation;

pub mod openai;

pub trait AiModel {
    async fn reply(&self, conversation: &Conversation) -> anyhow::Result<String>;
}
