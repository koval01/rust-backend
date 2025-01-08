use anyhow::{Context, Result};
use google_generative_ai_rs::v1::gemini::request::{
    GenerationConfig, Request, SafetySettings, SystemInstructionContent, SystemInstructionPart
};
use google_generative_ai_rs::v1::gemini::safety::{HarmBlockThreshold, HarmCategory};
use google_generative_ai_rs::v1::{
    api::{Client, PostResult},
    gemini::{Content, Model, Part, Role},
};

use std::env;

async fn create_client() -> Result<Client> {
    let api_key =
        env::var("GEMINI_API_KEY").context("GEMINI_API_KEY not set in environment variables")?;
    Ok(Client::new_from_model(Model::Gemini1_5Pro, api_key))
}

fn build_request(prompt: &str) -> Request {
    let system_instruction = r#"
1. **Objective**: You are required to generate a JSON object containing 10 language learning tasks focused on grammar. These tasks should resemble those found in Duolingo, including:
   - Fill in the blanks
   - Rearrange words to form a correct sentence
   - Translate a sentence
   - Choose the correct translation of a word or phrase

2. **Input**: You will receive a JSON object specifying:
   - `"level"`: the proficiency level of the user (e.g., "A1", "A2", "B1", etc.).
   - `"source_language"`: the ISO code of the source language (e.g., "en" for English).
   - `"target_language"`: the ISO code of the target language the user is learning (e.g., "de" for German).

3. **Output**: Return a JSON object containing an array of 10 tasks. Each task should include:
   - `"type"`: the type of task (`"fill_in_the_blank"`, `"rearrange_sentence"`, `"translate_sentence"`, `"choose_translation"`).
   - `"question"`: the text of the task. This will vary depending on the task type.
   - `"options"` (if applicable): an array of possible answers for tasks that require selection.
   - `"answer"`: the correct answer to the task.
   - `"hint"`: a clue or tip to help the user solve the task.
   - `"error_explanation"`: an explanation of why the incorrect answers are wrong, to aid in learning.

4. **Task Types**:
   - `"fill_in_the_blank"`: Provide a sentence with a blank to be filled in. Example: "Ich ____ ein Auto."
   - `"rearrange_sentence"`: Provide an array of words that need to be rearranged to form a correct sentence. Example: ["ist", "das", "mein", "Haus"].
   - `"translate_sentence"`: Provide a sentence that needs to be translated from the source language to the target language. Example: "I have a book."
   - `"choose_translation"`: Provide a word or phrase with multiple options for translation, where only one option is correct.

5. **Example Input**:
   ```json
   {
     "level": "A1",
     "source_language": "en",
     "target_language": "de"
   }
   ```

6. **Example Output**:
   ```json
   {
     "level": "A1",
     "tasks": [
       {
         "type": "fill_in_the_blank",
         "question": "Ich ____ ein Auto.",
         "options": ["habe", "hat", "habst", "haben"],
         "answer": "habe",
         "hint": "Think about the first-person singular form of 'haben'.",
         "error_explanation": {
           "hat": "This is the third-person singular form.",
           "habst": "This is not a correct conjugation of 'haben'.",
           "haben": "This is the infinitive form, not conjugated."
         }
       },
       {
         "type": "rearrange_sentence",
         "question": ["ist", "das", "mein", "Haus"],
         "answer": "Das ist mein Haus.",
         "hint": "Start with the subject followed by the verb.",
         "error_explanation": {}
       },
       {
         "type": "translate_sentence",
         "question": "I have a book.",
         "answer": "Ich habe ein Buch.",
         "hint": "Remember the basic sentence structure in German.",
         "error_explanation": {}
       },
       {
         "type": "choose_translation",
         "question": "Translate 'dog' into German.",
         "options": ["Hund", "Katze", "Maus", "Pferd"],
         "answer": "Hund",
         "hint": "It's a common household pet.",
         "error_explanation": {
           "Katze": "This means 'cat'.",
           "Maus": "This means 'mouse'.",
           "Pferd": "This means 'horse'."
         }
       }
       // 6 more tasks...
     ]
   }
   ```

7. **Requirements**:
   - Ensure all tasks are grammatically correct and appropriate for the specified level.
   - Use realistic and contextually appropriate examples for the given level.
   - For tasks with multiple options, ensure only one option is correct, and the other options are plausible but incorrect.

8. **Key Points**:
   - Always return exactly 10 tasks in the output.
   - Include a `"hint"` and an `"error_explanation"` for each task to enhance learning.
   - Follow the JSON structure strictly to avoid any formatting errors.

9. **Additional rules**:
   - Take into account the input keys `"source_language"` and `"target_language"`.
   - The `"source_language"` key determines in which language you need to give tasks to the user (condition, error explanation, hint, etc.).
   - The language from the `"target_language"` key can be used solely as the object of study.
   - The language from the `"source_language"` key can only be used to explain errors or hints.

Don't use English without explicitly telling you, prompt and examples English is only used to explain the task to you. 
Also German is only used as an example, work with the languages that the user is asked for in the input json.
"#.trim().to_string();
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
                threshold: HarmBlockThreshold::BlockNone,
            },
            SafetySettings {
                category: HarmCategory::HarmCategoryHarassment,
                threshold: HarmBlockThreshold::BlockNone,
            },
            SafetySettings {
                category: HarmCategory::HarmCategoryHateSpeech,
                threshold: HarmBlockThreshold::BlockNone,
            },
            SafetySettings {
                category: HarmCategory::HarmCategorySexuallyExplicit,
                threshold: HarmBlockThreshold::BlockNone,
            },
        ],
        generation_config: Some(GenerationConfig {
            temperature: Option::from(0.3),
            top_p: Option::from(0.88),
            top_k: Option::from(512),
            candidate_count: Option::from(1),
            max_output_tokens: Option::from(8192),
            stop_sequences: None,
            response_mime_type: Some("application/json".to_string()),
            response_schema: None // using own impl
        }),
        system_instruction: Option::from(SystemInstructionContent {
            parts: vec![SystemInstructionPart {
                text: Some(system_instruction.to_string()),
            }],
        }),
    }
}

pub async fn generate(level: &str, source_language: &str, target_language: &str) -> Result<PostResult> {
    let client = create_client().await?;
    let prompt = serde_json::json!({
        "level": level,
        "source_language": source_language,
        "target_language": target_language
    });
    let prompt = serde_json::to_string(&prompt)?;
    let request = build_request(&*prompt);

    client
        .post(60, &request)
        .await
        .context("Failed to post request")
}
