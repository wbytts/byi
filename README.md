# byi

`byi` 是一个基于 Cargo workspace 的本地 Rust 工具项目。仓库根目录是主命令行包，`crates/` 下放各 Rust 后端和辅助包。

## 项目结构

```text
.
├── Cargo.toml
├── src/main.rs
├── crates/
│   ├── core/
│   ├── github/
│   ├── skill/
│   ├── storage/
│   ├── webdav/
│   └── utils/
├── scripts/
│   ├── install.sh
│   └── install.ps1
└── .github/workflows/release.yml
```

## 本地开发

```bash
cargo test
cargo run -- --help
```

## 同步配置

`byi` 可以绑定远端存储，把本机数据同步到指定目录。目前远端类型包含：

- `github仓库`
- `webdav`

WebDAV 下继续选择配置方式：

- `坚果云`
- `自定义`

进入同步配置入口：

```bash
byi sync config
```

如果尚未配置，会展示：

- `a. 初始化同步配置`
- `b. 退出`

如果已经配置，会展示：

- `a. 更改同步配置`
- `b. 测试同步配置`
- `c. 退出`

GitHub 同步远端使用 GitHub CLI 处理鉴权和 API 访问。配置时会检测本机是否安装 `gh`；如果没有，会给出安装引导。安装后按提示完成：

```bash
gh auth login --web -h github.com --scopes repo
```

初始化远端：

```bash
byi sync init --provider github --repo owner/repo --branch main --base-path .byi
byi sync init --provider webdav --preset jianguoyun --username name@example.com --base-path .byi
byi sync init --provider webdav --preset custom --url https://example.com/dav/ --username name --base-path .byi
```

查看当前绑定：

```bash
byi sync status
byi sync test
```

同步数据：

```bash
byi sync pull
byi sync push
```

当前 MVP 只同步 `metadata.toml`，配置目录固定在本机 `~/.config/byi` 下。

实现边界：

- `crates/github`: GitHub 相关能力封装，包括 GitHub CLI 检测、鉴权引导、repo 检查和 Contents API 文件读写。
- `crates/storage`: 存储抽象层，隔离 GitHub、WebDAV 等具体 remote 实现。
- `crates/webdav`: WebDAV 配置能力，包括坚果云 preset 和自定义 URL。
- `crates/skill`: 本地 skill 发现、加载、实例化和扫描。

## 安装最新版本

macOS / Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/wbytts/byi/main/scripts/install.sh | sh
```

Windows PowerShell:

```powershell
iwr https://raw.githubusercontent.com/wbytts/byi/main/scripts/install.ps1 -UseB | iex
```

安装脚本会从 GitHub 最新 Release 下载匹配当前系统的资产，并安装到默认目录：

- macOS / Linux: `$HOME/.local/bin`
- Windows: `%USERPROFILE%\.local\bin`

可通过环境变量覆盖：

- `BYI_INSTALL_REPO`: GitHub 仓库，默认 `wbytts/byi`
- `BYI_INSTALL_DIR`: 安装目录
