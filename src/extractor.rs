use hmac::{Hmac, Mac};
use jwt::{Error as JwtError, SignWithKey, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;

use axum::{
    extract::FromRequestParts, 
    http::request::Parts, 
    Extension, RequestPartsExt
};

use chrono::Utc;
use reqwest::StatusCode;
use serde_json::Value;

use crate::{
    error::ApiError,
    model::GoogleUser
};

#[derive(Clone)]
pub struct JwtKey(Hmac<Sha256>);

impl JwtKey {
    pub fn new(secret: &[u8]) -> Result<Self, JwtError> {
        Hmac::new_from_slice(secret)
            .map(Self)
            .map_err(|_| JwtError::InvalidSignature)
    }

    pub fn sign(&self, claims: &BTreeMap<&str, Value>) -> Result<String, JwtError> {
        claims.sign_with_key(&self.0)
    }

    pub fn verify(&self, token: &str) -> Result<BTreeMap<String, Value>, JwtError> {
        token.verify_with_key(&self.0)
    }
}

impl<S> FromRequestParts<S> for GoogleUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let Extension(jwt_key) = parts.extract::<Extension<JwtKey>>()
            .await
            .map_err(|_| ApiError::Custom(StatusCode::UNAUTHORIZED, "Missing JWT key".into()))?;

        let headers = parts.headers.clone();
        let auth_header = headers
            .get("Authorization")
            .ok_or(ApiError::Custom(StatusCode::UNAUTHORIZED, "Missing Authorization header".into()))?;

        let token = auth_header
            .to_str()
            .map_err(|_| ApiError::Custom(StatusCode::UNAUTHORIZED, "Invalid Authorization header".into()))?
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Custom(StatusCode::UNAUTHORIZED, "Invalid token format".into()))?;

        let claims: BTreeMap<String, Value> = jwt_key
            .verify(token)
            .map_err(|e| ApiError::Custom(StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

        let sub = claims
            .get("sub")
            .and_then(Value::as_str)
            .ok_or(ApiError::Custom(StatusCode::UNAUTHORIZED, "Missing sub claim".into()))?
            .to_owned();

        let expiry = claims
            .get("expiry")
            .and_then(Value::as_i64)
            .ok_or(ApiError::Custom(StatusCode::UNAUTHORIZED, "Missing or invalid expiry claim".into()))?;

        if Utc::now().timestamp() > expiry {
            return Err(ApiError::Custom(StatusCode::UNAUTHORIZED, "Token expired".into()));
        }

        let expiry = Some(expiry);

        let email = claims
            .get("email")
            .and_then(Value::as_str)
            .ok_or(ApiError::Custom(StatusCode::UNAUTHORIZED, "Missing email claim".into()))?
            .to_owned();

        let verified_email = claims
            .get("email_verified")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let name = claims
            .get("name")
            .and_then(Value::as_str)
            .ok_or(ApiError::Custom(StatusCode::UNAUTHORIZED, "Missing name claim".into()))?
            .to_owned();

        let given_name = claims
            .get("given_name")
            .and_then(Value::as_str)
            .ok_or(ApiError::Custom(StatusCode::UNAUTHORIZED, "Missing given_name claim".into()))?
            .to_owned();

        let family_name = claims
            .get("family_name")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);

        let picture = claims
            .get("picture")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);

        Ok(GoogleUser {
            sub,
            email,
            verified_email,
            name,
            given_name,
            family_name,
            picture,
            expiry
        })
    }
}
