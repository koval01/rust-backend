use serde::{Deserialize, Serialize};
use crate::{model::User, prisma};

use std::str::FromStr;

#[derive(Serialize)]
pub struct UserResponseData {
    pub user: User,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum Level {
    A1,
    A2,
    B1,
    B2,
    C1,
    C2
}

impl FromStr for Level {
    type Err = ();

    fn from_str(input: &str) -> Result<Level, Self::Err> {
        match input {
            "A1" => Ok(Level::A1),
            "A2" => Ok(Level::A2),
            "B1" => Ok(Level::B1),
            "B2" => Ok(Level::B2),
            "C1" => Ok(Level::C1),
            "C2" => Ok(Level::C2),
            _ => Err(()),
        }
    }
}

impl From<&Level> for prisma::Level {
    fn from(level: &Level) -> Self {
        match level {
            Level::A1 => prisma::Level::A1,
            Level::A2 => prisma::Level::A2,
            Level::B1 => prisma::Level::B1,
            Level::B2 => prisma::Level::B2,
            Level::C1 => prisma::Level::C1,
            Level::C2 => prisma::Level::C2,
        }
    }
}

impl From<prisma::Level> for Level {
    fn from(level: prisma::Level) -> Self {
        match level {
            prisma::Level::A1 => Level::A1,
            prisma::Level::A2 => Level::A2,
            prisma::Level::B1 => Level::B1,
            prisma::Level::B2 => Level::B2,
            prisma::Level::C1 => Level::C1,
            prisma::Level::C2 => Level::C2,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[serde(alias = "en")]
    English,
    #[serde(alias = "es")]
    Spanish,
    #[serde(alias = "zh")]
    Chinese,
    #[serde(alias = "ar")]
    Arabic,
    #[serde(alias = "pt")]
    Portuguese,
    #[serde(alias = "ru")]
    Russian,
    #[serde(alias = "jp")]
    Japanese,
    #[serde(alias = "de")]
    German,
    #[serde(alias = "ko")]
    Korean,
    #[serde(alias = "fr")]
    French,
    #[serde(alias = "tr")]
    Turkish,
    #[serde(alias = "it")]
    Italian,
    #[serde(alias = "uk")]
    Ukrainian,
    #[serde(alias = "pl")]
    Polish,
    #[serde(alias = "cz")]
    Czech,
}

impl FromStr for Language {
    type Err = ();

    fn from_str(input: &str) -> Result<Language, Self::Err> {
        match input.to_lowercase().as_str() {
            "en" => Ok(Language::English),
            "es" => Ok(Language::Spanish),
            "zh" => Ok(Language::Chinese),
            "ar" => Ok(Language::Arabic),
            "pt" => Ok(Language::Portuguese),
            "ru" => Ok(Language::Russian),
            "jp" => Ok(Language::Japanese),
            "de" => Ok(Language::German),
            "ko" => Ok(Language::Korean),
            "fr" => Ok(Language::French),
            "tr" => Ok(Language::Turkish),
            "it" => Ok(Language::Italian),
            "uk" => Ok(Language::Ukrainian),
            "pl" => Ok(Language::Polish),
            "cz" => Ok(Language::Czech),
            _ => Err(()),
        }
    }
}

#[derive(Deserialize)]
pub struct LessonQuery {
    pub level: Level,
    pub source_language: Language,
    pub target_language: Language,
}
