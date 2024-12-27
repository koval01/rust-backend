mod route;
mod middleware;
mod error;
mod extractor;
mod handler;
mod model;
mod response;
mod util;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use bb8_redis::RedisConnectionManager;
use bb8_redis::bb8;

use redis::AsyncCommands;

#[allow(warnings, unused)]
mod prisma;
use prisma::PrismaClient;

use axum::http::{header::{ACCEPT, CONTENT_TYPE}, HeaderName, HeaderValue, Method};
use axum::extract::{ Extension };
use route::create_router;

use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer
};

use sentry::{ClientOptions, IntoDsn};
use sentry_tower::NewSentryLayer;
use tracing::info;

use crate::{
    middleware::{request_id_middleware, timestamp_guard_middleware}
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let _dsn = env::var("SENTRY_DSN").unwrap_or_else(|_| "".to_string());
    let _guard = sentry::init((
        _dsn.into_dsn().unwrap(),
        ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 0.2,
            ..Default::default()
        },
    ));

    let cors_host = env::var("CORS_HOST").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let cors = CorsLayer::new()
        .allow_origin(cors_host.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([
            ACCEPT, 
            CONTENT_TYPE,
            HeaderName::from_static("x-initdata"),
            HeaderName::from_static("x-timestamp"),
        ]);

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost".to_string());
    let redis_manager = RedisConnectionManager::new(redis_url).unwrap();
    let redis_pool = bb8::Pool::builder()
        .max_size((num_cpus::get() * 10) as u32)
        .min_idle((num_cpus::get() * 2 + 1) as u32)
        .max_lifetime(None)
        .connection_timeout(Duration::from_millis(1000))
        .idle_timeout(Some(Duration::from_secs(60)))
        .build(redis_manager)
        .await
        .unwrap();

    {
        let mut conn = redis_pool.get().await.unwrap();
        let _: () = conn.set("health_check", "ok").await.unwrap();
    }

    let prisma_client = Arc::new(PrismaClient::_builder().build().await.unwrap());

    let middleware_stack = ServiceBuilder::new()
        .layer(NewSentryLayer::new_from_top())
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(request_id_middleware))
        .layer(axum::middleware::from_fn(timestamp_guard_middleware))
        .layer(cors)
        .layer(tower::limit::ConcurrencyLimitLayer::new(1000))
        .into_inner();

    let app = create_router()
        .layer(middleware_stack)
        .layer(Extension(redis_pool))
        .layer(Extension(prisma_client));

    let _bind = env::var("SERVER_BIND").unwrap_or_else(|_| "0.0.0.0:8000".to_string());
    let listener = tokio::net::TcpListener::bind(&_bind)
        .await
        .unwrap();

    info!("🚀 Server started successfully on {}", _bind);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
        .tcp_nodelay(true)
        .await
        .unwrap();
}
