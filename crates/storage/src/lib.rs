#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "provider", rename_all = "kebab-case")]
pub enum RemoteConfig {
    #[serde(rename = "github")]
    GitHub(GitHubRemoteConfig),
    #[serde(rename = "webdav")]
    WebDav(byi_webdav::WebDavRemoteConfig),
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GitHubRemoteConfig {
    pub repo: String,
    pub branch: String,
    pub base_path: String,
    pub auth: String,
}

pub trait RemoteStorage {
    fn read_text(&self, file_name: &str) -> Result<String, String>;
    fn write_text(&self, file_name: &str, contents: &str) -> Result<(), String>;
    fn test(&self) -> Result<(), String>;
}

pub struct GitHubStorage {
    config: GitHubRemoteConfig,
}

impl GitHubStorage {
    pub fn new(config: GitHubRemoteConfig) -> Self {
        Self { config }
    }
}

impl RemoteStorage for GitHubStorage {
    fn read_text(&self, file_name: &str) -> Result<String, String> {
        let remote_path = join_remote_path(&self.config.base_path, file_name);
        byi_github::GitHubCli::get_file(&self.github_remote(), &remote_path)
    }

    fn write_text(&self, file_name: &str, contents: &str) -> Result<(), String> {
        let remote_path = join_remote_path(&self.config.base_path, file_name);
        byi_github::GitHubCli::put_file(&self.github_remote(), &remote_path, contents)
    }

    fn test(&self) -> Result<(), String> {
        byi_github::GitHubCli::ensure_repo_access(&self.config.repo)
    }
}

impl GitHubStorage {
    fn github_remote(&self) -> byi_github::GitHubRemote {
        byi_github::GitHubRemote {
            repo: self.config.repo.clone(),
            branch: self.config.branch.clone(),
            base_path: self.config.base_path.clone(),
        }
    }
}

pub struct WebDavStorage {
    config: byi_webdav::WebDavRemoteConfig,
}

impl WebDavStorage {
    pub fn new(config: byi_webdav::WebDavRemoteConfig) -> Self {
        Self { config }
    }
}

impl RemoteStorage for WebDavStorage {
    fn read_text(&self, _file_name: &str) -> Result<String, String> {
        Err(format!(
            "WebDAV remote is not implemented yet: {}",
            self.config.endpoint_url
        ))
    }

    fn write_text(&self, _file_name: &str, _contents: &str) -> Result<(), String> {
        Err(format!(
            "WebDAV remote is not implemented yet: {}",
            self.config.endpoint_url
        ))
    }

    fn test(&self) -> Result<(), String> {
        Err(format!(
            "WebDAV remote is not implemented yet: {}",
            self.config.endpoint_url
        ))
    }
}

pub fn storage_for(remote: &RemoteConfig) -> Box<dyn RemoteStorage> {
    match remote {
        RemoteConfig::GitHub(config) => Box::new(GitHubStorage::new(config.clone())),
        RemoteConfig::WebDav(config) => Box::new(WebDavStorage::new(config.clone())),
    }
}

pub fn join_remote_path(base_path: &str, file_name: &str) -> String {
    let base_path = base_path.trim_matches('/');

    if base_path.is_empty() {
        file_name.to_string()
    } else {
        format!("{base_path}/{file_name}")
    }
}
