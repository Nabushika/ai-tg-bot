use anyhow::Result;

use crate::models::{Role, State};

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

pub enum CommandResult {
    //DoNothing,
    RegenerateLastMessage,
    ReplyToUser(String),
}

// Does not handle /start
pub fn handle_command(msg: &str, mut state: &mut State) -> Result<CommandResult> {
    let (cmd, rest) = msg.split_once(' ').unwrap_or((msg, ""));
    // Only work in conversation
    let failed_command = Ok(CommandResult::ReplyToUser(format!(
        "Command `{cmd}` requires you to be in a conversation!"
    )));
    #[allow(clippy::match_wildcard_for_single_variants)]
    let conversation = match &mut state {
        State::ChatDialogue { conversation, .. } => Some(conversation),
        _ => None,
    };
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
            conversation.messages.pop();
            Ok(CommandResult::RegenerateLastMessage)
        }
        _ => Ok(CommandResult::ReplyToUser(format!(
            "Unknown command {cmd}."
        ))),
    }
}
