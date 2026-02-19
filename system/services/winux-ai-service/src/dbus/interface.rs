//! D-Bus interface implementation for com.winux.AI

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use zbus::{interface, Connection, SignalContext};

use crate::api::{AzureOpenAIClient, RateLimiter};
use crate::cache::ResponseCache;
use crate::features::{ChatFeature, CodeFeature, CompleteFeature, SummarizeFeature, TranslateFeature, VisionFeature};

/// AI Service D-Bus interface
pub struct AIService {
    complete: Arc<CompleteFeature>,
    chat: Arc<ChatFeature>,
    summarize: Arc<SummarizeFeature>,
    translate: Arc<TranslateFeature>,
    code: Arc<CodeFeature>,
    vision: Arc<VisionFeature>,
    connection: Arc<RwLock<Option<Connection>>>,
}

impl AIService {
    /// Create a new AI service instance
    pub fn new(
        client: Arc<AzureOpenAIClient>,
        cache: Arc<ResponseCache>,
        rate_limiter: Arc<RateLimiter>,
    ) -> Self {
        let complete = Arc::new(CompleteFeature::new(
            client.clone(),
            cache.clone(),
            rate_limiter.clone(),
        ));

        let chat = Arc::new(ChatFeature::new(
            client.clone(),
            cache.clone(),
            rate_limiter.clone(),
        ));

        let summarize = Arc::new(SummarizeFeature::new(
            client.clone(),
            cache.clone(),
            rate_limiter.clone(),
        ));

        let translate = Arc::new(TranslateFeature::new(
            client.clone(),
            cache.clone(),
            rate_limiter.clone(),
        ));

        let code = Arc::new(CodeFeature::new(
            client.clone(),
            cache.clone(),
            rate_limiter.clone(),
        ));

        let vision = Arc::new(VisionFeature::new(
            client,
            cache,
            rate_limiter,
        ));

        Self {
            complete,
            chat,
            summarize,
            translate,
            code,
            vision,
            connection: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the D-Bus connection for emitting signals
    pub async fn set_connection(&self, conn: Connection) {
        let mut connection = self.connection.write().await;
        *connection = Some(conn);
    }

    /// Emit a streaming response signal
    async fn emit_streaming_signal(
        &self,
        ctxt: &SignalContext<'_>,
        request_id: &str,
        chunk: &str,
        done: bool,
    ) -> zbus::Result<()> {
        Self::streaming_response(ctxt, request_id, chunk, done).await
    }
}

#[interface(name = "com.winux.AI")]
impl AIService {
    /// Complete text based on a prompt
    ///
    /// # Arguments
    /// * `prompt` - The text prompt to complete
    /// * `model` - The model to use (e.g., "gpt-4o")
    ///
    /// # Returns
    /// The completed text response
    async fn complete(&self, prompt: &str, model: &str) -> zbus::fdo::Result<String> {
        tracing::info!("D-Bus: Complete request received");

        self.complete
            .complete(prompt, model)
            .await
            .map_err(|e| {
                tracing::error!("Complete error: {}", e);
                zbus::fdo::Error::Failed(e.to_string())
            })
    }

    /// Chat with message history
    ///
    /// # Arguments
    /// * `messages` - Array of (role, content) tuples
    /// * `model` - The model to use
    ///
    /// # Returns
    /// The assistant's response
    async fn chat(&self, messages: Vec<(String, String)>, model: &str) -> zbus::fdo::Result<String> {
        tracing::info!("D-Bus: Chat request received with {} messages", messages.len());

        self.chat
            .chat(messages, model)
            .await
            .map_err(|e| {
                tracing::error!("Chat error: {}", e);
                zbus::fdo::Error::Failed(e.to_string())
            })
    }

    /// Chat with streaming response
    ///
    /// # Arguments
    /// * `messages` - Array of (role, content) tuples
    /// * `model` - The model to use
    ///
    /// # Returns
    /// A request ID to correlate with StreamingResponse signals
    async fn chat_stream(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        messages: Vec<(String, String)>,
        model: &str,
    ) -> zbus::fdo::Result<String> {
        let request_id = Uuid::new_v4().to_string();
        tracing::info!("D-Bus: Streaming chat request {} received", request_id);

        let chat = self.chat.clone();
        let rid = request_id.clone();

        // Spawn task to handle streaming
        let ctxt_owned = ctxt.to_owned();
        tokio::spawn(async move {
            match chat.chat_stream(messages, model, rid.clone()).await {
                Ok(mut rx) => {
                    while let Some(chunk) = rx.recv().await {
                        if let Err(e) = AIService::streaming_response(
                            &ctxt_owned,
                            &chunk.request_id,
                            &chunk.content,
                            chunk.done,
                        ).await {
                            tracing::error!("Failed to emit streaming signal: {}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Streaming chat error: {}", e);
                    let _ = AIService::streaming_response(
                        &ctxt_owned,
                        &rid,
                        &format!("Error: {}", e),
                        true,
                    ).await;
                }
            }
        });

        Ok(request_id)
    }

    /// Summarize text
    ///
    /// # Arguments
    /// * `text` - The text to summarize
    ///
    /// # Returns
    /// A concise summary of the text
    async fn summarize(&self, text: &str) -> zbus::fdo::Result<String> {
        tracing::info!("D-Bus: Summarize request received");

        self.summarize
            .summarize(text)
            .await
            .map_err(|e| {
                tracing::error!("Summarize error: {}", e);
                zbus::fdo::Error::Failed(e.to_string())
            })
    }

    /// Translate text between languages
    ///
    /// # Arguments
    /// * `text` - The text to translate
    /// * `from_lang` - Source language code (or "auto" for detection)
    /// * `to_lang` - Target language code
    ///
    /// # Returns
    /// The translated text
    async fn translate(
        &self,
        text: &str,
        from_lang: &str,
        to_lang: &str,
    ) -> zbus::fdo::Result<String> {
        tracing::info!("D-Bus: Translate request received ({} -> {})", from_lang, to_lang);

        self.translate
            .translate(text, from_lang, to_lang)
            .await
            .map_err(|e| {
                tracing::error!("Translate error: {}", e);
                zbus::fdo::Error::Failed(e.to_string())
            })
    }

    /// Analyze code
    ///
    /// # Arguments
    /// * `code` - The code to analyze
    /// * `language` - The programming language
    ///
    /// # Returns
    /// Analysis of the code including issues and suggestions
    async fn analyze_code(&self, code: &str, language: &str) -> zbus::fdo::Result<String> {
        tracing::info!("D-Bus: AnalyzeCode request received for {} code", language);

        self.code
            .analyze(code, language)
            .await
            .map_err(|e| {
                tracing::error!("AnalyzeCode error: {}", e);
                zbus::fdo::Error::Failed(e.to_string())
            })
    }

    /// Analyze an image
    ///
    /// # Arguments
    /// * `image_path` - Path to the image file
    /// * `prompt` - What to analyze about the image
    ///
    /// # Returns
    /// Description or analysis of the image
    async fn analyze_image(&self, image_path: &str, prompt: &str) -> zbus::fdo::Result<String> {
        tracing::info!("D-Bus: AnalyzeImage request received for: {}", image_path);

        self.vision
            .analyze_image(image_path, prompt)
            .await
            .map_err(|e| {
                tracing::error!("AnalyzeImage error: {}", e);
                zbus::fdo::Error::Failed(e.to_string())
            })
    }

    /// Get service version
    async fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Check if service is healthy
    async fn health_check(&self) -> bool {
        true
    }

    /// Streaming response signal
    ///
    /// Emitted for each chunk of a streaming response
    #[zbus(signal)]
    async fn streaming_response(
        ctxt: &SignalContext<'_>,
        request_id: &str,
        chunk: &str,
        done: bool,
    ) -> zbus::Result<()>;
}

/// D-Bus service name
pub const SERVICE_NAME: &str = "com.winux.AI";

/// D-Bus object path
pub const OBJECT_PATH: &str = "/com/winux/AI";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_constants() {
        assert_eq!(SERVICE_NAME, "com.winux.AI");
        assert_eq!(OBJECT_PATH, "/com/winux/AI");
    }
}
