//! # Heroku Ping-Pong Bot Example (Webhook)
//!
//! This example shows how to run a Telegram bot on **Heroku** using **webhooks** instead of long polling.
//!
//! When deployed, this bot will respond to every message with `"pong"`.
//!
//! ## Requirements
//!
//! 1. A Heroku account and the [Heroku CLI](https://devcenter.heroku.com/articles/heroku-cli) installed.
//! 2. Add the [Rust buildpack][1] to your Heroku app:
//!
//! ### For a new app:
//! ```bash
//! heroku create --buildpack emk/rust
//! ```
//!
//! ### For an existing app:
//! ```bash
//! heroku buildpacks:set emk/rust
//! ```
//!
//! 3. Set the required environment variables:
//!
//! ```bash
//! heroku config:set TELOXIDE_TOKEN=your_telegram_token
//! heroku config:set HOST=your_app_name.herokuapp.com
//! ```
//!
//! [1]: https://github.com/emk/heroku-buildpack-rust
//!
//! ## 📦 Deployment
//!
//! After setting everything up, push your code to Heroku:
//!
//! ```bash
//! git push heroku main
//! ```
//!
//! Then try messaging your bot on Telegram — it should reply with `"pong"`!

use std::env;

use teloxide::{prelude::*, update_listeners::webhooks};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting Heroku ping-pong bot...");

    let bot = Bot::from_env();

    // Heroku dynamically sets the port, so we read it from the environment
    let port: u16 = env::var("PORT")
        .expect("PORT env variable is not set")
        .parse()
        .expect("PORT env variable value is not an integer");

    let addr = ([0, 0, 0, 0], port).into();

    // Heroku host example: "heroku-ping-pong-bot.herokuapp.com"
    let host = env::var("HOST").expect("HOST env variable is not set");
    let url = format!("https://{host}/webhook").parse().unwrap();

    // Set up an HTTPS listener on Heroku using Axum + webhook configuration
    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook");

    // Handle updates by replying with "pong"
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
