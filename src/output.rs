use byi_storage::RemoteConfig;
use byi_webdav::WebDavPreset;

pub(crate) fn format_remote_status(remote: &RemoteConfig) -> String {
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
