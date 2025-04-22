//! # Ngrok Ping-Pong Bot Example (Webhook)
//!
//! This example shows how to run a Telegram bot locally using **ngrok** to expose your bot
//! to the internet and receive updates via **webhook** instead of long polling.
//!
//! When a message is sent to your bot, it replies with `"pong"`.
//!
//! ## Setup Instructions
//!
//! 1. Start ngrok in a terminal (you'll need [ngrok](https://ngrok.com) installed):
//!
//! ```bash
//! ngrok http 8443
//! ```
//!
//! This will give you a public HTTPS URL like `https://fancy-duck-1234.ngrok.io`.
//!
//! 2. Set that URL in the `url` field in this example code (with `/webhook` appended).
//!
//! ```rust
//! let url = "https://fancy-duck-1234.ngrok.io/webhook".parse().unwrap();
//! ```
//!
//! Now go to Telegram and message your bot — it should respond with `"pong"`!
//!
//! ## Notes
//!
//! - Use `127.0.0.1:8443` as the local address — it’s what ngrok will tunnel to.
//! - Make sure the URL is HTTPS — Telegram requires secure webhook endpoints.

use teloxide::{prelude::*, update_listeners::webhooks};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting ngrok ping-pong bot...");

    let bot = Bot::from_env();

    // Local address to bind the webhook server to (this is what ngrok tunnels).
    let addr = ([127, 0, 0, 1], 8443).into();

    // Replace this with the public HTTPS ngrok URL, e.g., https://abc123.ngrok.io/webhook
    let url = "Your HTTPS ngrok URL here. Get it by `ngrok http 8443`"
    .parse()
    .unwrap();

    // Set up the webhook listener with Axum + provided ngrok URL.
    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook");
    
    // Use a simple handler that replies "pong" to every message.
    teloxide::repl_with_listener(
        bot,
        |bot: Bot, msg: Message| async move {
            bot.send_message(msg.chat.id, "pong").await?;
            Ok(())
        },
        listener,
    )
    .await;
}
