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

pub enum CommandResult<'a> {
    //DoNothing,
    RegenerateLastMessage(&'a mut Conversation),
    ReplyToUser(String),
}

// Does not handle /start
pub fn handle_command<'a>(msg: &str, state: &'a mut UserState) -> Result<CommandResult<'a>> {
    let (cmd, rest) = msg.split_once(' ').unwrap_or((msg, ""));
    // Only work in conversation
    let failed_command = Ok(CommandResult::ReplyToUser(format!(
        "Command `{cmd}` requires you to be in a conversation!"
    )));
    let conversation = state.get_current_conversation();
    match cmd {
        "/reset" => {
            let Some(conversation) = conversation else {
                return failed_command;
            };
            conversation.messages.clear();
            conversation.system = None;
            Ok(CommandResult::ReplyToUser("Conversation reset!".into()))
        }
        "/system" => {
            let Some(conversation) = conversation else {
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
            let Some(conversation) = conversation else {
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
