// AI module - Azure OpenAI integration

mod azure_openai;
mod models;
mod prompts;

pub use azure_openai::AzureOpenAIClient;
pub use models::{Model, Settings};
pub use prompts::SystemPrompts;
