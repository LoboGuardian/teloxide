//! # 🧾 Command Bot Example
//!
//! This bot demonstrates how to use **strongly typed bot commands** with `teloxide`.
//!
//! Instead of manually parsing command strings, you can define your commands as an `enum` and derive `BotCommands`.
//! `teloxide` will do the parsing for you automatically — including aliases and structured parameters.
//!
//! ## Supported Commands
//!
//! - `/help` or `/h` or `/?` – shows available commands
//! - `/username <your username>` or `/u <your username>`
//! - `/usernameandage <username> <age>` or `/ua <username> <age>`
//!
//! Then send a command to your bot on Telegram like:
//!
//! ```text
//! /username ferris_the_crab
//! ```

use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() {

    // Initialize logger for debug/info output.
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    // Load the bot token from environment variable.
    let bot = Bot::from_env();

    // Start the command loop — every command triggers `answer`.
    Command::repl(bot, answer).await;
}

/// Supported commands for this bot.
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this help message.
    #[command(aliases = ["h", "?"])]
    Help,

    /// Handle a username.
    /// Usage: /username <your_name>
    #[command(alias = "u")]
    Username(String),

    /// Handle a username and an age.
    /// Usage: /usernameandage <your_name> <your_age>
    #[command(parse_with = "split", alias = "ua", hide_aliases)]
    UsernameAndAge { username: String, age: u8 },
}

/// Responds to parsed commands.
async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            // Show all command descriptions
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        }
        Command::Username(username) => {
            // Respond with the parsed username
            bot.send_message(msg.chat.id, format!("Your username is @{username}.")).await?
        }
        Command::UsernameAndAge { username, age } => {
            // Respond with both username and age
            bot.send_message(msg.chat.id, format!("Your username is @{username} and age is {age}."))
                .await?
        }
    };

    Ok(())
}
