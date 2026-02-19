//! Example client for winux-ai-service
//!
//! This example demonstrates how to use the AI service from other applications.
//!
//! Run with: cargo run --example client

use anyhow::Result;
use zbus::{proxy, Connection};

/// D-Bus proxy for the AI service
#[proxy(
    interface = "com.winux.AI",
    default_service = "com.winux.AI",
    default_path = "/com/winux/AI"
)]
trait AIService {
    /// Complete text
    async fn complete(&self, prompt: &str, model: &str) -> zbus::Result<String>;

    /// Chat with messages
    async fn chat(&self, messages: Vec<(String, String)>, model: &str) -> zbus::Result<String>;

    /// Summarize text
    async fn summarize(&self, text: &str) -> zbus::Result<String>;

    /// Translate text
    async fn translate(&self, text: &str, from_lang: &str, to_lang: &str) -> zbus::Result<String>;

    /// Analyze code
    async fn analyze_code(&self, code: &str, language: &str) -> zbus::Result<String>;

    /// Analyze image
    async fn analyze_image(&self, image_path: &str, prompt: &str) -> zbus::Result<String>;

    /// Get version
    async fn version(&self) -> zbus::Result<String>;

    /// Health check
    async fn health_check(&self) -> zbus::Result<bool>;

    /// Start streaming chat
    async fn chat_stream(&self, messages: Vec<(String, String)>, model: &str) -> zbus::Result<String>;

    /// Streaming response signal
    #[zbus(signal)]
    async fn streaming_response(request_id: &str, chunk: &str, done: bool) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Winux AI Service Client Example\n");

    // Connect to the system bus
    let connection = Connection::system().await?;

    // Create a proxy to the AI service
    let proxy = AIServiceProxy::new(&connection).await?;

    // Check service health
    println!("Checking service health...");
    let healthy = proxy.health_check().await?;
    println!("Service healthy: {}\n", healthy);

    // Get version
    let version = proxy.version().await?;
    println!("Service version: {}\n", version);

    // Example: Text completion
    println!("=== Text Completion ===");
    let prompt = "Explain what a Linux distribution is in one sentence:";
    println!("Prompt: {}", prompt);
    let response = proxy.complete(prompt, "gpt-4o").await?;
    println!("Response: {}\n", response);

    // Example: Chat
    println!("=== Chat ===");
    let messages = vec![
        ("system".to_string(), "You are a helpful assistant.".to_string()),
        ("user".to_string(), "What is Winux?".to_string()),
    ];
    println!("Messages: {:?}", messages);
    let response = proxy.chat(messages, "gpt-4o").await?;
    println!("Response: {}\n", response);

    // Example: Summarization
    println!("=== Summarization ===");
    let text = "Artificial intelligence (AI) is intelligence demonstrated by machines, \
                as opposed to natural intelligence displayed by animals including humans. \
                AI research has been defined as the field of study of intelligent agents, \
                which refers to any system that perceives its environment and takes actions \
                that maximize its chance of achieving its goals.";
    println!("Text: {}...", &text[..50]);
    let summary = proxy.summarize(text).await?;
    println!("Summary: {}\n", summary);

    // Example: Translation
    println!("=== Translation ===");
    let text = "Hello, how are you today?";
    println!("English: {}", text);
    let translation = proxy.translate(text, "en", "pt").await?;
    println!("Portuguese: {}\n", translation);

    // Example: Code Analysis
    println!("=== Code Analysis ===");
    let code = r#"
fn main() {
    let x = vec![1, 2, 3];
    for i in 0..10 {
        println!("{}", x[i]);
    }
}
"#;
    println!("Code:\n{}", code);
    let analysis = proxy.analyze_code(code, "rust").await?;
    println!("Analysis: {}\n", analysis);

    println!("All examples completed successfully!");

    Ok(())
}
