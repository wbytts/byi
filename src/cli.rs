use clap::{Args, Parser, Subcommand, ValueEnum, error::ErrorKind};

#[derive(Debug, Parser)]
#[command(
    name = "byi",
    version,
    disable_help_subcommand = true,
    color = clap::ColorChoice::Never,
    after_help = "Examples:\n  byi web\n  byi sync config\n  byi sync init --provider github --repo owner/repo --branch main --base-path .byi\n  byi sync init --provider webdav --preset jianguoyun --username name@example.com --base-path .byi\n  byi sync init --provider webdav --preset custom --url https://example.com/dav/ --username name --base-path .byi\n  byi sync status\n  byi sync test\n  byi sync pull\n  byi sync push"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    #[command(about = "输出问候信息")]
    Hello,
    #[command(
        about = "配置远端并同步本地数据",
        after_help = "Remote providers:\n  github      GitHub 仓库，使用 GitHub CLI 鉴权\n  webdav      WebDAV；配置方式包含坚果云和自定义\n\nWebDAV presets:\n  jianguoyun  坚果云 WebDAV\n  custom      自定义 WebDAV URL\n\nExamples:\n  byi sync config\n  byi sync init --provider github --repo owner/repo --branch main --base-path .byi\n  byi sync init --provider webdav --preset jianguoyun --username name@example.com --base-path .byi\n  byi sync init --provider webdav --preset custom --url https://example.com/dav/ --username name --base-path .byi\n  byi sync status\n  byi sync test\n  byi sync pull\n  byi sync push"
    )]
    Sync {
        #[command(subcommand)]
        command: Option<SyncCommand>,
    },
    #[command(about = "启动本地 Web 控制台")]
    Web(WebCommand),
}

#[derive(Clone, Debug, Args)]
pub(crate) struct WebCommand {
    #[arg(long, default_value = "127.0.0.1")]
    pub(crate) host: String,
    #[arg(long, default_value_t = 3768)]
    pub(crate) port: u16,
    #[arg(long)]
    pub(crate) frontend_dir: Option<std::path::PathBuf>,
    #[arg(long)]
    pub(crate) no_build: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SyncCommand {
    #[command(about = "打开同步配置菜单；未配置时可初始化，已配置时可更改或测试")]
    Config,
    #[command(about = "写入同步配置")]
    Init(RemoteInitOptions),
    #[command(about = "查看当前同步配置")]
    Status,
    #[command(about = "测试当前同步远端是否可访问")]
    Test,
    #[command(about = "从当前同步远端拉取数据到本地")]
    Pull,
    #[command(about = "将本地数据推送到当前同步远端")]
    Push,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum RemoteProvider {
    #[value(name = "github", alias = "github仓库")]
    GitHub,
    #[value(name = "webdav")]
    WebDav,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct RemoteInitOptions {
    #[arg(long, alias = "type", value_enum, default_value_t = RemoteProvider::GitHub)]
    pub(crate) provider: RemoteProvider,
    #[arg(long)]
    pub(crate) repo: Option<String>,
    #[arg(long, default_value = "main")]
    pub(crate) branch: String,
    #[arg(long, default_value = ".byi")]
    pub(crate) base_path: String,
    #[arg(long)]
    pub(crate) preset: Option<String>,
    #[arg(long)]
    pub(crate) url: Option<String>,
    #[arg(long)]
    pub(crate) username: Option<String>,
}

impl RemoteInitOptions {
    pub(crate) fn github(repo: String, branch: String, base_path: String) -> Self {
        Self {
            provider: RemoteProvider::GitHub,
            repo: Some(repo),
            branch,
            base_path,
            preset: None,
            url: None,
            username: None,
        }
    }

    pub(crate) fn webdav(
        preset: String,
        url: Option<String>,
        username: Option<String>,
        base_path: String,
    ) -> Self {
        Self {
            provider: RemoteProvider::WebDav,
            repo: None,
            branch: "main".to_string(),
            base_path,
            preset: Some(preset),
            url,
            username,
        }
    }
}

pub(crate) fn parse_cli(args: impl IntoIterator<Item = String>) -> Result<Cli, String> {
    let args = std::iter::once("byi".to_string()).chain(args);

    match Cli::try_parse_from(args) {
        Ok(cli) => Ok(cli),
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            Err(error.to_string())
        }
        Err(error) => Err(error.to_string()),
    }
}

pub(crate) fn sync_help_text() -> String {
    Cli::try_parse_from(["byi", "sync", "--help"])
        .expect_err("sync --help should render help")
        .to_string()
        .trim_end()
        .to_string()
}

pub(crate) fn validate_repo(repo: &str) -> Result<(), String> {
    let parts: Vec<_> = repo.split('/').collect();

    if parts.len() == 2 && parts.iter().all(|part| !part.trim().is_empty()) {
        Ok(())
    } else {
        Err("GitHub repo 必须使用 owner/repo 格式。".to_string())
    }
}

pub(crate) fn validate_relative_path(path: &str) -> Result<(), String> {
    let path = std::path::Path::new(path);

    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
    {
        return Err("--base-path 必须是仓库内相对路径，不能包含 ..。".to_string());
    }

    Ok(())
}
