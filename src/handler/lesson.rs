use std::sync::Arc;

use axum::{Extension, Json};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::{
    error::ApiError,
    extractor::InitData,
    model::User,
    model::Lesson,
    prisma::PrismaClient,
    service::llm,
    response::{ApiResponse, LessonQuery}
};

use tracing::error;

type Database = Extension<Arc<PrismaClient>>;

pub async fn lesson_handler_get(
    InitData(user): InitData<User>,
    params: Result<Query<LessonQuery>, axum::extract::rejection::QueryRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let params = params.map_err(ApiError::from)?;
    
    // Call the LLM generation service
    let result = llm::generate(
        &format!("{:?}", params.level), 
        &format!("{:?}", params.source_language), 
        &format!("{:?}", params.target_language))
        .await
        .map_err(|e| {
            error!("{:?}", e);
            ApiError::InternalServerError
        })?;

    let gemini_response = result.rest().unwrap();
    let candidate = gemini_response.candidates
        .first()
        .ok_or(ApiError::InternalServerError)?;

    let part = candidate
        .content
        .parts
        .first()
        .ok_or(ApiError::InternalServerError)?;

    let response_text = part
        .text
        .as_ref()
        .ok_or(ApiError::InternalServerError)?;
    
    let lesson_response: Lesson = serde_json::from_str(response_text)
        .map_err(|e| {
            error!("JSON Gemini response parsing error: {:?}. Input data: {:?}", e, response_text);
            ApiError::InternalServerError
        })?;

    let response = ApiResponse::success(lesson_response);
    Ok((StatusCode::OK, Json(response)))
}