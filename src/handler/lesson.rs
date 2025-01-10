use std::sync::Arc;

use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    {Extension, Json}
};

use serde_json::json;
use tracing::error;

use prisma_client_rust::{raw, PrismaValue};

use crate::prisma;
use crate::{
    service::llm,
    error::ApiError,
    extractor::InitData,
    prisma::PrismaClient,
    model::{Lesson, User, DataWithUserLessonId},
    response::{ApiResponse, LessonQuery}
};

type Database = Extension<Arc<PrismaClient>>;

pub async fn lesson_handler_get(
    InitData(user): InitData<User>,
    db: Database,
    params: Result<Query<LessonQuery>, axum::extract::rejection::QueryRejection>,
    Extension(gemini_client): Extension<llm::LanguageLearningClient>
) -> Result<impl IntoResponse, ApiError> {
    let params = params.map_err(ApiError::from)?;

    let lesson: Vec<DataWithUserLessonId> = db
        ._query_raw(raw!(r#"
        WITH total_lessons AS (
            SELECT COUNT(*) AS cnt
            FROM "Lesson" l
            LEFT JOIN "UserLesson" ul
            ON l.id = ul."lessonId" AND ul."userId" = {}
            WHERE
                (ul."id" IS NULL OR ul."nextAvailable" < CURRENT_TIMESTAMP)
                AND l."studiedLang" = {}
                AND l."lessonLang" = {}
                AND l."level" = CAST({} AS "Level")
        ),
        selected_lesson AS (
            SELECT l.*
            FROM "Lesson" l
            LEFT JOIN "UserLesson" ul
            ON l.id = ul."lessonId" AND ul."userId" = {}
            WHERE
                (ul."id" IS NULL OR ul."nextAvailable" < CURRENT_TIMESTAMP)
                AND l."studiedLang" = {}
                AND l."lessonLang" = {}
                AND l."level" = CAST({} AS "Level")
            OFFSET FLOOR(RANDOM() * (SELECT cnt FROM total_lessons)) LIMIT 1
        ),
        insert_user_lesson AS (
            INSERT INTO "UserLesson" ("id", "userId", "lessonId", "status", "score")
            SELECT
                gen_random_uuid(),
                {},
                l.id,
                'PENDING',
                0
            FROM selected_lesson l
            RETURNING *
        )
        SELECT
            l.*,
            ul.id as "userLessonId"
        FROM selected_lesson l
        JOIN insert_user_lesson ul ON l.id = ul."lessonId";
        "#,
        PrismaValue::Int(user.id),
        PrismaValue::Enum(format!("{:?}", params.target_language)),
        PrismaValue::Enum(format!("{:?}", params.source_language)),
        PrismaValue::Enum(format!("{:?}", params.level)),
        PrismaValue::Int(user.id),
        PrismaValue::Enum(format!("{:?}", params.target_language)),
        PrismaValue::Enum(format!("{:?}", params.source_language)),
        PrismaValue::Enum(format!("{:?}", params.level)),
        PrismaValue::Int(user.id)
        ))
        .exec()
        .await?;

    match lesson.first() {
        Some(lesson) => {
            let response = ApiResponse::success(json!({
                "lesson": lesson.lesson.lesson_data,
                "lesson_id": lesson.user_lesson_id
            }));
            return Ok((StatusCode::OK, Json(response)));
        },
        None => ()
    }

    // Call the LLM generation service
    let gemini_request = llm::LanguageLearningRequest::new(
        format!("{:?}", params.level),
        format!("{:?}", params.source_language),
        format!("{:?}", params.target_language)
    );

    let gemini_result = gemini_client.generate_tasks(gemini_request)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            ApiError::InternalServerError
        })?;

    let gemini_response = gemini_result.rest().unwrap();
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
