use crate::model::User;
use serde::Serialize;

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub status: String,
    pub user: User,
}
