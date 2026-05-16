use std::{fs, path::Path};

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use byi_storage::{GitHubRemoteConfig, RemoteConfig};
use byi_webdav::{WebDavPreset, WebDavRemoteConfig};
use serde::{Deserialize, Serialize};

use crate::{AppState, error::ApiResult};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/sync/status", get(status))
        .route("/api/sync/config/github", post(configure_github))
        .route("/api/sync/config/webdav", post(configure_webdav))
        .route("/api/sync/test", post(test_remote))
        .route("/api/sync/pull", post(pull))
        .route("/api/sync/push", post(push))
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct ByiConfig {
    remote: Option<RemoteConfig>,
}

#[derive(Serialize)]
pub(crate) struct SyncStatusResponse {
    configured: bool,
    message: String,
    remote: Option<RemoteSummary>,
}

#[derive(Serialize)]
#[serde(tag = "provider", rename_all = "kebab-case")]
enum RemoteSummary {
    #[serde(rename = "github")]
    GitHub {
        repo: String,
        branch: String,
        base_path: String,
        auth: String,
    },
    #[serde(rename = "webdav")]
    WebDav {
        preset: String,
        endpoint_url: String,
        username: Option<String>,
        base_path: String,
    },
}

#[derive(Deserialize)]
struct GitHubConfigRequest {
    repo: String,
    branch: Option<String>,
    base_path: Option<String>,
}

#[derive(Deserialize)]
struct WebDavConfigRequest {
    preset: Option<String>,
    url: Option<String>,
    username: Option<String>,
    base_path: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct OperationResponse {
    ok: bool,
    message: String,
    status: SyncStatusResponse,
}

async fn status(State(state): State<AppState>) -> ApiResult<SyncStatusResponse> {
    Ok(Json(status_response(&state.config_dir)?))
}

async fn configure_github(
    State(state): State<AppState>,
    Json(request): Json<GitHubConfigRequest>,
) -> ApiResult<SyncStatusResponse> {
    validate_repo(&request.repo)?;
    let branch = request.branch.unwrap_or_else(|| "main".to_string());
    let base_path = request.base_path.unwrap_or_else(|| ".byi".to_string());
    validate_relative_path(&base_path)?;

    save_config(
        &state.config_dir,
        &ByiConfig {
            remote: Some(RemoteConfig::GitHub(GitHubRemoteConfig {
                repo: request.repo,
                branch,
                base_path,
                auth: "github-cli".to_string(),
            })),
        },
    )?;

    Ok(Json(status_response(&state.config_dir)?))
}

async fn configure_webdav(
    State(state): State<AppState>,
    Json(request): Json<WebDavConfigRequest>,
) -> ApiResult<SyncStatusResponse> {
    let preset = request
        .preset
        .as_deref()
        .map(byi_webdav::parse_preset)
        .transpose()?
        .unwrap_or(WebDavPreset::Jianguoyun);
    let base_path = request.base_path.unwrap_or_else(|| ".byi".to_string());
    validate_relative_path(&base_path)?;
    let username = request.username.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    });

    let remote = match preset {
        WebDavPreset::Jianguoyun => WebDavRemoteConfig::jianguoyun(username, base_path),
        WebDavPreset::Custom => {
            let url = request
                .url
                .ok_or_else(|| "Custom WebDAV requires a url.".to_string())?;
            byi_webdav::validate_endpoint_url(&url)?;
            WebDavRemoteConfig::custom(url, username, base_path)
        }
    };

    save_config(
        &state.config_dir,
        &ByiConfig {
            remote: Some(RemoteConfig::WebDav(remote)),
        },
    )?;

    Ok(Json(status_response(&state.config_dir)?))
}

async fn test_remote(State(state): State<AppState>) -> ApiResult<OperationResponse> {
    run_operation(&state.config_dir, |remote| {
        byi_storage::storage_for(&remote).test()?;
        Ok("Sync remote test passed.".to_string())
    })
}

async fn pull(State(state): State<AppState>) -> ApiResult<OperationResponse> {
    run_operation(&state.config_dir, |remote| {
        let contents = byi_storage::storage_for(&remote).read_text("metadata.toml")?;
        write_store_file(&state.config_dir, "metadata.toml", &contents)?;
        Ok("Pulled remote data.".to_string())
    })
}

async fn push(State(state): State<AppState>) -> ApiResult<OperationResponse> {
    run_operation(&state.config_dir, |remote| {
        let metadata = ensure_metadata(&state.config_dir)?;
        byi_storage::storage_for(&remote).write_text("metadata.toml", &metadata)?;
        Ok("Pushed local data.".to_string())
    })
}

fn run_operation(
    config_dir: &Path,
    operation: impl FnOnce(RemoteConfig) -> Result<String, String>,
) -> ApiResult<OperationResponse> {
    let remote = require_remote(config_dir)?;
    let message = operation(remote)?;
    Ok(Json(OperationResponse {
        ok: true,
        message,
        status: status_response(config_dir)?,
    }))
}

fn status_response(config_dir: &Path) -> Result<SyncStatusResponse, String> {
    let config = load_config(config_dir)?;
    let Some(remote) = config.remote else {
        return Ok(SyncStatusResponse {
            configured: false,
            message: "Sync remote is not initialized.".to_string(),
            remote: None,
        });
    };

    Ok(SyncStatusResponse {
        configured: true,
        message: format_remote_status(&remote),
        remote: Some(remote_summary(remote)),
    })
}

fn load_config(config_dir: &Path) -> Result<ByiConfig, String> {
    let path = config_dir.join("config.toml");

    if !path.exists() {
        return Ok(ByiConfig::default());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read config {}: {err}", path.display()))?;
    toml::from_str(&contents)
        .map_err(|err| format!("Failed to parse config {}: {err}", path.display()))
}

fn save_config(config_dir: &Path, config: &ByiConfig) -> Result<(), String> {
    fs::create_dir_all(config_dir).map_err(|err| {
        format!(
            "Failed to create config directory {}: {err}",
            config_dir.display()
        )
    })?;
    let path = config_dir.join("config.toml");
    let contents = toml::to_string_pretty(config)
        .map_err(|err| format!("Failed to serialize config {}: {err}", path.display()))?;
    fs::write(&path, contents)
        .map_err(|err| format!("Failed to write config {}: {err}", path.display()))
}

fn require_remote(config_dir: &Path) -> Result<RemoteConfig, String> {
    load_config(config_dir)?
        .remote
        .ok_or_else(|| "Sync remote is not initialized. Run `byi sync config` first.".to_string())
}

fn write_store_file(config_dir: &Path, name: &str, contents: &str) -> Result<(), String> {
    let dir = config_dir.join("store");
    fs::create_dir_all(&dir).map_err(|err| {
        format!(
            "Failed to create local data directory {}: {err}",
            dir.display()
        )
    })?;
    let path = dir.join(name);
    fs::write(&path, contents)
        .map_err(|err| format!("Failed to write local data {}: {err}", path.display()))
}

fn ensure_metadata(config_dir: &Path) -> Result<String, String> {
    let dir = config_dir.join("store");
    fs::create_dir_all(&dir).map_err(|err| {
        format!(
            "Failed to create local data directory {}: {err}",
            dir.display()
        )
    })?;
    let path = dir.join("metadata.toml");

    if path.exists() {
        return fs::read_to_string(&path)
            .map_err(|err| format!("Failed to read local data {}: {err}", path.display()));
    }

    let contents = "schema_version = 1\ntool = \"byi\"\n";
    fs::write(&path, contents)
        .map_err(|err| format!("Failed to write local data {}: {err}", path.display()))?;
    Ok(contents.to_string())
}

fn remote_summary(remote: RemoteConfig) -> RemoteSummary {
    match remote {
        RemoteConfig::GitHub(config) => RemoteSummary::GitHub {
            repo: config.repo,
            branch: config.branch,
            base_path: config.base_path,
            auth: config.auth,
        },
        RemoteConfig::WebDav(config) => RemoteSummary::WebDav {
            preset: format_webdav_preset(&config.preset).to_string(),
            endpoint_url: config.endpoint_url,
            username: config.username,
            base_path: config.base_path,
        },
    }
}

fn format_remote_status(remote: &RemoteConfig) -> String {
    match remote {
        RemoteConfig::GitHub(config) => format!(
            "GitHub remote: {}\nBranch: {}\nBase path: {}\nAuth: GitHub CLI",
            config.repo, config.branch, config.base_path
        ),
        RemoteConfig::WebDav(config) => format!(
            "WebDAV remote: {}\nPreset: {}\nBase path: {}\nUsername: {}",
            config.endpoint_url,
            format_webdav_preset(&config.preset),
            config.base_path,
            config.username.as_deref().unwrap_or("(not set)")
        ),
    }
}

fn format_webdav_preset(preset: &WebDavPreset) -> &'static str {
    match preset {
        WebDavPreset::Jianguoyun => "jianguoyun",
        WebDavPreset::Custom => "custom",
    }
}

fn validate_repo(repo: &str) -> Result<(), String> {
    let parts: Vec<_> = repo.split('/').collect();

    if parts.len() == 2 && parts.iter().all(|part| !part.trim().is_empty()) {
        Ok(())
    } else {
        Err("GitHub repo must use owner/repo format.".to_string())
    }
}

fn validate_relative_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
    {
        Err("Base path must be relative and cannot contain `..`.".to_string())
    } else {
        Ok(())
    }
}
