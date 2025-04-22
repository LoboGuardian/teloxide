//! # DB Remember Bot Example
//!
//! This bot remembers a number you send it and allows you to `/get` or `/reset` it later,
//! even after the bot restarts, thanks to persistent **dialogue storage**.
//!
//! It supports both **Redis** and **SQLite** backends:
//!
//! - If the environment variable `DB_REMEMBER_REDIS` is set → Redis is used.
//! - Otherwise, it defaults to SQLite (`db.sqlite` file).
//!
//! ## Features Demonstrated
//!
//! - Stateful dialogues with persistent storage  
//! - Redis and SQLite storage backends  
//! - Command parsing (`/get`, `/reset`) inside dialogue states  
//! - Input validation with state transitions
//!
//! ## Running the bot
//!
//! ### With SQLite (default)
//! ```bash
//! cargo run --bin db_remember
//! ```
//!
//! ### With Redis
//! ```bash
//! export DB_REMEMBER_REDIS=1
//! cargo run --bin db_remember
//! ```

use teloxide::{
    dispatching::dialogue::{
        serializer::{Bincode, Json},
        ErasedStorage, RedisStorage, SqliteStorage, Storage,
    },
    prelude::*,
    utils::command::BotCommands,
};

/// Alias for dialogue state and storage types.
type MyDialogue = Dialogue<State, ErasedStorage<State>>;
type MyStorage = std::sync::Arc<ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Dialogue states, persisted across sessions.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum State {
    /// Initial state. User hasn't sent a number yet.
    #[default]
    Start,
    /// User sent a number. Stored here until reset.
    GotNumber(i32),
}

/// Commands allowed while in `GotNumber` state.
#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    /// Get the stored (your) number.
    Get,
    /// Reset the stored (your) number.
    Reset,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting DB remember bot...");

    let bot = Bot::from_env();

    // Choose Redis or SQLite depending on the environment.
    let storage: MyStorage = if std::env::var("DB_REMEMBER_REDIS").is_ok() {
        RedisStorage::open("redis://127.0.0.1:6379", Bincode).await.unwrap().erase()
    } else {
        SqliteStorage::open("db.sqlite", Json).await.unwrap().erase()
    };

    // Define how we handle updates (in this case, only messages).
    let handler = Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<State>, State>()
        .branch(dptree::case![State::Start].endpoint(start))
        .branch(
            dptree::case![State::GotNumber(n)]
                .branch(dptree::entry().filter_command::<Command>().endpoint(got_number))
                .branch(dptree::endpoint(invalid_command)),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

/// Initial handler: waits for a number.
async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<i32>()) {
        Some(Ok(n)) => {
            dialogue.update(State::GotNumber(n)).await?;
            bot.send_message(
                msg.chat.id,
                format!("Remembered number {n}. Now use /get or /reset."),
            )
            .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Please, send me a number.").await?;
        }
    }

    Ok(())
}

/// Handles `/get` and `/reset` after a number has been remembered.
async fn got_number(
    bot: Bot,
    dialogue: MyDialogue,
    num: i32, // Available from `State::GotNumber`.
    msg: Message,
    cmd: Command,
) -> HandlerResult {
    match cmd {
        Command::Get => {
            bot.send_message(msg.chat.id, format!("Here is your number: {num}.")).await?;
        }
        Command::Reset => {
            dialogue.reset().await?;
            bot.send_message(msg.chat.id, "Number reset.").await?;
        }
    }
    Ok(())
}

/// Handles text commands that are not recognized while in `GotNumber` state.
async fn invalid_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Please, send /get or /reset.").await?;
    Ok(())
}
