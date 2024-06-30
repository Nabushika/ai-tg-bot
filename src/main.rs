use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::ChatMessage as OllamaChatMessage;
use ollama_rs::Ollama;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use teloxide::types::ChatAction;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Backend {
    Ollama(String), // String is the model name
    OpenAI(String), // String is the model name
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum State {
    Start,
    ChatDialogue {
        backend: Backend,
        messages: Vec<String>, // Store messages as strings for compatibility
        system: Option<String>,
    },
}

impl Default for State {
    fn default() -> Self {
        State::Start
    }
}

fn models_to_options(
    ollama_models: Vec<String>,
    openai_models: Vec<String>,
) -> teloxide::types::ReplyMarkup {
    let mut keyboard = ollama_models
        .into_iter()
        .map(|model| {
            vec![teloxide::types::KeyboardButton::new(format!(
                "/start ollama {model}"
            ))]
        })
        .collect::<Vec<_>>();

    keyboard.extend(openai_models.into_iter().map(|model| {
        vec![teloxide::types::KeyboardButton::new(format!(
            "/start openai {model}"
        ))]
    }));

    teloxide::types::KeyboardMarkup::new(keyboard)
        .one_time_keyboard(true)
        .into()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up the Telegram bot API
    let Ok(tg_bot_token) = std::env::var("TG_BOT_TOKEN") else {
        eprintln!("Need telegram bot token TG_BOT_TOKEN in environment");
        anyhow::bail!("Need telegram bot token TG_BOT_TOKEN in environment");
    };
    let bot = Bot::new(tg_bot_token);

    // Set up the OLLAMA model
    let ollama = Ollama::new("http://localhost".into(), 11434);

    // Set up OpenAI client
    let openai_config = OpenAIConfig::new().with_api_base("http://localhost:5000/v1");
    let openai_client = OpenAIClient::with_config(openai_config);

    let ollama_models = ollama
        .list_local_models()
        .await?
        .into_iter()
        .map(|model| model.name)
        .collect::<Vec<_>>();

    let openai_models = vec!["turboderp_Llama-3-70B-Instruct-exl2_5.0bpw".into()]; // Add more as needed

    let chats = std::fs::read("chats.json")
        .ok()
        .and_then(|conts| serde_json::from_slice::<HashMap<ChatId, State>>(&conts).ok())
        .unwrap_or_default();
    let chats = Arc::new(Mutex::new(chats));
    let my_chats = Arc::clone(&chats);

    tokio::select! {
    _ = teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let ch = Arc::clone(&chats);
        let ollama_models = ollama_models.clone();
        let openai_models = openai_models.clone();
        let ollama = ollama.clone();
        let openai_client = openai_client.clone();
        async move {
            let chat_id = msg.chat.id;
            let mut state = ch.lock().unwrap().remove(&chat_id).unwrap_or_default();
            let send_message = match state {
                State::Start => {
                    if let Some(text) = msg.text() {
                        if let Some(command) = text.strip_prefix("/start ") {
                            let parts: Vec<&str> = command.split_whitespace().collect();
                            if parts.len() == 2 {
                                let backend = parts[0];
                                let model = parts[1];
                                match backend {
                                    "ollama" if ollama_models.contains(&model.to_string()) => {
                                        state = State::ChatDialogue {
                                            backend: Backend::Ollama(model.to_string()),
                                            messages: vec![],
                                            system: None,
                                        };
                                        Some(format!("Chosen Ollama model {model}! You can start chatting now."))
                                    }
                                    "openai" if openai_models.contains(&model.to_string()) => {
                                        state = State::ChatDialogue {
                                            backend: Backend::OpenAI(model.to_string()),
                                            messages: vec![],
                                            system: None,
                                        };
                                        Some(format!("Chosen OpenAI model {model}! You can start chatting now."))
                                    }
                                    _ => Some("Invalid model or backend. Please choose again.".to_string()),
                                }
                            } else {
                                Some("Invalid command format. Use '/start backend model'".to_string())
                            }
                        } else {
                            bot.send_message(chat_id, "Please choose a model")
                                .reply_markup(models_to_options(ollama_models.clone(), openai_models.clone()))
                                .await?;
                            None
                        }
                    } else {
                        Some("This bot only accepts text messages (for now)!".into())
                    }
                }
                State::ChatDialogue {
                    ref backend,
                    ref mut messages,
                    ref mut system,
                } => {
                    if let Some(text) = msg.text() {
                        if text == "/reset" {
                            state = State::Start;
                            Some("Conversation reset. Choose a model to start again.".into())
                        } else if let Some(sys_cmd) = text.strip_prefix("/system ") {
                            *system = Some(sys_cmd.to_string());
                            Some("Set system message (only works with OpenAI backend for now)".into())
                        } else {
                            messages.push(text.to_string());
                            bot.send_chat_action(chat_id, ChatAction::Typing).await?;

                            let response = match backend {
                                Backend::Ollama(model) => {
                                    let ollama_messages = messages.iter()
                                        .enumerate()
                                        .map(|(i, m)| {
                                            if i % 2 == 0 {
                                                OllamaChatMessage::user(m.to_string())
                                            } else {
                                                OllamaChatMessage::assistant(m.to_string())
                                            }
                                        })
                                        .collect::<Vec<_>>();

                                    let response = ollama
                                        .send_chat_messages(ChatMessageRequest::new(
                                            model.clone(),
                                            ollama_messages,
                                        ))
                                        .await.unwrap();
                                    response.message.unwrap().content
                                }
                                Backend::OpenAI(model) => {
                                    let mut openai_messages = messages.iter()
                                        .enumerate()
                                        .map(|(i, m)| {
                                            if i % 2 == 0 {
                                                ChatCompletionRequestUserMessageArgs::default()
                                                    .content(m.to_string())
                                                    .build().unwrap().into()
                                            } else {
                                                ChatCompletionRequestAssistantMessageArgs::default()
                                                    .content(m.to_string())
                                                    .build().unwrap().into()
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    if let Some(system) = system {
                                        openai_messages.insert(0, ChatCompletionRequestSystemMessageArgs::default().content(system.to_string()).build().unwrap().into());
                                    }

                                    let request = CreateChatCompletionRequestArgs::default()
                                        .model(model)
                                        .messages(openai_messages)
                                        .build()
                                        .unwrap();

                                    let response = openai_client.chat().create(request).await.unwrap();
                                    response.choices[0].message.content.clone().unwrap_or_default()
                                }
                            };

                            messages.push(response.clone());
                            Some(response)
                        }
                    } else {
                        Some("This bot only accepts text messages (for now)!".into())
                    }
                }
            };
            if let Some(send) = send_message {
                println!("{}: {}", msg.from().map(|user| user.full_name()).unwrap_or_else(|| "UNKNOWN".into()), msg.text().unwrap_or(""));
                println!("BOT: {}", send);
                bot.send_message(chat_id, send).await?;
            }
            ch.lock().unwrap().insert(chat_id, state);
            Ok(())
        }
    }) => {},
    _ = tokio::signal::ctrl_c() => {
            println!("Shutting down!")
        }
    };

    // Save chats
    let chats_ser = serde_json::to_string_pretty(&*my_chats.lock().unwrap()).unwrap();
    std::fs::write("chats.json", chats_ser).unwrap();

    Ok(())
}
