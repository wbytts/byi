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
