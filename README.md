# AI Telegram Bot

Simple Telegram bot allowing the user to interact with an LLM through the OpenAI API

## Features
- Simple UI, walking the user through selecting a model, then just chatting
- Commands to enhance the experience: `/redo` to regenerate a message, `/system` to edit the system message, and `/reset` to clear a conversation.
- Group chat support! If the bot is an admin, it will see all messages.
  - Currently it will reply to every message
- Saves conversations to `./chats.json`, allowing users to pick conversations back up if the bot goes offline.

## Upcoming Features
- Selective replying in group chats
  - will probably use an LLM to decide when it's appropriate to respond
- Proper conversation support (named conversations, ability to switch between conversations, different characters??)
- vision capabilities?? (will probably be a while LMAO)

Currently being tested at [@NabuLlama3Bot](https://t.me/NabuLlama3Bot).
- It's a local model, so don't expect it to be running all the time! (and will be shut down if it gets abused)
- Running Llama3-70b 5.0bpw EXL2 (don't expect GPT-4o or Claude 3.5 levels of responsiveness/intelligence!)
- Messages don't go to OpenAI, but they are stored locally on my PC.

Contributions welcome!
