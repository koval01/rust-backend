use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    id: i64,
    first_name: String,
    last_name: String,
    language_code: String,
    allows_write_to_pm: bool,
    photo_url: String,
}
