//! # Purchase Bot Example
//!
//! This bot demonstrates how to manage **multi-step dialogues** in Telegram using `teloxide`.
//!
//! Users interact with the bot through messages and inline buttons.
//!
//! # Example Flow
//!
//! ```text
//! /start
//! → Let's start! What's your full name?
//! → John Doe
//! → Select a product:
//!   [Apple] [Banana] [Orange] [Potato]
//! → (User clicks Banana)
//! → John Doe, product 'Banana' has been purchased successfully!
//! ```
//!
//! ## Features Demonstrated
//!
//! - Command handling (`/start`, `/help`, `/cancel`)  
//! - Dialogue state management  
//! - Parsing messages and button clicks in the same flow  
//! - Replying with `InlineKeyboardMarkup`

use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

/// Alias for the dialogue type.
type MyDialogue = Dialogue<State, InMemStorage<State>>;

/// Standard handler result with error box.
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// All possible dialogue states.
#[derive(Clone, Default)]
pub enum State {
    /// Initial state before any command.
    #[default]
    Start,
    /// Waiting for the user's full name
    ReceiveFullName,
    /// Waiting for product selection after receiving full name.
    ReceiveProductChoice {
        full_name: String,
    },
}

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this help message..
    Help,
    /// Start the purchase procedure.
    Start,
    /// Cancel the purchase procedure.
    Cancel,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting purchase bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

/// Dialogue handler schema: routes commands, messages, and callback queries.
fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start)),
        )
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveFullName].endpoint(receive_full_name))
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query().branch(
        case![State::ReceiveProductChoice { full_name }].endpoint(receive_product_selection),
    );

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

/// Starts the purchase dialogue.
async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's start! What's your full name?").await?;
    dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}

/// Shows command help text.
async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

/// Cancels the ongoing dialogue.
async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Cancelling the dialogue.").await?;
    dialogue.exit().await?;
    Ok(())
}

/// Default response for unexpected input
async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Unable to handle the message. Type /help to see the usage.")
        .await?;
    Ok(())
}

/// Handles receiving a user's full name.
async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(full_name) => {
            // Create a row of buttons with Debian product names.
            let products = ["Apple", "Banana", "Orange", "Potato"]
                .map(|product| InlineKeyboardButton::callback(product, product));

            bot.send_message(msg.chat.id, "Select a product:")
                .reply_markup(InlineKeyboardMarkup::new([products]))
                .await?;

            dialogue.update(State::ReceiveProductChoice { full_name }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me your full name.").await?;
        }
    }

    Ok(())
}

/// Handles when the user selects a product via button.
async fn receive_product_selection(
    bot: Bot,
    dialogue: MyDialogue,
    full_name: String, // Available from `State::ReceiveProductChoice`.
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(product) = &q.data {
        bot.send_message(
            dialogue.chat_id(),
            format!("{full_name}, product '{product}' has been purchased successfully!"),
        )
        .await?;
        dialogue.exit().await?;
    }

    Ok(())
}
