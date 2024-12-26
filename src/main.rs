mod route;
mod middleware;
mod error;
mod extractor;
mod handler;
mod model;
mod response;
mod util;

use std::env;
use bb8_redis::RedisConnectionManager;
use bb8_redis::bb8;

#[allow(warnings, unused)]
mod prisma;

use prisma::PrismaClient;

use std::sync::Arc;
use std::time::Duration;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use axum::extract::{ Extension };
use redis::AsyncCommands;
use tower::ServiceBuilder;
use route::create_router;
use tower_http::cors::CorsLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cors_host = env::var("CORS_HOST").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let cors = CorsLayer::new()
        .allow_origin(cors_host.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost".to_string());
    let redis_manager = RedisConnectionManager::new(redis_url).unwrap();
    let redis_pool = bb8::Pool::builder()
        .max_size((num_cpus::get() * 10) as u32) // Увеличиваем размер пула
        .min_idle((num_cpus::get() * 2) as u32)
        .max_lifetime(None)
        .connection_timeout(Duration::from_millis(100))
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
        .layer(cors)
        .layer(tower::limit::ConcurrencyLimitLayer::new(1000))
        .into_inner();

    let app = create_router()
        .layer(middleware_stack)
        .layer(Extension(redis_pool))
        .layer(Extension(prisma_client));

    info!("🚀 Server started successfully");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
        .tcp_nodelay(true)
        .await
        .unwrap();
}
