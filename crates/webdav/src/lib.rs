use reqwest::blocking::Client;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WebDavPreset {
    Jianguoyun,
    Custom,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct WebDavRemoteConfig {
    pub preset: WebDavPreset,
    pub endpoint_url: String,
    pub username: Option<String>,
    pub base_path: String,
}

impl WebDavRemoteConfig {
    pub fn jianguoyun(username: Option<String>, base_path: String) -> Self {
        Self {
            preset: WebDavPreset::Jianguoyun,
            endpoint_url: jianguoyun_endpoint().to_string(),
            username,
            base_path,
        }
    }

    pub fn custom(endpoint_url: String, username: Option<String>, base_path: String) -> Self {
        Self {
            preset: WebDavPreset::Custom,
            endpoint_url,
            username,
            base_path,
        }
    }

    pub fn endpoint_for(&self, file_name: &str) -> String {
        let base = self.endpoint_url.trim_end_matches('/');
        let file_name = file_name.trim_matches('/');
        let base_path = self.base_path.trim_matches('/');

        let remote = if base_path.is_empty() {
            file_name.to_string()
        } else if file_name.is_empty() {
            base_path.to_string()
        } else {
            format!("{base_path}/{file_name}")
        };
        format!("{base}/{remote}")
    }
}

pub struct WebDavClient {
    client: Client,
    config: WebDavRemoteConfig,
}

impl WebDavClient {
    pub fn new(config: WebDavRemoteConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn config(&self) -> &WebDavRemoteConfig {
        &self.config
    }

    pub fn get(&self, path: &str) -> Result<Vec<u8>, String> {
        let url = self.config.endpoint_for(path);
        let mut request = self.client.request(reqwest::Method::GET, url);
        if let Some(username) = &self.config.username {
            request = request.basic_auth(username, Some(""));
        }
        let response = request
            .send()
            .map_err(|e| format!("WebDAV GET 请求失败: {e}"))?;
        if response.status().is_success() {
            response
                .bytes()
                .map_err(|e| format!("读取 WebDAV 响应体失败: {e}"))
                .map(|b| b.to_vec())
        } else {
            Err(format!("WebDAV GET 失败，状态码: {}", response.status()))
        }
    }

    pub fn put(&self, path: &str, contents: &[u8]) -> Result<(), String> {
        let url = self.config.endpoint_for(path);
        let mut request = self
            .client
            .request(reqwest::Method::PUT, url)
            .body(contents.to_vec());
        if let Some(username) = &self.config.username {
            request = request.basic_auth(username, Some(""));
        }
        let response = request
            .send()
            .map_err(|e| format!("WebDAV PUT 请求失败: {e}"))?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().unwrap_or_default();
            Err(format!("WebDAV PUT 失败，状态码 {status}: {body}"))
        }
    }

    pub fn delete(&self, path: &str) -> Result<(), String> {
        let url = self.config.endpoint_for(path);
        let mut request = self.client.request(reqwest::Method::DELETE, url);
        if let Some(username) = &self.config.username {
            request = request.basic_auth(username, Some(""));
        }
        let response = request
            .send()
            .map_err(|e| format!("WebDAV DELETE 请求失败: {e}"))?;
        let status = response.status();
        if status.is_success() || status == reqwest::StatusCode::NOT_FOUND {
            Ok(())
        } else {
            let body = response.text().unwrap_or_default();
            Err(format!("WebDAV DELETE 失败，状态码 {status}: {body}"))
        }
    }

    pub fn mkcol(&self, path: &str) -> Result<(), String> {
        let url = self.config.endpoint_for(path);
        let method = reqwest::Method::from_bytes(b"MKCOL")
            .map_err(|e| format!("构建 MKCOL 方法失败: {e}"))?;
        let mut request = self.client.request(method, url);
        if let Some(username) = &self.config.username {
            request = request.basic_auth(username, Some(""));
        }
        let response = request
            .send()
            .map_err(|e| format!("WebDAV MKCOL 请求失败: {e}"))?;
        let status = response.status();
        if status == reqwest::StatusCode::CREATED
            || status == reqwest::StatusCode::METHOD_NOT_ALLOWED
        {
            Ok(())
        } else {
            let body = response.text().unwrap_or_default();
            Err(format!("WebDAV MKCOL 失败，状态码 {status}: {body}"))
        }
    }

    pub fn test(&self) -> Result<(), String> {
        let url = self.config.endpoint_url.trim_end_matches('/');
        let method = reqwest::Method::from_bytes(b"PROPFIND")
            .map_err(|e| format!("构建 PROPFIND 方法失败: {e}"))?;
        let mut request = self.client.request(method, url);
        if let Some(username) = &self.config.username {
            request = request.basic_auth(username, Some(""));
        }
        let response = request
            .header("Depth", "0")
            .send()
            .map_err(|e| format!("WebDAV 测试请求失败: {e}"))?;
        let status = response.status();
        if status.is_success() || status == reqwest::StatusCode::MULTI_STATUS {
            Ok(())
        } else {
            let body = response.text().unwrap_or_default();
            Err(format!("WebDAV 测试失败，状态码 {status}: {body}"))
        }
    }
}

pub fn jianguoyun_endpoint() -> &'static str {
    "https://dav.jianguoyun.com/dav/"
}

pub fn parse_preset(value: &str) -> Result<WebDavPreset, String> {
    match value {
        "jianguoyun" | "坚果云" => Ok(WebDavPreset::Jianguoyun),
        "custom" | "自定义" => Ok(WebDavPreset::Custom),
        value => Err(format!("不支持的 WebDAV 配置方式: {value}")),
    }
}

pub fn validate_endpoint_url(value: &str) -> Result<(), String> {
    if value.starts_with("https://") || value.starts_with("http://") {
        Ok(())
    } else {
        Err("WebDAV URL 必须以 http:// 或 https:// 开头。".to_string())
    }
}
