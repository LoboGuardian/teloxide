//! # Dialogue Bot Example – "3 Questions Bot"
//!
//! This bot walks the user through a simple 3-step dialogue:
//!
//! ```text
//! User: Hey
//! Bot: Let's start! What's your full name?
//! User: Gandalf the Grey
//! Bot: How old are you?
//! User: 223
//! Bot: What's your location?
//! User: Middle-earth
//! Bot: Full name: Gandalf the Grey
//!      Age: 223
//!      Location: Middle-earth
//! ```
//!
//! It uses `teloxide`’s `Dialogue` system with in-memory storage (`InMemStorage`).
//! Each step is a separate state, and transitions are handled based on user input.
//!
//! ## Features Demonstrated
//!
//! - Multi-step conversation (dialogue FSM)  
//! - Typed dialogue states using enums  
//! - State transitions and validation (e.g., age must be a number)  
//! - Final message summary + dialogue exit

use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

/// Type alias for cleaner handler signatures.
type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Enum representing each step/state in the dialogue.
#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    /// Waiting for full name.
    ReceiveFullName,
    /// Waiting for age, after getting full name.
    ReceiveAge {
        full_name: String,
    },
    /// Waiting for location, after getting full name and age.
    ReceiveLocation {
        full_name: String,
        age: u8,
    },
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::ReceiveFullName].endpoint(receive_full_name))
            .branch(dptree::case![State::ReceiveAge { full_name }].endpoint(receive_age))
            .branch(
                dptree::case![State::ReceiveLocation { full_name, age }].endpoint(receive_location),
            ),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

/// First message — starts the dialogue.
async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's start! What's your full name?").await?;
    dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}

/// Receives full name from user and transitions to age input.
async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "How old are you?").await?;
            dialogue.update(State::ReceiveAge { full_name: text.into() }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }

    Ok(())
}

/// Receives age and moves on to ask for location.
async fn receive_age(
    bot: Bot,
    dialogue: MyDialogue,
    full_name: String, // Available from `State::ReceiveAge`.
    msg: Message,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<u8>()) {
        Some(Ok(age)) => {
            bot.send_message(msg.chat.id, "What's your location?").await?;
            dialogue.update(State::ReceiveLocation { full_name, age }).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Send me a number.").await?;
        }
    }

    Ok(())
}

/// Final step: collects location, summarizes everything, and ends the dialogue.
async fn receive_location(
    bot: Bot,
    dialogue: MyDialogue,
    (full_name, age): (String, u8), // Available from `State::ReceiveLocation`.
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(location) => {
            let report = format!("Full name: {full_name}\nAge: {age}\nLocation: {location}");
            bot.send_message(msg.chat.id, report).await?;
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }

    Ok(())
}
