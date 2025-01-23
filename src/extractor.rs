use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use url::form_urlencoded;

use crate::error::ApiError;

pub struct InitData<T>(pub T);

impl<T, S> FromRequestParts<S> for InitData<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let decoded_init_data = parts
            .extensions
            .get::<String>()
            .ok_or(ApiError::BadRequest)?;

        let mut query_pairs = form_urlencoded::parse(decoded_init_data.as_bytes());
        let user_query = query_pairs
            .find(|(key, _)| key == "user")
            .ok_or(ApiError::BadRequest)?
            .1
            .to_string();

        let data: T = serde_json::from_str(&user_query).map_err(|_| ApiError::BadRequest)?;
        Ok(InitData(data))
    }
}
