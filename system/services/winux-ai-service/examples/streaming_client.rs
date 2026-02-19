//! Streaming client example for winux-ai-service
//!
//! This example demonstrates how to use streaming responses from the AI service.
//!
//! Run with: cargo run --example streaming_client

use anyhow::Result;
use futures::StreamExt;
use zbus::{proxy, Connection, MatchRule, MessageStream};
use std::collections::HashMap;
use std::io::{self, Write};

/// D-Bus proxy for the AI service
#[proxy(
    interface = "com.winux.AI",
    default_service = "com.winux.AI",
    default_path = "/com/winux/AI"
)]
trait AIService {
    /// Start streaming chat
    async fn chat_stream(&self, messages: Vec<(String, String)>, model: &str) -> zbus::Result<String>;
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Winux AI Service Streaming Client Example\n");

    // Connect to the system bus
    let connection = Connection::system().await?;

    // Create a proxy to the AI service
    let proxy = AIServiceProxy::new(&connection).await?;

    // Subscribe to streaming signals
    let rule = MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .sender("com.winux.AI")?
        .interface("com.winux.AI")?
        .member("StreamingResponse")?
        .build();

    let mut stream = MessageStream::for_match_rule(rule, &connection, None).await?;

    // Start a streaming chat
    let messages = vec![
        ("system".to_string(), "You are a helpful assistant. Respond in a conversational way.".to_string()),
        ("user".to_string(), "Tell me a short story about a robot learning to paint.".to_string()),
    ];

    println!("Starting streaming chat...\n");
    let request_id = proxy.chat_stream(messages, "gpt-4o").await?;
    println!("Request ID: {}\n", request_id);

    print!("Response: ");
    io::stdout().flush()?;

    // Collect streaming responses
    let mut done = false;
    let mut full_response = String::new();

    while !done {
        if let Some(msg) = stream.next().await {
            match msg {
                Ok(msg) => {
                    // Parse signal arguments
                    let body = msg.body();
                    if let Ok((rid, chunk, is_done)) = body.deserialize::<(String, String, bool)>() {
                        if rid == request_id {
                            print!("{}", chunk);
                            io::stdout().flush()?;
                            full_response.push_str(&chunk);
                            done = is_done;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\nError receiving signal: {}", e);
                    break;
                }
            }
        }
    }

    println!("\n\n--- Stream completed ---");
    println!("Total characters received: {}", full_response.len());

    Ok(())
}
