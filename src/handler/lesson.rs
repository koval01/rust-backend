use std::sync::Arc;

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};

use serde_json::json;
use tracing::error;

use crate::prisma;
use crate::{
    service::llm,
    error::ApiError, 
    extractor::InitData,
    prisma::PrismaClient,
    model::{Lesson, User},
    response::{ApiResponse, LessonQuery}
};

type Database = Extension<Arc<PrismaClient>>;

pub async fn lesson_handler_get(
    InitData(user): InitData<User>,
    db: Database,
    params: Result<Query<LessonQuery>, axum::extract::rejection::QueryRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let params = params.map_err(ApiError::from)?;

    // Call the LLM generation service
    let result = llm::generate(
        &format!("{:?}", params.level),
        &format!("{:?}", params.source_language),
        &format!("{:?}", params.target_language),
    )
    .await
    .map_err(|e| {
        error!("{:?}", e);
        ApiError::InternalServerError
    })?;

    let gemini_response = result.rest().unwrap();
    let candidate = gemini_response
        .candidates
        .first()
        .ok_or(ApiError::InternalServerError)?;

    let part = candidate
        .content
        .parts
        .first()
        .ok_or(ApiError::InternalServerError)?;

    let response_text = part.text.as_ref().ok_or(ApiError::InternalServerError)?;

    let lesson_response: Lesson = serde_json::from_str(response_text).map_err(|e| {
        error!(
            "JSON Gemini response parsing error: {:?}. Input data: {:?}",
            e, response_text
        );
        ApiError::InternalServerError
    })?;

    let (_, user_lesson) = db
        ._transaction()
        .run(|client| async move {
            let new_lesson = client
                .lesson()
                .create(
                    response_text.parse().unwrap(),
                    format!("{:?}", params.target_language),
                    format!("{:?}", params.source_language),
                    (&params.level).into(),
                    vec![],
                )
                .exec()
                .await?;

            client
                .user_lesson()
                .create(
                    prisma::user::id::equals(user.id),
                    prisma::lesson::id::equals(String::from(&new_lesson.id)),
                    vec![]
                )
                .exec()
                .await
                .map(|user_lesson| (new_lesson, user_lesson))
        })
        .await?;

    let response = ApiResponse::success(json!({
        "lesson": lesson_response,
        "lesson_id": user_lesson.id
    }));
    Ok((StatusCode::OK, Json(response)))
}
