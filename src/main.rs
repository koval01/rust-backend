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

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use axum::extract::{ Extension };
use redis::AsyncCommands;
use route::create_router;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let cors_host = env::var("CORS_HOST").unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    let cors = CorsLayer::new()
        .allow_origin(cors_host.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost".to_string());
    let redis_manager = RedisConnectionManager::new(redis_url).unwrap();
    let redis_pool = bb8::Pool::builder().build(redis_manager).await.unwrap();

    {
        // ping the database before starting
        let mut conn = redis_pool.get().await.unwrap();
        conn.set::<&str, &str, ()>("foo", "bar").await.unwrap();
        let result: String = conn.get("foo").await.unwrap();
        assert_eq!(result, "bar");
    }
    
    let prisma_client = Arc::new(PrismaClient::_builder().build().await.unwrap());

    let app = create_router().layer(cors).layer(Extension(prisma_client)).with_state(redis_pool);

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}