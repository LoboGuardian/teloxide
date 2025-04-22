//! # Dispatching Features Overview Bot
//!
//! This advanced example demonstrates how to use the updated `dispatching` module in `teloxide`.
//!
//! It supports multiple branching handlers with contextual filtering:
//!
//! - **Simple commands** like `/help`, `/myid`
//! - **Maintainer-only commands** like `/rand <from> <to>`
//! - **Group-only commands** with bot mentions (e.g. `/repeat@your_bot`)
//! - **Special handler for dice messages**
//!
//! ## Maintainer-only access
//!
//! Set your Telegram user ID here:
//!
//! ```rust
//! bot_maintainer: UserId(0) // replace 0 with your ID
//! ```

use rand::Rng;

use teloxide::{
    dispatching::HandlerExt, prelude::*, sugar::request::RequestReplyExt, types::Dice,
    utils::command::BotCommands,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting dispatching features bot...");

    let bot = Bot::from_env();

    // Custom configuration passed into handlers as a dependency.
    let parameters = ConfigParameters {
        bot_maintainer: UserId(0), // ← Replace this with your actual Telegram user ID!
        maintainer_username: None,
    };

    // Main dispatcher handler tree
    let handler = Update::filter_message()
        // === First Branch: simple public commands ===
        // You can use branching to define multiple ways in which an update will be handled. If the
        // first branch fails, an update will be passed to the second branch, and so on.
        .branch(
            dptree::entry()
                // Filter commands: the next handlers will receive a parsed `SimpleCommand`.
                .filter_command::<SimpleCommand>()
                // If a command parsing fails, this handler will not be executed.
                .endpoint(simple_commands_handler),
        )
        // === Second Branch: maintainer-only commands ===
        .branch(
            // Filter a maintainer by a user ID.
            dptree::filter(|cfg: ConfigParameters, msg: Message| {
                msg.from.map(|user| user.id == cfg.bot_maintainer).unwrap_or_default()
            })
            .filter_command::<MaintainerCommands>()
            .endpoint(|msg: Message, bot: Bot, cmd: MaintainerCommands| async move {
                match cmd {
                    MaintainerCommands::Rand { from, to } => {
                        let mut rng = rand::rngs::OsRng;
                        let value: u64 = rng.gen_range(from..=to);

                        bot.send_message(msg.chat.id, value.to_string()).await?;
                        Ok(())
                    }
                }
            }),
        )
        // === Third Branch: only for group messages ===
        .branch(
            // Filtering allow you to filter updates by some condition.
            dptree::filter(|msg: Message| msg.chat.is_group() || msg.chat.is_supergroup())
                .branch(
                    // Filtering by mention allows to filter only `/repeat@my_bot` commands.
                    // Use if you want to make sure that users refer specifically to your bot.
                    // Same as filter_command, the next handlers will receive a parsed
                    // `GroupCommand`.
                    dptree::entry().filter_mention_command::<GroupCommand>().endpoint(
                        |bot: Bot, msg: Message, cmd: GroupCommand| async move {
                            match cmd {
                                GroupCommand::Repeat { text } => {
                                    bot.send_message(msg.chat.id, format!("You said: {text}"))
                                        .await?;
                                    Ok(())
                                }
                            }
                        },
                    ),
                )
                .branch(
                    // An endpoint is the last update handler.
                    dptree::endpoint(|msg: Message, bot: Bot| async move {
                        log::info!("Received a message from a group chat.");
                        bot.send_message(msg.chat.id, "This is a group chat.").await?;
                        respond(())
                    }),
                ),
        )
        // === Fourth Branch: special case for dice messages ===
        .branch(
            // There are some extension filtering functions on `Message`. The following filter will
            // filter only messages with dices.
            Message::filter_dice().endpoint(|bot: Bot, msg: Message, dice: Dice| async move {
                bot.send_message(msg.chat.id, format!("Dice value: {}", dice.value))
                    .reply_to(msg)
                    .await?;
                Ok(())
            }),
        );

    Dispatcher::builder(bot, handler)
        // Pass configuration and dependencies into handlers
        //
        // Here you specify initial dependencies that all handlers will receive; they can be
        // database connections, configurations, and other auxiliary arguments. It is similar to
        // `actix_web::Extensions`.
        .dependencies(dptree::deps![parameters])
        // Catch unhandled updates
        //
        // If no handler succeeded to handle an update, this closure will be called.
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
        })
        // Catch global errors
        //
        // If the dispatcher fails for some reason, execute this handler.
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

// === Global Configuration Struct ===
#[derive(Clone)]
struct ConfigParameters {
    bot_maintainer: UserId,
    maintainer_username: Option<String>,
}

// === Simple User Commands ===
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum SimpleCommand {
    /// Shows this help message.
    Help,
    /// Shows who the bot maintainer is.
    Maintainer,
    /// Shows your Telegram ID.
    MyId,
}

/// Shows your Telegram ID.
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum MaintainerCommands {
    /// Generate a random number in range
    #[command(parse_with = "split")]
    Rand { from: u64, to: u64 },
}

// === Group-only Commands ===
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum GroupCommand {
    /// Repeats a message
    Repeat { text: String },
}

// === Handler for Simple Commands ===
async fn simple_commands_handler(
    cfg: ConfigParameters,
    bot: Bot,
    me: teloxide::types::Me,
    msg: Message,
    cmd: SimpleCommand,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        SimpleCommand::Help => {
            if msg.from.unwrap().id == cfg.bot_maintainer {
                format!(
                    "{}\n\n{}",
                    SimpleCommand::descriptions(),
                    MaintainerCommands::descriptions()
                )
            } else if msg.chat.is_group() || msg.chat.is_supergroup() {
                SimpleCommand::descriptions().username_from_me(&me).to_string()
            } else {
                SimpleCommand::descriptions().to_string()
            }
        }
        SimpleCommand::Maintainer => {
            if msg.from.as_ref().unwrap().id == cfg.bot_maintainer {
                "Maintainer is you!".into()
            } else if let Some(username) = cfg.maintainer_username {
                format!("Maintainer is @{username}")
            } else {
                format!("Maintainer ID is {}", cfg.bot_maintainer)
            }
        }
        SimpleCommand::MyId => {
            format!("{}", msg.from.unwrap().id)
        }
    };

    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}
