//! # Admin Bot Example (with `teloxide`)
//!
//! This example shows how to create a Telegram bot with admin capabilities like:
//! - Kicking users
//! - Temporarily banning users
//! - Muting users
//!
//! The commands require replying to a user's message to target them.
//!
//! ## Supported Commands
//!
//! - `/kick` — Kicks the replied user from the chat.
//! - `/ban <time> <unit>` — Bans the replied user for a period (e.g., `/ban 5 m`).
//! - `/mute <time> <unit>` — Mutes the replied user for a period.
//! - `/help` — Shows the list of available commands.
//!
//! ## Time Units
//!
//! - `s`, `seconds`
//! - `m`, `minutes`
//! - `h`, `hours`

use chrono::Duration;
use std::str::FromStr;
use teloxide::{prelude::*, types::ChatPermissions, utils::command::BotCommands};

// Derive BotCommands to parse text with a command into this enumeration.
//
// 1. `rename_rule = "lowercase"` turns all the commands into lowercase letters.
// 2. `description = "..."` specifies a text before all the commands.
//
// That is, you can just call Command::descriptions() to get a description of
// your commands in this format:
// %GENERAL-DESCRIPTION%
// %PREFIX%%COMMAND% - %DESCRIPTION%

/// Use commands in format /%command% %num% %unit%
/// Commands for the bot. Commands are parsed automatically from messages using `BotCommands`.
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", parse_with = "split")]
enum Command {
    /// Kick the replied user from the chat.
    Kick,
    /// Temporarily ban the replied user.
    /// Usage: /ban <time> <unit>
    Ban { time: u64, unit: UnitOfTime },
    /// Temporarily mute the replied user.
    /// Usage: /mute <time> <unit>
    Mute { time: u64, unit: UnitOfTime },
    /// Show available commands.
    Help,
}

/// Units of time accepted for mute/ban durations.
#[derive(Clone)]
enum UnitOfTime {
    Seconds,
    Minutes,
    Hours,
}

impl FromStr for UnitOfTime {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "h" | "hours" => Ok(UnitOfTime::Hours),
            "m" | "minutes" => Ok(UnitOfTime::Minutes),
            "s" | "seconds" => Ok(UnitOfTime::Seconds),
            _ => Err("Allowed units: h, m, s"),
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting admin bot...");

    let bot = teloxide::Bot::from_env();

    Command::repl(bot, action).await;
}

/// Handles each command from users.
async fn action(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Kick => kick_user(bot, msg).await?,
        Command::Ban { time, unit } => ban_user(bot, msg, calc_restrict_time(time, unit)).await?,
        Command::Mute { time, unit } => mute_user(bot, msg, calc_restrict_time(time, unit)).await?,
    };

    Ok(())
}

/// Kicks the replied user from the chat.
///
/// The command must be used in reply to another user's message.
async fn kick_user(bot: Bot, msg: Message) -> ResponseResult<()> {
    match msg.reply_to_message() {
        Some(replied) => {
            // bot.unban_chat_member can also kicks a user from a group chat.
            bot.unban_chat_member(msg.chat.id, replied.from.as_ref().unwrap().id).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Use this command in reply to another message").await?;
        }
    }
    Ok(())
}

/// Bans the replied user for a limited time.
async fn ban_user(bot: Bot, msg: Message, time: Duration) -> ResponseResult<()> {
    match msg.reply_to_message() {
        Some(replied) => {
            bot.kick_chat_member(
                msg.chat.id,
                replied.from.as_ref().expect("Must be MessageKind::Common").id,
            )
            .until_date(msg.date + time)
            .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Use this command in a reply to another message!")
                .await?;
        }
    }
    Ok(())
}

/// Mutes the replied user for a limited time.
async fn mute_user(bot: Bot, msg: Message, time: Duration) -> ResponseResult<()> {
    match msg.reply_to_message() {
        Some(replied) => {
            bot.restrict_chat_member(
                msg.chat.id,
                replied.from.as_ref().expect("Must be MessageKind::Common").id,
                ChatPermissions::empty(),
            )
            .until_date(msg.date + time)
            .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Use this command in a reply to another message!")
                .await?;
        }
    }
    Ok(())
}

/// Converts a time value and unit into a `chrono::Duration`.
// Calculates time of user restriction.
fn calc_restrict_time(time: u64, unit: UnitOfTime) -> Duration {
    // FIXME: actually handle the case of too big integers correctly,
    // instead of unwrapping
    // Note: this will panic on overflow. A proper implementation should handle large values safely.
    match unit {
        UnitOfTime::Hours => Duration::try_hours(time as i64).unwrap(),
        UnitOfTime::Minutes => Duration::try_minutes(time as i64).unwrap(),
        UnitOfTime::Seconds => Duration::try_seconds(time as i64).unwrap(),
    }
}
