use std::{net::SocketAddr, path::PathBuf};

use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir};

mod error;
mod sync;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct WebServerOptions {
    pub host: String,
    pub port: u16,
    pub frontend_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl WebServerOptions {
    pub fn bind_addr(&self) -> Result<SocketAddr, String> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|err| format!("Invalid bind address {}:{}: {err}", self.host, self.port))
    }
}

#[derive(Clone)]
pub(crate) struct AppState {
    frontend_dir: PathBuf,
    config_dir: PathBuf,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Serialize)]
struct InfoResponse {
    name: &'static str,
    version: &'static str,
    core_message: String,
    frontend_dir: String,
}

pub fn router(frontend_dir: PathBuf) -> Router {
    router_with_config_dir(frontend_dir, default_config_dir())
}

pub fn router_with_config_dir(frontend_dir: PathBuf, config_dir: PathBuf) -> Router {
    let state = AppState {
        frontend_dir: frontend_dir.clone(),
        config_dir,
    };

    Router::new()
        .route("/api/health", get(health))
        .route("/api/info", get(info))
        .merge(sync::router())
        .with_state(state)
        .fallback_service(ServeDir::new(frontend_dir).append_index_html_on_directories(true))
        .layer(CorsLayer::permissive())
}

pub async fn serve(options: WebServerOptions) -> Result<(), String> {
    let bind_addr = options.bind_addr()?;
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|err| format!("Failed to start web server at {bind_addr}: {err}"))?;

    axum::serve(
        listener,
        router_with_config_dir(options.frontend_dir, options.config_dir),
    )
    .await
    .map_err(|err| format!("Web server exited unexpectedly: {err}"))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "byi-server",
    })
}

async fn info(State(state): State<AppState>) -> Json<InfoResponse> {
    Json(InfoResponse {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        core_message: byi_core::hello_message(),
        frontend_dir: state.frontend_dir.display().to_string(),
    })
}

fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("byi")
}
