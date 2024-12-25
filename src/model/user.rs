use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub language_code: String,
    pub allows_write_to_pm: bool,
    pub photo_url: Option<String>,
}
