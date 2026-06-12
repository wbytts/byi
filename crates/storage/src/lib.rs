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
    fn read_file(&self, file_name: &str) -> Result<Vec<u8>, String>;
    fn write_file(&self, file_name: &str, contents: &[u8]) -> Result<(), String>;
    fn delete_file(&self, file_name: &str) -> Result<(), String>;
    fn test(&self) -> Result<(), String>;
}

pub struct GitHubStorage {
    config: GitHubRemoteConfig,
}

impl GitHubStorage {
    pub fn new(config: GitHubRemoteConfig) -> Self {
        Self { config }
    }

    fn github_remote(&self) -> byi_github::GitHubRemote {
        byi_github::GitHubRemote {
            repo: self.config.repo.clone(),
            branch: self.config.branch.clone(),
            base_path: self.config.base_path.clone(),
        }
    }
}

impl RemoteStorage for GitHubStorage {
    fn read_file(&self, file_name: &str) -> Result<Vec<u8>, String> {
        let remote_path = join_remote_path(&self.config.base_path, file_name);
        byi_github::GitHubCli::get_file_bytes(&self.github_remote(), &remote_path)
    }

    fn write_file(&self, file_name: &str, contents: &[u8]) -> Result<(), String> {
        let remote_path = join_remote_path(&self.config.base_path, file_name);
        byi_github::GitHubCli::put_file_bytes(&self.github_remote(), &remote_path, contents)
    }

    fn delete_file(&self, file_name: &str) -> Result<(), String> {
        let remote_path = join_remote_path(&self.config.base_path, file_name);
        byi_github::GitHubCli::delete_file(&self.github_remote(), &remote_path)
    }

    fn test(&self) -> Result<(), String> {
        byi_github::GitHubCli::ensure_repo_access(&self.config.repo)
    }
}

pub struct WebDavStorage {
    client: byi_webdav::WebDavClient,
}

impl WebDavStorage {
    pub fn new(config: byi_webdav::WebDavRemoteConfig) -> Self {
        Self {
            client: byi_webdav::WebDavClient::new(config),
        }
    }

    fn ensure_collection_path(&self, file_name: &str) -> Result<(), String> {
        let path = std::path::Path::new(file_name);
        let mut current = String::new();

        for component in path.components() {
            let std::path::Component::Normal(part) = component else {
                continue;
            };
            let part = part.to_string_lossy();
            if current.is_empty() {
                current.push_str(&part);
            } else {
                current.push('/');
                current.push_str(&part);
            }
        }

        if let Some((parent, _)) = current.rsplit_once('/') {
            let mut prefix = String::new();
            for segment in parent.split('/') {
                if prefix.is_empty() {
                    prefix.push_str(segment);
                } else {
                    prefix.push('/');
                    prefix.push_str(segment);
                }
                let _ = self.client.mkcol(&prefix);
            }
        }

        Ok(())
    }
}

impl RemoteStorage for WebDavStorage {
    fn read_file(&self, file_name: &str) -> Result<Vec<u8>, String> {
        self.client.get(file_name)
    }

    fn write_file(&self, file_name: &str, contents: &[u8]) -> Result<(), String> {
        self.ensure_collection_path(file_name)?;
        self.client.put(file_name, contents)
    }

    fn delete_file(&self, file_name: &str) -> Result<(), String> {
        self.client.delete(file_name)
    }

    fn test(&self) -> Result<(), String> {
        self.client.test()
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
    let file_name = file_name.trim_matches('/');

    if base_path.is_empty() {
        file_name.to_string()
    } else if file_name.is_empty() {
        base_path.to_string()
    } else {
        format!("{base_path}/{file_name}")
    }
}
