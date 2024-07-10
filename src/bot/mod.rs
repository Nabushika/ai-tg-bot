use anyhow::Result;

use crate::models::{Conversation, Role, UserState};

// command => requirements
// start => state, models? Tg bot for keyboard
//   models static: do we want dyn Fn()?
// (x) reset => conversation
// redo => conversation, ai bot?
// (x) system => conversation
//
// FUTURE
// character.new => state?
// character.select => state, conversation
// conversation.new/select => state, conversation
//
// also most of these only work inside a conversation...
//
// EVERYTHING NEEDED
// tg bot, state(conversation), models??, ai bot??

// TODO: fit this into the commands, rather than having to maintain a separate list
const COMMANDS: &[(&str, &str)] = &[
    ("reset", "Resets the conversation and system message"),
    ("redo", "Forces the bot to re-type the last message"),
    ("system", "Set the system message for current conversation"),
    //("start", "Start a new conversation. Requires model name."),
    ("help", "Show a list of commands and brief descriptions"),
    ("rename", "Rename conversation"),
    ("desc", "Update description of conversation"),
    ("new", "Start a new conversation"),
    ("list", "List all conversations"),
];

pub enum CommandResult<'a> {
    //DoNothing,
    RegenerateLastMessage(&'a mut Conversation),
    ReplyToUser(String),
    GenerateDescription(&'a mut Conversation),
}

// Does not handle /start
pub fn handle_command<'a>(msg: &str, state: &'a mut UserState) -> Result<CommandResult<'a>> {
    let (cmd, rest) = msg.split_once(' ').unwrap_or((msg, ""));
    // Only work in conversation
    let failed_command = Ok(CommandResult::ReplyToUser(format!(
        "Command `{cmd}` requires you to be in a conversation!"
    )));
    match cmd {
        "/reset" => {
            let Some(conversation) = state.get_current_conversation() else {
                return failed_command;
            };
            conversation.messages.clear();
            conversation.system = None;
            Ok(CommandResult::ReplyToUser("Conversation reset!".into()))
        }
        "/rename" => {
            let conversation = state.get_or_create_conversation();
            rest.clone_into(&mut conversation.name);
            Ok(CommandResult::ReplyToUser(format!(
                "Set current conversation name to \"{rest}\"!"
            )))
        }
        "/desc" => {
            let Some(conversation) = state.get_current_conversation() else {
                return failed_command;
            };
            Ok(CommandResult::GenerateDescription(conversation))
        }
        "/new" => {
            state.current_conversation = None;
            Ok(CommandResult::ReplyToUser(
                "New conversation started".into(),
            ))
        }
        "/list" => {
            let conversations = state
                .conversations
                .iter()
                .map(|f| format!("{f}"))
                .collect::<Vec<_>>()
                .join("\n\n");
            Ok(CommandResult::ReplyToUser(format!(
                "Current conversations: {conversations}"
            )))
        }
        "/system" => {
            let Some(conversation) = state.get_current_conversation() else {
                return failed_command;
            };
            if rest.is_empty() {
                return Ok(CommandResult::ReplyToUser(
                    "Please set a system message with `/system [system message]`.".into(),
                ));
            }
            conversation.system = Some(rest.into());
            Ok(CommandResult::ReplyToUser("System message set!".into()))
        }
        "/redo" => {
            let Some(conversation) = state.get_current_conversation() else {
                return failed_command;
            };
            if conversation
                .messages
                .last()
                .map_or(false, |m| m.from != Role::Assistant)
            {
                return Ok(CommandResult::ReplyToUser(
                    "Can only /redo if the last message is LlamaBot's!".into(),
                ));
            };
            Ok(CommandResult::RegenerateLastMessage(conversation))
        }
        _ => Ok(CommandResult::ReplyToUser(format!(
            "Unknown command {cmd}."
        ))),
    }
}
