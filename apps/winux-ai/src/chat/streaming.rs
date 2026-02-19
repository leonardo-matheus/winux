// Streaming response handling for real-time AI responses

use serde::Deserialize;
use tokio::sync::mpsc;

#[derive(Debug, Deserialize)]
pub struct StreamingChunk {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<i64>,
    pub model: Option<String>,
    pub choices: Vec<StreamingChoice>,
}

#[derive(Debug, Deserialize)]
pub struct StreamingChoice {
    pub index: usize,
    pub delta: StreamingDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StreamingDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}

pub struct StreamingResponse {
    receiver: mpsc::Receiver<String>,
}

impl StreamingResponse {
    pub fn new(receiver: mpsc::Receiver<String>) -> Self {
        Self { receiver }
    }

    pub async fn recv(&mut self) -> Option<String> {
        self.receiver.recv().await
    }
}

/// Parse Server-Sent Events (SSE) data line
pub fn parse_sse_line(line: &str) -> Option<String> {
    // SSE format: "data: {...json...}"
    if let Some(json_str) = line.strip_prefix("data: ") {
        if json_str == "[DONE]" {
            return None;
        }

        if let Ok(chunk) = serde_json::from_str::<StreamingChunk>(json_str) {
            if let Some(choice) = chunk.choices.first() {
                if let Some(content) = &choice.delta.content {
                    return Some(content.clone());
                }
            }
        }
    }
    None
}

/// Stream handler that processes SSE events and sends content to UI
pub struct StreamHandler {
    sender: mpsc::Sender<String>,
    buffer: String,
}

impl StreamHandler {
    pub fn new(sender: mpsc::Sender<String>) -> Self {
        Self {
            sender,
            buffer: String::new(),
        }
    }

    /// Process incoming bytes from the HTTP stream
    pub async fn process_bytes(&mut self, bytes: &[u8]) -> Result<(), anyhow::Error> {
        let text = String::from_utf8_lossy(bytes);
        self.buffer.push_str(&text);

        // Process complete lines
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer[..newline_pos].trim().to_string();
            self.buffer = self.buffer[newline_pos + 1..].to_string();

            if !line.is_empty() {
                if let Some(content) = parse_sse_line(&line) {
                    self.sender.send(content).await?;
                }
            }
        }

        Ok(())
    }

    /// Flush any remaining content in the buffer
    pub async fn flush(&mut self) -> Result<(), anyhow::Error> {
        if !self.buffer.is_empty() {
            let line = self.buffer.trim().to_string();
            if let Some(content) = parse_sse_line(&line) {
                self.sender.send(content).await?;
            }
            self.buffer.clear();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_line() {
        let line = r#"data: {"id":"1","choices":[{"index":0,"delta":{"content":"Hello"}}]}"#;
        let result = parse_sse_line(line);
        assert_eq!(result, Some("Hello".to_string()));
    }

    #[test]
    fn test_parse_sse_done() {
        let line = "data: [DONE]";
        let result = parse_sse_line(line);
        assert_eq!(result, None);
    }
}
