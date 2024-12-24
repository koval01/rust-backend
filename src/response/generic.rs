use axum::http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
#[serde(untagged)]
pub enum ApiData<T> {
    Data(T),
    Empty,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    pub data: ApiData<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            message: String::new(),
            data: ApiData::Data(data),
            code: None,
        }
    }
}

impl<T> ApiResponse<T> {
    pub fn message_only(message: &str) -> Self {
        Self {
            status: "success".to_string(),
            message: message.to_string(),
            data: ApiData::Empty,
            code: None,
        }
    }

    pub fn error(message: &str, code: StatusCode) -> Self {
        Self {
            status: "error".to_string(),
            message: message.to_string(),
            data: ApiData::Empty,
            code: Some(code.as_u16()),
        }
    }
}
