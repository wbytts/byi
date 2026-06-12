use std::fs;

use byi_storage::RemoteConfig;

use crate::app::App;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub(super) struct ByiConfig {
    pub(super) remote: Option<RemoteConfig>,
}

impl App {
    pub(super) fn load_config(&self) -> Result<ByiConfig, String> {
        let path = self.config_dir.join("config.toml");

        if !path.exists() {
            return Ok(ByiConfig::default());
        }

        let contents = fs::read_to_string(&path)
            .map_err(|err| format!("读取配置失败 {}: {err}", path.display()))?;
        toml::from_str(&contents).map_err(|err| format!("解析配置失败 {}: {err}", path.display()))
    }

    pub(super) fn save_config(&self, config: &ByiConfig) -> Result<(), String> {
        fs::create_dir_all(&self.config_dir)
            .map_err(|err| format!("创建配置目录失败 {}: {err}", self.config_dir.display()))?;
        let path = self.config_dir.join("config.toml");
        let contents = toml::to_string_pretty(config)
            .map_err(|err| format!("序列化配置失败 {}: {err}", path.display()))?;
        fs::write(&path, contents).map_err(|err| format!("写入配置失败 {}: {err}", path.display()))
    }

    pub(crate) fn require_remote(&self) -> Result<RemoteConfig, String> {
        self.load_config()?.remote.ok_or_else(|| {
            "Sync remote is not initialized. Run `byi sync config` first.".to_string()
        })
    }
}
