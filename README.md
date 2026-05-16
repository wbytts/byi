# byi

`byi` 是一个基于 Cargo workspace + pnpm workspace 的本地工具项目。仓库根目录是主命令行包，`crates/` 下放 Rust 后端和辅助包，`packages/` 下放前端相关项目。

## 项目结构

```text
.
├── Cargo.toml
├── package.json
├── pnpm-workspace.yaml
├── src/main.rs
├── crates/
│   ├── core/
│   ├── github/
│   ├── server/
│   ├── storage/
│   ├── webdav/
│   └── utils/
├── packages/
│   ├── pkg/
│   └── local-web/
├── scripts/
│   ├── install.sh
│   └── install.ps1
└── .github/workflows/release.yml
```

## 本地开发

```bash
cargo test
cargo run -- --help
pnpm install
pnpm --filter local-web build
```

## 本地 Web

`byi web` 会启动一个本地 Axum 服务，默认监听 `127.0.0.1:3768`。如果 `packages/local-web/dist` 不存在，会先执行 `pnpm --dir packages/local-web build` 构建 React 静态页面，然后由后端服务同源托管前端页面和 API。

```bash
cargo run -- web
byi web
```

打开：

```text
http://127.0.0.1:3768
```

当前 Web API：

```text
GET /api/health
GET /api/info
```

`packages/local-web` 技术栈：

- React + TypeScript + Vite
- Ant Design
- Tailwind CSS
- Zustand

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
- `crates/server`: Axum 本地后端，提供 `/api/*` 接口并托管 `packages/local-web` 构建后的静态资源。
- `crates/storage`: 存储抽象层，隔离 GitHub、WebDAV 等具体 remote 实现。
- `crates/webdav`: WebDAV 配置能力，包括坚果云 preset 和自定义 URL。

## 安装最新版本

### npm

如果通过 npm 分发，需要先在仓库根目录准备 Rust 构建产物，再从 `packages/pkg` 发包：

```bash
cargo build --release
pnpm --dir packages/pkg pack
pnpm --dir packages/pkg publish
```

安装后会通过 npm 包内携带的 Rust 二进制执行 `byi`：

```bash
npm install -g @wbytts/byi
byi --help
```

如果要让同一个 npm 包同时包含多个平台，需要先分别构建对应 target，再执行 `pack` / `publish`。

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

## 发布

推荐通过 GitHub Actions 页面手动触发 `Release` workflow 发布安装包。也保留了 `v*.*.*` tag push 自动触发。

```bash
Actions -> Release -> Run workflow
```

GitHub Release 会附带这些原生安装资产：

```text
byi-x86_64-apple-darwin.tar.gz
byi-aarch64-apple-darwin.tar.gz
byi-x86_64-unknown-linux-gnu.tar.gz
byi-aarch64-unknown-linux-gnu.tar.gz
byi-x86_64-pc-windows-msvc.zip
byi-aarch64-pc-windows-msvc.zip
```

同一次 workflow 还会生成一个汇总 npm 安装包资产：

```text
wbytts-byi-0.0.1.tgz
```

如果仓库配置了 `NPM_TOKEN` secret，workflow 还会自动把 `packages/pkg` 发布到 npm。发布 job 会先汇总各平台 runner 构建出的 Rust 二进制，再打成同一个 npm 包。

发布约束：

- 正式发布只允许来自默认分支历史上的提交
- Git tag 版本必须和 `packages/pkg/package.json` 的 `version` 一致
- `workflow_dispatch` 可用于手动测试构建；只有显式打开 `publish_release` 或 `publish_npm` 时才会进入发布阶段

手动触发时的常用输入：

- `ref`: 要构建的分支、commit 或 tag；默认当前分支
- `release_tag`: 正式发布版本，例如 `v0.0.1`
- `publish_release`: 是否创建或更新 GitHub Release
- `publish_npm`: 是否发布到 npm

常见用法：

- 只测试构建：只填 `ref`
- 发布 GitHub Release：填写 `ref` 和 `release_tag`，打开 `publish_release`
- 同时发布 npm：再额外打开 `publish_npm`
