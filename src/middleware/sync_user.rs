use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
    Extension,
};
use std::sync::Arc;

use crate::{
    prisma::*,
    model::User,
    error::ApiError,
    extractor::InitData,
};

pub async fn sync_user_middleware(
    InitData(init_user): InitData<User>,
    Extension(db): Extension<Arc<PrismaClient>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let db_user = db
        .user()
        .find_unique(user::id::equals(init_user.id))
        .exec()
        .await?;

    match db_user {
        None => {
            db.user()
                .create(
                    init_user.id,
                    init_user.first_name,
                    init_user.language_code,
                    init_user.allows_write_to_pm,
                    init_user.photo_url,
                    vec![
                        user::last_name::set(init_user.last_name),
                        user::username::set(init_user.username),
                    ],
                )
                .exec()
                .await?;
        }
        Some(db_user) => {
            if needs_update(&init_user, &db_user) {
                db.user()
                    .update(
                        user::id::equals(init_user.id),
                        vec![
                            user::first_name::set(init_user.first_name),
                            user::last_name::set(init_user.last_name),
                            user::username::set(init_user.username),
                            user::language_code::set(init_user.language_code),
                            user::SetParam::SetAllowsWriteToPm(init_user.allows_write_to_pm),
                            user::photo_url::set(init_user.photo_url),
                        ],
                    )
                    .exec()
                    .await?;
            }
        }
    }

    Ok(next.run(request).await)
}

fn needs_update(init_user: &User, db_user: &user::Data) -> bool {
    init_user.first_name != db_user.first_name
        || init_user.last_name != db_user.last_name
        || init_user.username != db_user.username
        || init_user.language_code != db_user.language_code
        || init_user.allows_write_to_pm != db_user.allows_write_to_pm
        || init_user.photo_url != db_user.photo_url
}
