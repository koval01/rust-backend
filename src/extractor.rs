use hmac::{Hmac, Mac};
use jwt::{Error as JwtError, SignWithKey, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;

use axum::{extract::FromRequestParts, http::request::Parts, Extension, RequestPartsExt};
use reqwest::StatusCode;
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

    pub fn sign(&self, claims: &BTreeMap<&str, &str>) -> Result<String, JwtError> {
        claims.sign_with_key(&self.0)
    }

    pub fn verify(&self, token: &str) -> Result<BTreeMap<String, String>, JwtError> {
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

        let claims: BTreeMap<String, String> = jwt_key
            .verify(token)
            .map_err(|e| ApiError::Custom(StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

        let sub = claims
            .get("sub")
            .ok_or(ApiError::Custom(StatusCode::UNAUTHORIZED, "Missing sub claim".into()))?
            .to_owned();

        let email = claims
            .get("email")
            .ok_or(ApiError::Custom(StatusCode::UNAUTHORIZED, "Missing email claim".into()))?
            .to_owned();

        let verified_email = claims
            .get("email_verified")
            .map(|v| v == "true")
            .unwrap_or(false);

        let name = claims
            .get("name")
            .ok_or(ApiError::Custom(StatusCode::UNAUTHORIZED, "Missing name claim".into()))?
            .to_owned();

        let given_name = claims
            .get("given_name")
            .ok_or(ApiError::Custom(StatusCode::UNAUTHORIZED, "Missing given_name claim".into()))?
            .to_owned();

        let family_name = claims
            .get("family_name")
            .map(|s| s.to_owned());

        let picture = claims.get("picture").map(|s| s.to_owned());

        Ok(GoogleUser {
            sub,
            email,
            verified_email,
            name,
            given_name,
            family_name,
            picture,
        })
    }
}
