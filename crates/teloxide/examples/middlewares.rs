//! # Middlewares Bot Example
//!
//! This bot demonstrates how to use `inspect`, `map_async`, and other middleware-style
//! functions in a `teloxide` dispatcher.
//!
//! It's useful for **logging**, **monitoring**, or even **modifying updates** before
//! or after they hit your actual handler (`endpoint`).
//!
//! ## Execution Flow
//!
//! ```text
//! Before (message #ID)       <- inspect (pre-endpoint)
//! Inside the endpoint.       <- your actual bot logic
//! After (message #ID)        <- inspect (post-endpoint)
//! ```

use teloxide::prelude::*;

/// Convenient alias for handler result type.
type HandlerResult = Result<(), teloxide::RequestError>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting middlewares bot...");

    let bot = Bot::from_env();

    // This handler demonstrates middleware-style chaining:
    let handler = Update::filter_message()

        // Executes before the endpoint.
        // Pre-endpoint middleware: logs before endpoint logic runs.
        .inspect(|msg: Message| println!("Before (message #{}).", msg.id))

        // Our "endpoint".
        // Transforms the update into a Result by running your logic.
        .map_async(my_endpoint)

        // Executes after the endpoint
        // Post-endpoint middleware: always runs, even on failure.
        .inspect(|msg: Message| {
            println!("After (message #{}).", msg.id);
        })

        // Retrieve the result of the endpoint and pass it to the dispatcher.
        // This is the final endpoint that `teloxide` expects.
        .endpoint(|result: HandlerResult| async move {
            // Alternatively, we could also pattern-match on this value for more
            // fine-grained behaviour.
            result
        });

    // Build and run the dispatcher with our custom middleware chain.
    Dispatcher::builder(bot, handler)
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

/// Actual business logic for incoming messages.
async fn my_endpoint(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Inside the endpoint.").await?;
    
    Ok(())
}
