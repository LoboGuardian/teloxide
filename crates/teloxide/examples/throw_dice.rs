//! # Throw Dice Bot Example
//!
//! A fun little bot that replies with a random dice emoji `dice` to every incoming message.
//!
//! This is a great starting point for learning how to:
//! - Initialize a `teloxide` bot
//! - Handle incoming messages
//! - Respond with Telegram's built-in dice animation
//!
//! Now message your bot — it will throw a dice for every message it receives!

use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    // Initialize the logger for nice logs like "Starting throw dice bot..."
    pretty_env_logger::init();

    // Log the start of the bot
    log::info!("Starting throw dice bot...");

    // Create a new bot instance from the TELOXIDE_TOKEN environment variable
    let bot = Bot::from_env();

    // `repl` runs an infinite loop that handles each incoming message
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        // Respond with a random dice emoji
        bot.send_dice(msg.chat.id).await?;

        // Indicate that everything went fine
        Ok(())
    })
    .await;
}
