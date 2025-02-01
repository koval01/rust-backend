use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    User,
    Admin
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub google_id: String,
    pub display_name: String,
    pub role: Role,
    pub photo_url: Option<String>,
    pub visible: bool
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GoogleUser {
    #[serde(rename = "id")]
    pub sub: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: String,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub expiry: Option<i64>
}

impl GoogleUser {
    pub fn to_btree_map(&self) -> BTreeMap<&str, Value> {
        let mut map = BTreeMap::new();

        map.insert("sub", Value::String(self.sub.clone()));
        map.insert("email", Value::String(self.email.clone()));
        map.insert("name", Value::String(self.name.clone()));
        map.insert("given_name", Value::String(self.given_name.clone()));

        if let Some(ref family_name) = self.family_name {
            map.insert("family_name", Value::String(family_name.clone()));
        }
        if let Some(ref picture) = self.picture {
            map.insert("picture", Value::String(picture.clone()));
        }
        if let Some(expiry) = self.expiry {
            map.insert("expiry", Value::Number(serde_json::Number::from(expiry)));
        }

        map
    }
}
