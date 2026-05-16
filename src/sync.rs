use byi_storage::{GitHubRemoteConfig, RemoteConfig};
use byi_webdav::{WebDavPreset, WebDavRemoteConfig};

use crate::app::App;
use crate::cli::{
    RemoteInitOptions, RemoteProvider, SyncCommand, sync_help_text, validate_relative_path,
    validate_repo,
};
use crate::output::format_remote_status;
use config::ByiConfig;
use prompt::{prompt, prompt_with_default, select_option};

mod config;
mod prompt;

#[cfg(test)]
mod tests;

impl App {
    fn remote_config(&self) -> Result<String, String> {
        let config = self.load_config()?;

        match config.remote {
            Some(remote) => {
                let menu = format!(
                    "{}\n\n操作选项:\na. 更改同步配置\nb. 测试同步配置\nc. 退出\n\n远端类型:\n1. github仓库\n2. webdav\n\n下一步:\n  更改 GitHub 同步配置: byi sync init --provider github --repo owner/repo --branch main --base-path .byi\n  配置坚果云 WebDAV: byi sync init --provider webdav --preset jianguoyun --username name@example.com --base-path .byi\n  配置自定义 WebDAV: byi sync init --provider webdav --preset custom --url https://example.com/dav/ --username name --base-path .byi\n  测试同步配置: byi sync test",
                    format_remote_status(&remote)
                );
                if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                    println!("{}", format_remote_status(&remote));
                    return self.handle_configured_remote_choice();
                }
                Ok(menu)
            }
            None => {
                let menu = "Sync remote is not initialized.\n\n操作选项:\na. 初始化同步配置\nb. 退出\n\n远端类型:\n1. github仓库\n2. webdav\n\n下一步:\n  初始化 GitHub 同步配置: byi sync init --provider github --repo owner/repo --branch main --base-path .byi\n  初始化坚果云 WebDAV: byi sync init --provider webdav --preset jianguoyun --username name@example.com --base-path .byi\n  初始化自定义 WebDAV: byi sync init --provider webdav --preset custom --url https://example.com/dav/ --username name --base-path .byi"
                    .to_string();
                if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                    println!("Sync remote is not initialized.");
                    return self.handle_unconfigured_remote_choice();
                }
                Ok(menu)
            }
        }
    }

    fn handle_configured_remote_choice(&self) -> Result<String, String> {
        match select_option("请选择操作", &["更改同步配置", "测试同步配置", "退出"])?
        {
            Some(0) => self.prompt_remote_type_and_init(),
            Some(1) => self.remote_test(),
            Some(2) | None => Ok("已退出。".to_string()),
            Some(index) => Err(format!("未知选项: {index}")),
        }
    }

    fn handle_unconfigured_remote_choice(&self) -> Result<String, String> {
        match select_option("请选择操作", &["初始化同步配置", "退出"])? {
            Some(0) => self.prompt_remote_type_and_init(),
            Some(1) | None => Ok("已退出。".to_string()),
            Some(index) => Err(format!("未知选项: {index}")),
        }
    }

    fn prompt_remote_type_and_init(&self) -> Result<String, String> {
        match select_option("请选择远端类型", &["github仓库", "webdav"])? {
            Some(0) => {
                if !byi_github::GitHubCli::is_installed() {
                    return Err(byi_github::GitHubCli::install_help().to_string());
                }
                let repo = prompt("GitHub repo (owner/repo): ")?;
                let branch = prompt_with_default("Branch", "main")?;
                let base_path = prompt_with_default("Base path", ".byi")?;
                self.remote_init(RemoteInitOptions::github(
                    repo.trim().to_string(),
                    branch,
                    base_path,
                ))
            }
            Some(1) => self.prompt_webdav_and_init(),
            None => Ok("已退出。".to_string()),
            Some(index) => Err(format!("不支持的远端类型: {index}")),
        }
    }

    fn prompt_webdav_and_init(&self) -> Result<String, String> {
        let preset = match select_option("请选择 WebDAV 配置方式", &["坚果云", "自定义"])?
        {
            Some(0) => "jianguoyun".to_string(),
            Some(1) => "custom".to_string(),
            None => return Ok("已退出。".to_string()),
            Some(index) => return Err(format!("不支持的 WebDAV 配置方式: {index}")),
        };
        let url = if preset == "custom" {
            Some(prompt("WebDAV URL: ")?)
        } else {
            None
        };

        let username = prompt("Username (optional): ")?;
        let username = (!username.trim().is_empty()).then_some(username);
        let base_path = prompt_with_default("Base path", ".byi")?;

        self.remote_init(RemoteInitOptions::webdav(preset, url, username, base_path))
    }

    fn remote_init(&self, options: RemoteInitOptions) -> Result<String, String> {
        let remote = match options.provider {
            RemoteProvider::GitHub => {
                let repo = options
                    .repo
                    .ok_or_else(|| "GitHub remote init 需要 --repo owner/repo。".to_string())?;
                validate_repo(&repo)?;
                validate_relative_path(&options.base_path)?;

                if self.check_github {
                    byi_github::GitHubCli::ensure_repo_access(&repo)?;
                }

                RemoteConfig::GitHub(GitHubRemoteConfig {
                    repo,
                    branch: options.branch,
                    base_path: options.base_path,
                    auth: "github-cli".to_string(),
                })
            }
            RemoteProvider::WebDav => {
                validate_relative_path(&options.base_path)?;
                let preset = options
                    .preset
                    .as_deref()
                    .map(byi_webdav::parse_preset)
                    .transpose()?
                    .unwrap_or(WebDavPreset::Jianguoyun);
                let config = match preset {
                    WebDavPreset::Jianguoyun => {
                        WebDavRemoteConfig::jianguoyun(options.username, options.base_path)
                    }
                    WebDavPreset::Custom => {
                        let url = options
                            .url
                            .ok_or_else(|| "自定义 WebDAV 需要 --url。".to_string())?;
                        byi_webdav::validate_endpoint_url(&url)?;
                        WebDavRemoteConfig::custom(url, options.username, options.base_path)
                    }
                };
                RemoteConfig::WebDav(config)
            }
        };

        self.save_config(&ByiConfig {
            remote: Some(remote.clone()),
        })?;

        Ok(format!(
            "Sync remote initialized.\n{}",
            format_remote_status(&remote)
        ))
    }

    fn remote_status(&self) -> Result<String, String> {
        let config = self.load_config()?;

        let Some(remote) = config.remote else {
            return Ok("Sync remote is not initialized.".to_string());
        };

        Ok(format_remote_status(&remote))
    }

    fn remote_test(&self) -> Result<String, String> {
        let remote = self.require_remote()?;
        byi_storage::storage_for(&remote).test()?;

        Ok("Sync remote test passed.".to_string())
    }

    pub(crate) fn run_sync(&self, command: Option<SyncCommand>) -> Result<String, String> {
        match command {
            None => Ok(sync_help_text()),
            Some(SyncCommand::Config) => self.remote_config(),
            Some(SyncCommand::Init(options)) => self.remote_init(options),
            Some(SyncCommand::Status) => self.remote_status(),
            Some(SyncCommand::Test) => self.remote_test(),
            Some(SyncCommand::Pull) => self.sync_pull(),
            Some(SyncCommand::Push) => self.sync_push(),
        }
    }

    fn sync_pull(&self) -> Result<String, String> {
        let remote = self.require_remote()?;
        let contents = byi_storage::storage_for(&remote).read_text("metadata.toml")?;
        self.write_store_file("metadata.toml", &contents)?;

        Ok(format!(
            "Pulled remote data.\n{}",
            format_remote_status(&remote)
        ))
    }

    fn sync_push(&self) -> Result<String, String> {
        let remote = self.require_remote()?;
        let metadata = self.ensure_metadata()?;
        byi_storage::storage_for(&remote).write_text("metadata.toml", &metadata)?;

        Ok(format!(
            "Pushed local data.\n{}",
            format_remote_status(&remote)
        ))
    }
}
