use std::sync::Arc;

use anyhow::Result;
use axum::{
    Router,
    body::Bytes,
    extract::{Query, State},
    routing::{get, put},
};
use reqwest::StatusCode;
use serde::Deserialize;
use tower_http::trace::TraceLayer;

use crate::{AppError, config::Config};

#[derive(Deserialize, Debug)]
struct FileParamas {
    destination: String,
}

async fn handle_download(
    State(config): State<Arc<Config>>,
    Query(params): Query<FileParamas>,
) -> anyhow::Result<Vec<u8>, AppError> {
    let path = config.location.join(params.destination);

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    Ok(tokio::fs::read(path).await?)
}

async fn handle_upload(
    State(config): State<Arc<Config>>,
    Query(params): Query<FileParamas>,
    body: Bytes,
) -> anyhow::Result<StatusCode, AppError> {
    let path = config.location.join(params.destination);

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(path, body).await?;

    Ok(StatusCode::CREATED)
}

pub(crate) fn app(config: Config) -> Router {
    Router::new()
        .route("/file", put(handle_upload))
        .route("/file", get(handle_download))
        .with_state(Arc::new(config))
        .layer(TraceLayer::new_for_http())
}

pub async fn serve(config: Config) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(config.address()).await?;
    axum::serve(listener, app(config)).await?;

    Ok(())
}
