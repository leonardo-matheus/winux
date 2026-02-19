// Image Vision - Analyze images using GPT-4o Vision

use crate::ai::{AzureOpenAIClient, SystemPrompts};
use anyhow::{anyhow, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::path::Path;
use std::sync::Arc;

pub struct ImageVision {
    client: Arc<AzureOpenAIClient>,
}

impl ImageVision {
    pub fn new(client: Arc<AzureOpenAIClient>) -> Self {
        Self { client }
    }

    /// Analyze an image from file path
    pub async fn analyze(&self, image_path: &Path, prompt: Option<&str>) -> Result<String> {
        let base64_image = self.encode_image(image_path)?;
        let prompt = prompt.unwrap_or("Describe this image in detail.");

        self.client.analyze_image(&base64_image, prompt).await
    }

    /// Analyze an image from URL
    pub async fn analyze_url(&self, url: &str, prompt: Option<&str>) -> Result<String> {
        let prompt = prompt.unwrap_or("Describe this image in detail.");
        self.client.analyze_image(url, prompt).await
    }

    /// Describe what's in an image
    pub async fn describe(&self, image_path: &Path) -> Result<String> {
        let base64_image = self.encode_image(image_path)?;
        let prompt = SystemPrompts::image_analyzer();

        self.client.analyze_image(&base64_image, prompt).await
    }

    /// Extract text from an image (OCR)
    pub async fn extract_text(&self, image_path: &Path) -> Result<String> {
        let base64_image = self.encode_image(image_path)?;
        let prompt = "Extract all visible text from this image. Maintain the original formatting as much as possible.";

        self.client.analyze_image(&base64_image, prompt).await
    }

    /// Analyze a screenshot and explain the UI
    pub async fn analyze_screenshot(&self, image_path: &Path) -> Result<String> {
        let base64_image = self.encode_image(image_path)?;
        let prompt = r#"This is a screenshot. Please:
1. Identify the application or website shown
2. Describe the current state and what's visible
3. Note any relevant UI elements, buttons, or controls
4. If there's text content, summarize it
5. If there's an error or issue visible, explain it"#;

        self.client.analyze_image(&base64_image, prompt).await
    }

    /// Extract code from a code screenshot
    pub async fn extract_code(&self, image_path: &Path) -> Result<String> {
        let base64_image = self.encode_image(image_path)?;
        let prompt = r#"This image contains code. Please:
1. Transcribe the code exactly as shown
2. Identify the programming language
3. Put the code in a properly formatted code block
4. Note any visible syntax errors or issues"#;

        self.client.analyze_image(&base64_image, prompt).await
    }

    /// Analyze a diagram or flowchart
    pub async fn analyze_diagram(&self, image_path: &Path) -> Result<String> {
        let base64_image = self.encode_image(image_path)?;
        let prompt = r#"This is a diagram or flowchart. Please:
1. Identify the type of diagram (flowchart, UML, architecture, etc.)
2. List all components/nodes
3. Describe the relationships and connections
4. Explain the overall flow or structure
5. Provide a textual representation if possible"#;

        self.client.analyze_image(&base64_image, prompt).await
    }

    /// Compare two images
    pub async fn compare(&self, image1_path: &Path, image2_path: &Path) -> Result<String> {
        let base64_image1 = self.encode_image(image1_path)?;
        let base64_image2 = self.encode_image(image2_path)?;

        // For now, analyze sequentially - could be enhanced with multi-image support
        let desc1 = self.client.analyze_image(&base64_image1, "Describe this image in detail.").await?;
        let desc2 = self.client.analyze_image(&base64_image2, "Describe this image in detail.").await?;

        // Ask to compare the descriptions
        let comparison_messages = vec![
            serde_json::json!({
                "role": "system",
                "content": "You are an image comparison expert."
            }),
            serde_json::json!({
                "role": "user",
                "content": format!(
                    "Compare these two image descriptions and identify differences:\n\nImage 1:\n{}\n\nImage 2:\n{}",
                    desc1, desc2
                )
            }),
        ];

        self.client.chat_completion(comparison_messages).await
    }

    /// Get accessibility description (for screen readers)
    pub async fn accessibility_description(&self, image_path: &Path) -> Result<String> {
        let base64_image = self.encode_image(image_path)?;
        let prompt = "Provide an accessibility-focused description of this image suitable for screen readers. Include all important visual information that a visually impaired person would need to understand the image.";

        self.client.analyze_image(&base64_image, prompt).await
    }

    /// Ask a question about an image
    pub async fn ask(&self, image_path: &Path, question: &str) -> Result<String> {
        let base64_image = self.encode_image(image_path)?;
        self.client.analyze_image(&base64_image, question).await
    }

    /// Encode image to base64 data URL
    fn encode_image(&self, path: &Path) -> Result<String> {
        let bytes = std::fs::read(path)
            .map_err(|e| anyhow!("Failed to read image: {}", e))?;

        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        // Validate it's an image
        if !mime.starts_with("image/") {
            return Err(anyhow!("File is not an image: {}", mime));
        }

        let base64_data = STANDARD.encode(&bytes);
        Ok(format!("data:{};base64,{}", mime, base64_data))
    }

    /// Check if file is a supported image format
    pub fn is_supported_format(path: &Path) -> bool {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        matches!(
            mime.as_ref(),
            "image/png" | "image/jpeg" | "image/gif" | "image/webp"
        )
    }
}
