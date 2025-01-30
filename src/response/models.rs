use serde::{Deserialize, Serialize};
use crate::{model::GoogleUser, prisma};

use std::str::FromStr;

#[derive(Serialize)]
pub struct UserResponseData {
    pub user: GoogleUser,
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

#[derive(Deserialize, Debug, Serialize)]
pub enum Language {
    EN,
    ES,
    ZH,
    AR,
    PT,
    RU,
    JP,
    DE,
    KO,
    FR,
    TR,
    IT,
    UK,
    PL,
    CZ,
}

impl Language {
    #[allow(dead_code)]
    pub fn as_name(&self) -> &'static str {
        match self {
            Language::EN => "English",
            Language::ES => "Spanish",
            Language::ZH => "Chinese",
            Language::AR => "Arabic",
            Language::PT => "Portuguese",
            Language::RU => "Russian",
            Language::JP => "Japanese",
            Language::DE => "German",
            Language::KO => "Korean",
            Language::FR => "French",
            Language::TR => "Turkish",
            Language::IT => "Italian",
            Language::UK => "Ukrainian",
            Language::PL => "Polish",
            Language::CZ => "Czech",
        }
    }
}

impl FromStr for Language {
    type Err = ();

    fn from_str(input: &str) -> Result<Language, Self::Err> {
        match input.to_lowercase().as_str() {
            "en" => Ok(Language::EN),
            "es" => Ok(Language::ES),
            "zh" => Ok(Language::ZH),
            "ar" => Ok(Language::AR),
            "pt" => Ok(Language::PT),
            "ru" => Ok(Language::RU),
            "jp" => Ok(Language::JP),
            "de" => Ok(Language::DE),
            "ko" => Ok(Language::KO),
            "fr" => Ok(Language::FR),
            "tr" => Ok(Language::TR),
            "it" => Ok(Language::IT),
            "uk" => Ok(Language::UK),
            "pl" => Ok(Language::PL),
            "cz" => Ok(Language::CZ),
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
