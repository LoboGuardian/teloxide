//! # Shared State Bot Example
//!
//! This example shows how to keep **shared state** in a Telegram bot using `teloxide`.
//!
//! It keeps track of how many messages the bot has received so far.
//! For every new message, it replies with the **total number of messages** it has seen.
//!
//! ## Concepts demonstrated
//!
//! - `Arc<AtomicU64>` for thread-safe shared state.
//! - `Dispatcher::dependencies()` to inject shared state into your handler.
//! - Handling `Message` updates with a simple endpoint.
//!
//! Each time someone messages the bot, it'll reply with a counter like:
//!
//! _"I received 3 messages in total."_

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    // Enable logging for debugging and visibility.
    pretty_env_logger::init();
    log::info!("Starting shared state bot...");

    // Create the bot from the environment token.
    let bot = Bot::from_env();

    // Our message counter: shared between threads/tasks using Arc + AtomicU64.
    let messages_total = Arc::new(AtomicU64::new(0));

    // Define the handler for incoming messages.
    let handler = Update::filter_message().endpoint(
        |bot: Bot, messages_total: Arc<AtomicU64>, msg: Message| async move {
            // Atomically increment the counter and get the previous value.
            let previous = messages_total.fetch_add(1, Ordering::Relaxed);

            // Respond with the current total message count.
            bot.send_message(msg.chat.id, format!("I received {previous} messages in total."))
                .await?;

            respond(())
        },
    );

    // Set up the dispatcher with our bot and handler.
    Dispatcher::builder(bot, handler)
        // Inject the shared counter state into the handler as a dependency.
        .dependencies(dptree::deps![messages_total])
        .enable_ctrlc_handler() // Allow Ctrl+C to stop the bot.
        .build()
        .dispatch()
        .await;
}
