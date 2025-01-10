use anyhow::{Context, Result};
use google_generative_ai_rs::v1::{
    api::{Client, PostResult},
    gemini::{
        request::{GenerationConfig, Request, SafetySettings, SystemInstructionContent, SystemInstructionPart},
        safety::{HarmBlockThreshold, HarmCategory},
        Content, Model, Part, Role,
    },
};
use serde::{Deserialize, Serialize};

use std::env;
use std::sync::Arc;

const MAX_OUTPUT_TOKENS: i32 = 8192;
const TEMPERATURE: f32 = 0.4;
const TOP_P: f32 = 0.8;
const TOP_K: i32 = 1024;
const CANDIDATE_COUNT: i32 = 1;
const TIMEOUT_SECONDS: u64 = 60;

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageLearningRequest {
    level: String,
    source_language: String,
    target_language: String,
}

impl LanguageLearningRequest {
    pub fn new(level: impl Into<String>, source_language: impl Into<String>, target_language: impl Into<String>) -> Self {
        Self {
            level: level.into(),
            source_language: source_language.into(),
            target_language: target_language.into(),
        }
    }
}

pub struct LanguageLearningClient {
    client: Arc<Client>,
}

impl Clone for LanguageLearningClient {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client)
        }
    }
}

impl LanguageLearningClient {
    pub async fn new() -> Result<Self> {
        let api_key = env::var("GEMINI_API_KEY")
            .context("GEMINI_API_KEY not set in environment variables")?;

        let client = Client::new_from_model(Model::Gemini1_5Pro, api_key);
        Ok(Self {
            client: Arc::new(client)
        })
    }

    pub async fn generate_tasks(&self, request: LanguageLearningRequest) -> Result<PostResult> {
        let prompt = serde_json::to_string(&request)?;
        let request = self.build_request(&prompt);

        self.client
            .post(TIMEOUT_SECONDS, &request)
            .await
            .context("Failed to post request to Gemini API")
    }

    fn build_request(&self, prompt: &str) -> Request {
        Request {
            contents: vec![self.create_content(prompt)],
            tools: vec![],
            safety_settings: self.create_safety_settings(),
            generation_config: Some(self.create_generation_config()),
            system_instruction: Some(self.create_system_instruction()),
        }
    }

    fn create_content(&self, prompt: &str) -> Content {
        Content {
            role: Role::User,
            parts: vec![Part {
                text: Some(prompt.to_string()),
                inline_data: None,
                file_data: None,
                video_metadata: None,
            }],
        }
    }

    fn create_safety_settings(&self) -> Vec<SafetySettings> {
        vec![
            HarmCategory::HarmCategoryDangerousContent,
            HarmCategory::HarmCategoryHarassment,
            HarmCategory::HarmCategoryHateSpeech,
            HarmCategory::HarmCategorySexuallyExplicit,
        ]
            .into_iter()
            .map(|category| SafetySettings {
                category,
                threshold: HarmBlockThreshold::BlockNone,
            })
            .collect()
    }

    fn create_generation_config(&self) -> GenerationConfig {
        GenerationConfig {
            temperature: Some(TEMPERATURE),
            top_p: Some(TOP_P),
            top_k: Some(TOP_K),
            candidate_count: Some(CANDIDATE_COUNT),
            max_output_tokens: Some(MAX_OUTPUT_TOKENS),
            stop_sequences: None,
            response_mime_type: Some("application/json".to_string()),
            response_schema: None,
        }
    }

    fn create_system_instruction(&self) -> SystemInstructionContent {
        SystemInstructionContent {
            parts: vec![SystemInstructionPart {
                text: Some(include_str!("../../prompts/language_learning_system_prompt.txt").to_string()),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        temp_env::with_var("GEMINI_API_KEY", Some("test_key"), || async {
            let client = LanguageLearningClient::new().await;
            assert!(client.is_ok());
        })
            .await;
    }

    #[test]
    fn test_request_creation() {
        let request = LanguageLearningRequest::new("A1", "en", "fr");
        assert_eq!(request.level, "A1");
        assert_eq!(request.source_language, "en");
        assert_eq!(request.target_language, "fr");
    }
}
