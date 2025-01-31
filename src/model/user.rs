use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

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
}

impl GoogleUser {
    pub fn to_btree_map(&self) -> BTreeMap<&str, &str> {
        let mut map = BTreeMap::new();

        map.insert("sub", self.sub.as_str());
        map.insert("email", self.email.as_str());
        map.insert("name", self.name.as_str());
        map.insert("given_name", self.given_name.as_str());

        if let Some(ref family_name) = self.family_name {
            map.insert("family_name", family_name.as_str());
        }
        if let Some(ref picture) = self.picture {
            map.insert("picture", picture.as_str());
        }

        map
    }
}
