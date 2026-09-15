mod cli;
mod config;
mod server;

use anyhow::Result;
use axum::response::IntoResponse;
use reqwest::StatusCode;
use tracing_subscriber::EnvFilter;

// enum Backend {}

pub(crate) struct AppError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(error = ?self.0, "Request failed");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tower_http=debug,dropfish=debug".into()),
        )
        .init();

    cli::run().await
}
