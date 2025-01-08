use ahash::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Task {
    #[serde(rename = "fill_in_the_blank")]
    FillInTheBlank {
        question: String,
        options: Vec<String>,
        answer: String,
        hint: String,
        error_explanation: HashMap<String, String>,
    },
    #[serde(rename = "rearrange_sentence")]
    RearrangeSentence {
        question: Vec<String>,
        answer: String,
        hint: String,
        error_explanation: HashMap<String, String>,
    },
    #[serde(rename = "translate_sentence")]
    TranslateSentence {
        question: String,
        answer: String,
        hint: String,
        error_explanation: HashMap<String, String>,
    },
    #[serde(rename = "choose_translation")]
    ChooseTranslation {
        question: String,
        options: Vec<String>,
        answer: String,
        hint: String,
        error_explanation: HashMap<String, String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorExplanation {
    pub option: String,  // The option that is incorrect
    pub explanation: String, // The explanation of why the option is incorrect
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Lesson {
    pub level: String,
    pub tasks: Vec<Task>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub level: String, // user level (e.g., A1, A2, etc.)
    pub source_language: String, // source language (e.g., "en")
    pub target_language: String, // target language (e.g., "de")
}
