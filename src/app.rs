use std::path::PathBuf;

use crate::cli::{Command, WebCommand, parse_cli};

pub(crate) struct App {
    pub(crate) config_dir: PathBuf,
    pub(crate) check_github: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            config_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("byi"),
            check_github: true,
        }
    }
}

impl App {
    pub(crate) fn run(&self, args: impl IntoIterator<Item = String>) -> Result<String, String> {
        let cli = match parse_cli(args) {
            Ok(cli) => cli,
            Err(message) if is_clap_display_message(&message) => {
                return Ok(message.trim_end().to_string());
            }
            Err(message) => return Err(message),
        };

        match cli.command {
            None | Some(Command::Hello) => Ok(byi_core::hello_message()),
            Some(Command::Sync { command }) => self.run_sync(command),
            Some(Command::Web(command)) => self.run_web(command),
        }
    }

    fn run_web(&self, command: WebCommand) -> Result<String, String> {
        let frontend_dir = command
            .frontend_dir
            .unwrap_or_else(default_frontend_dist_dir);
        if !command.no_build {
            build_frontend_if_needed(&frontend_dir)?;
        }

        let options = byi_server::WebServerOptions {
            host: command.host,
            port: command.port,
            frontend_dir,
            config_dir: self.config_dir.clone(),
        };
        let bind_addr = options.bind_addr()?;
        println!("BYI web is running at http://{bind_addr}");
        println!("API health: http://{bind_addr}/api/health");
        println!("Press Ctrl+C to stop.");

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|err| format!("创建 Tokio runtime 失败: {err}"))?;
        runtime.block_on(byi_server::serve(options))?;
        Ok("BYI web stopped.".to_string())
    }
}

fn is_clap_display_message(message: &str) -> bool {
    !message.starts_with("error:")
        && (message.starts_with("Usage:")
            || message.starts_with("byi ")
            || message.contains("\nUsage:"))
}

fn default_frontend_dist_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packages/local-web/dist")
}

fn build_frontend_if_needed(frontend_dir: &std::path::Path) -> Result<(), String> {
    if frontend_dir.join("index.html").exists() {
        return Ok(());
    }

    let package_dir = frontend_dir
        .parent()
        .ok_or_else(|| format!("无法推断前端项目目录: {}", frontend_dir.display()))?;
    if !package_dir.join("package.json").exists() {
        return Err(format!(
            "前端构建产物不存在，且找不到前端项目: {}",
            package_dir.display()
        ));
    }

    let status = std::process::Command::new("pnpm")
        .arg("--dir")
        .arg(package_dir)
        .arg("build")
        .status()
        .map_err(|err| format!("执行 pnpm build 失败: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("前端构建失败，退出码: {status}"))
    }
}
