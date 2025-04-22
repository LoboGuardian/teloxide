//! # Middlewares (Fallible) Bot Example
//!
//! This example demonstrates **middleware chaining with fallible logic** using `teloxide`.
//!
//! You’ll see how to:
//!
//! - Run logic *before* the main handler.
//! - Handle potential *errors* *after* the handler.
//! - Conditionally decide *whether to proceed* or stop.
//!
//! ## Flow Summary
//!
//! ```text
//! Before (message #ID)
//! Inside the endpoint.
//! In-between (message #ID)
//! After (message #ID)      <-- Only if no error!
//! ```
//!
//! ## If the endpoint fails:
//!
//! ```text
//! Before (message #ID)
//! Inside the endpoint.
//! In-between (message #ID)
//! Our endpoint failed: ...
//! (No "After" message printed)
//! ```

use teloxide::prelude::*;

/// Our convenient result alias.
type HandlerResult = Result<(), teloxide::RequestError>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting middlewares_fallible bot...");

    let bot = Bot::from_env();

    // Our middleware pipeline
    let handler = Update::filter_message()

        // Executes before the endpoint.
        // Pre-endpoint logic (logging).
        .inspect(|msg: Message| println!("Before (message #{}).", msg.id))

        // Our "endpoint".
        // Actual business logic (runs asynchronously).
        .map_async(my_endpoint)

        // Executes after the endpoint. If the endpoint failed, print an error message and stop
        // execution; if not, continue.
        // In-between logic that *filters* the flow based on the result of the endpoint.
        .filter(|result: HandlerResult, msg: Message| {
            println!("In-between (message #{}).", msg.id);
            match result {
                Ok(()) => true, // Continue to next stage.
                Err(err) => {
                    eprintln!("Our endpoint failed: {err}");
                    false // Continue to next stage.
                }
            }
        })

        // Executes only if the endpoint succeeded.
        // Final stage — only reached if previous steps succeeded.
        .endpoint(|msg: Message| async move {
            println!("After (message #{}).", msg.id);
            HandlerResult::Ok(())
        });
    
    // Build and launch the dispatcher.
    Dispatcher::builder(bot, handler)
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

/// Simulated "business logic".
async fn my_endpoint(bot: Bot, msg: Message) -> HandlerResult {
    // To simulate an error, you can change this line to:
    // return Err("Simulated error".into());

    bot.send_message(msg.chat.id, "Inside the endpoint.").await?;
    
    Ok(())
}
