use serde::Serialize;
use crate::model::User;

#[derive(Serialize)]
pub struct UserResponseData {
    pub user: User,
}
