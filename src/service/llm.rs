use std::env;
use google_generative_ai_rs::v1::gemini::request::{GenerationConfig, Request, SafetySettings};
use google_generative_ai_rs::v1::{
    api::{Client, PostResult},
    gemini::{Content, Model, Part, Role},
};
use anyhow::{Result, Context};
use google_generative_ai_rs::v1::gemini::safety::{HarmBlockThreshold, HarmCategory};

async fn create_client() -> Result<Client> {
    let api_key = env::var("GEMINI_API_KEY")
        .context("GEMINI_API_KEY not set in environment variables")?;
    Ok(Client::new_from_model(Model::Gemini1_5Pro, api_key))
}

fn build_request(prompt: &str) -> Request {
    Request {
        contents: vec![Content {
            role: Role::User,
            parts: vec![Part {
                text: Some(prompt.to_string()),
                inline_data: None,
                file_data: None,
                video_metadata: None,
            }],
        }],
        tools: vec![],
        safety_settings: vec![
            SafetySettings {
                category: HarmCategory::HarmCategoryDangerousContent,
                threshold: HarmBlockThreshold::BlockNone
            },
            SafetySettings {
                category: HarmCategory::HarmCategoryHarassment,
                threshold: HarmBlockThreshold::BlockNone
            },
            SafetySettings {
                category: HarmCategory::HarmCategoryHateSpeech,
                threshold: HarmBlockThreshold::BlockNone
            },
            SafetySettings {
                category: HarmCategory::HarmCategorySexuallyExplicit,
                threshold: HarmBlockThreshold::BlockNone
            }
        ],
        generation_config: Some(GenerationConfig {
            temperature: Option::from(0.7),
            top_p: Option::from(0.9),
            top_k: Option::from(128),
            candidate_count: Option::from(1),
            max_output_tokens: Option::from(8192),
            stop_sequences: None,
            response_mime_type: Some("application/json".to_string()),
            response_schema: None,
        }),
        system_instruction: None,
    }
}

pub async fn generate() -> Result<PostResult> {
    let client = create_client().await?;
    let prompt = r#"List 5 popular cookie recipes using this JSON schema: 
                    { "type": "object", "properties": { "recipe_name": { "type": "string" }}}"#;
    let request = build_request(prompt);

    client.post(60, &request).await.context("Failed to post request")
}
