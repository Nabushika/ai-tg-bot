#![warn(clippy::all, clippy::pedantic)]
use teloxide::prelude::*;
use teloxide::types::ChatAction;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod ai;
mod bot;
mod models;
use ai::openai::OpenAIModel;
use ai::Model;
use bot::CommandResult;
use models::{Backend, ChatMessage, Role, UserState};

const OPENAI_API_URL: &str = "http://localhost:5000/v1";
const GROQ_API_URL: &str = "https://api.groq.com/openai/v1";
const GROQ_MODEL: &str = "llama3-70b-8192";

const COMMANDS: &[(&str, &str)] = &[
    ("reset", "Resets the conversation and system message"),
    ("redo", "Forces the bot to re-type the last message"),
    ("system", "Set the system message for current conversation"),
    ("start", "Start a new conversation. Requires model name."),
];

async fn typing_while<T>(
    bot: &Bot,
    chat_id: ChatId,
    fut: impl std::future::Future<Output = T>,
) -> T {
    let typing_fut = async {
        loop {
            let _ = bot.send_chat_action(chat_id, ChatAction::Typing).await;
            tokio::time::sleep(Duration::from_secs(4)).await;
        }
    };
    tokio::select! {
        () = typing_fut => { unreachable!() },
        res = fut => res,
    }
}

async fn handle_msg(
    bot: &Bot,
    msg: Message,
    mut state: UserState,
    default_backend: &Backend,
) -> anyhow::Result<UserState> {
    let chat_id = msg.chat.id;
    let username = msg
        .from()
        .map_or_else(|| "UNKNOWN".into(), teloxide::types::User::full_name);
    println!("{}: {}", username, msg.text().unwrap_or(""));
    let group_chat = msg.chat.is_group();
    let Some(text) = msg.text() else {
        bot.send_message(chat_id, "This bot only supports text messages! (for now)")
            .await?;
        return Ok(state);
    };
    if text.starts_with('/') {
        // handle command
        let result = bot::handle_command(text, &mut state)?;

        #[allow(clippy::match_wildcard_for_single_variants)]
        match result {
            //CommandResult::DoNothing => {}
            CommandResult::ReplyToUser(msg) => {
                bot.send_message(chat_id, msg).await?;
            }
            CommandResult::RegenerateLastMessage(conversation) => {
                let result =
                    typing_while(bot, chat_id, default_backend.reply(conversation)).await?;
                bot.send_message(chat_id, &result).await?;
                println!("BOT: {result}");
                conversation.messages.push(ChatMessage::new(result, None));
            }
        }
        return Ok(state);
    }
    // Non-command message, handle here
    let Some(conversation) = state.get_current_conversation() else {
        bot.send_message(
            chat_id,
            "Please start a conversation with /start before messaging!",
        )
        .await?;
        return Ok(state);
    };
    let named_message = if group_chat {
        format!("{username}: {text}")
    } else {
        text.into()
    };
    conversation
        .messages
        .push(ChatMessage::new(named_message, Some(username)));
    let response = typing_while(bot, chat_id, default_backend.reply(conversation)).await?;
    conversation
        .messages
        .push(ChatMessage::new(response.clone(), None));
    println!("BOT: {response}");
    bot.send_message(chat_id, response).await?;
    Ok(state)
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
    //let ollama = Ollama::new("http://localhost".into(), 11434);

    // Set up OpenAI client
    //let openai_config = OpenAIConfig::new().with_api_base("http://localhost:5000/v1");
    //let openai_client = OpenAIClient::with_config(openai_config);

    let openai_models = vec!["turboderp_Llama-3-70B-Instruct-exl2_5.0bpw".to_string()]; // Add more as needed

    println!("OpenAI models: {openai_models:?}");

    let chats = std::fs::read("chats.json")
        .ok()
        .and_then(|conts| serde_json::from_slice::<HashMap<ChatId, UserState>>(&conts).ok())
        .inspect(|chats| println!("Loaded {} chats!", chats.len()))
        .unwrap_or_default();
    let chats = Arc::new(Mutex::new(chats));
    let interval_saver_chats = Arc::clone(&chats);
    let final_save_chats = Arc::clone(&chats);

    let mut interval_saver = tokio::time::interval(Duration::from_secs(300));
    tokio::task::spawn(async move {
        loop {
            interval_saver.tick().await;
            let chats_ser = serde_json::to_string(&*interval_saver_chats.lock().unwrap()).unwrap();
            if let Err(e) = tokio::fs::write("./chats.json", chats_ser).await {
                eprintln!("WARNING: failed to autosave chats: {e}");
            }
        }
    });

    let groq_token = std::env::var("GROQ_TOKEN").ok();
    let default_backend = groq_token.map_or_else(
        || {
            println!("Using default local OpenAI backend");
            Backend::OpenAI(OpenAIModel::new(
                OPENAI_API_URL.into(),
                openai_models[0].clone(),
            ))
        },
        |token| {
            println!("Using Groq backend");
            Backend::OpenAI(OpenAIModel::new_with_token(
                GROQ_API_URL.into(),
                GROQ_MODEL.into(),
                token,
            ))
        },
    );

    tokio::select! {
    () = teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let ch = Arc::clone(&chats);
        let default_backend = default_backend.clone();
        async move {
            let chat_id = msg.chat.id;
            let state = ch.lock().unwrap().get(&chat_id).cloned().unwrap_or_default();
            match handle_msg(&bot, msg, state, &default_backend.clone()).await {
                    Ok(new_state) => {
                        ch.lock().unwrap().insert(chat_id, new_state);
                    },
                    Err(e) => eprintln!("Error on handle_msg: {e:?}"),
                }
            Ok(())
        }
    }) => {},
    _ = tokio::signal::ctrl_c() => {
            println!("Shutting down!");
        }
    };

    // Save chats
    let chats_ser = serde_json::to_string_pretty(&*final_save_chats.lock().unwrap()).unwrap();
    std::fs::write("chats.json", chats_ser).unwrap();

    Ok(())
}

//// Start command, pasted here jic I need the keyboard code and stuff later
//// Yes I know this is messy, but... eh
//if let Some(command) = text.strip_prefix("/start") {
//    let command = command.strip_prefix(' ').unwrap_or(command);
//    let parts: Vec<&str> = command.split_whitespace().collect();
//    match parts[..] {
//        ["openai", model] => {
//            if models.contains(&model.to_string()) {
//                state = State::ChatDialogue {
//                    backend: Backend::OpenAI(ai::openai::OpenAIModel::new(
//                        OPENAI_API_URL.to_string(),
//                        model.to_string(),
//                    )),
//                    conversation: Conversation::default(),
//                };
//                bot.send_message(
//                    chat_id,
//                    format!("Chosen model {model}! You can start chatting now."),
//                )
//                .reply_markup(ReplyMarkup::kb_remove())
//                .await?;
//                bot.set_my_commands(
//                    COMMANDS
//                        .iter()
//                        .map(|(cmd, desc)| BotCommand::new(*cmd, *desc)),
//                )
//                .await?;
//            }
//        }
//        _ => {
//            bot.send_message(chat_id, "Please choose a model")
//                .reply_markup(models_to_options(models.clone()))
//                .await?;
//        }
//    }
//    return Ok(state);
//}
//// Not a /start command
//fn models_to_options(openai_models: Vec<String>) -> teloxide::types::ReplyMarkup {
//    let keyboard = openai_models
//        .into_iter()
//        .map(|model| {
//            vec![teloxide::types::KeyboardButton::new(format!(
//                "/start openai {model}"
//            ))]
//        })
//        .collect::<Vec<_>>();
//
//    teloxide::types::KeyboardMarkup::new(keyboard)
//        .one_time_keyboard(true)
//        .into()
//}
//
