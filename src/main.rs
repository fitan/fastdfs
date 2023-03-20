extern crate core;

mod storage;
mod http;
mod wrr;
mod next_file;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use http::file_upload;
use storage::Storage;
use crate::http::file_get;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/file",get(file_get).post(file_upload)).with_state(Arc::new(Storage::new()));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}