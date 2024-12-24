mod route;
mod middleware;
mod error;
mod extractor;
mod handler;
mod model;
mod response;
mod util;

#[allow(warnings, unused)]
mod prisma;

use prisma::PrismaClient;

use std::sync::Arc;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use axum::extract::{ Extension };
use route::create_router;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let prisma_client = Arc::new(PrismaClient::_builder().build().await.unwrap());

    let app = create_router().layer(cors).layer(Extension(prisma_client));

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}