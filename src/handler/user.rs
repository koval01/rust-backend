use axum::{
    response::IntoResponse,
    http::StatusCode,
    Json,
    Extension,
};
use std::sync::Arc;

use crate::{
    error::ApiError,
    model::User,
    response::{ApiResponse, UserResponseData},
    extractor::InitData,
};

use crate::prisma::*;

type Database = Extension<Arc<PrismaClient>>;

pub async fn user_handler_get(
    InitData(user): InitData<User>,
    db: Database
) -> Result<impl IntoResponse, ApiError> {
    let user_data = UserResponseData { user };
    let user = db
        .user()
        .find_first(vec![user::id::equals(user_data.user.id)])
        .exec()
        .await?;

    let response = ApiResponse::success(user);
    Ok((StatusCode::OK, Json(response)))
}

/* 
The code you can see below is already implemented as middleware, 
so manual work with CRUD methods is not needed
*/

// pub async fn user_handler_post(
//     InitData(user): InitData<User>,
//     db: Database
// ) -> Result<impl IntoResponse, ApiError> {
//     let User {
//         id,
//         first_name,
//         last_name,
//         username,
//         language_code,
//         allows_write_to_pm,
//         photo_url,
//     } = user;
//     let data = db
//         .user()
//         .create(
//             id, 
//             first_name,
//             language_code,
//             allows_write_to_pm,
//             photo_url,
//             vec![
//                 user::last_name::set(last_name),
//                 user::username::set(username),
//             ])
//         .exec()
//         .await;
// 
//     let response = ApiResponse::success(data);
//     Ok((StatusCode::OK, Json(response)))
// }
// 
// pub async fn user_handler_put(
//     InitData(user): InitData<User>,
//     db: Database
// ) -> Result<impl IntoResponse, ApiError> {
//     let User {
//         id,
//         first_name,
//         last_name,
//         username,
//         language_code,
//         allows_write_to_pm,
//         photo_url,
//     } = user;
//     
//     let data = db
//         .user()
//         .update(
//             user::id::equals(id),
//             vec![
//                 user::first_name::set(first_name),
//                 user::last_name::set(last_name),
//                 user::username::set(username),
//                 user::language_code::set(language_code),
//                 user::SetParam::SetAllowsWriteToPm(allows_write_to_pm),
//                 user::photo_url::set(photo_url),
//             ]
//         )
//         .exec()
//         .await;
// 
//     let response = ApiResponse::success(data);
//     Ok((StatusCode::OK, Json(response)))
// }
