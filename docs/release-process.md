# 构建与发布流程

这份文档描述 `byi` 当前纯 Rust 的构建与 GitHub Release 流程，和仓库中的 `.github/workflows/release.yml` 保持一致。

## 目标产物

一次完整发布会产出 GitHub Release 二进制安装包：

```text
byi-x86_64-apple-darwin.tar.gz
byi-aarch64-apple-darwin.tar.gz
byi-x86_64-unknown-linux-gnu.tar.gz
byi-aarch64-unknown-linux-gnu.tar.gz
byi-x86_64-pc-windows-msvc.zip
byi-aarch64-pc-windows-msvc.zip
```

## 前置条件

正式发布前需要满足：

- 待发布提交已经在默认分支历史上
- 发布 tag 与 `Cargo.toml` 中的 `version` 一致
  例如 `Cargo.toml` 是 `0.0.1`，则 tag 必须是 `v0.0.1`

## 本地发布前检查

发布前建议先在本地做最小验证：

```bash
cargo test
cargo run -- --help
```

## GitHub Actions 工作流概览

当前发布 workflow 文件：

```text
.github/workflows/release.yml
```

它有两种触发方式：

1. 推送 `v*.*.*` tag 自动触发
2. 在 GitHub Actions 页面手动触发 `Release` workflow

workflow 由 3 类 job 组成：

1. `prepare`
2. `build`
3. `release`

### prepare

负责解析发布参数并做发布门禁检查：

- 解析 `ref`
- 解析 `release_tag`
- 检查 `release_tag` 是否匹配 `v*.*.*`
- 检查 tag 版本是否等于 `Cargo.toml` 的 `version`
- 检查待发布提交是否在默认分支历史上

### build

对 6 个目标平台并行构建 Rust CLI：

- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `aarch64-pc-windows-msvc`

每个平台会打包成 GitHub Release 资产并上传。

### release

当 `publish_release=true` 时执行：

- 下载各平台 release asset artifact
- 将各平台二进制一起上传到 GitHub Release

如果目标 tag 已存在，则覆盖上传资产；否则创建新的 Release。

## 手动操作步骤

推荐使用手动触发 workflow 的方式发布。

进入：

```text
GitHub -> Actions -> Release -> Run workflow
```

可填写的输入：

- `ref`
- `release_tag`
- `publish_release`

### 1. 只测试构建

用途：

- 验证当前提交能否在 6 个平台上通过构建
- 不创建 GitHub Release

填写方式：

- `ref`: 目标分支、commit 或 tag
- 其他项保持默认

### 2. 发布 GitHub Release

用途：

- 发布原生安装包

填写方式：

- `ref`: 要发布的提交，通常为 `main`
- `release_tag`: 例如 `v0.0.1`
- `publish_release`: `true`

## tag 自动触发流程

也可以通过推送版本 tag 自动触发：

```bash
git tag v0.0.1
git push origin v0.0.1
```

自动触发时的行为：

- 自动创建或更新 GitHub Release

注意：

- 自动触发依然要求 tag 对应版本与 `Cargo.toml` 的 `version` 一致
- tag 对应提交必须位于默认分支历史上

## GitHub Release 安装方式

支持通过安装脚本直接从 GitHub Release 获取平台二进制：

macOS / Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/wbytts/byi/main/scripts/install.sh | sh
```

Windows PowerShell:

```powershell
iwr https://raw.githubusercontent.com/wbytts/byi/main/scripts/install.ps1 -UseB | iex
```

## 常见失败点

### 1. 版本不一致

现象：

- workflow 在 `prepare` 失败

原因：

- `release_tag` 与 `Cargo.toml` 的 `version` 不一致

修复：

- 先更新 `Cargo.toml`
- 再使用匹配版本的 tag，例如 `v0.0.1`

### 2. 提交不在默认分支历史上

现象：

- workflow 在 `prepare` 失败

原因：

- 发布了一个不属于默认分支历史的 commit

修复：

- 将改动合入默认分支后再发布

### 3. 构建平台失败

现象：

- `build` 某个平台 job 失败

原因通常是：

- 目标平台依赖缺失
- Rust target 构建错误
- 平台特定路径或权限问题

排查方向：

- 检查对应 runner 的 Rust 工具链是否完整
- 查看该平台的 `cargo build --release --locked --target <target>` 输出
- 确认跨平台依赖（如 OpenSSL、Windows SDK）已正确安装

## 发布后的验证

发布完成后，可以从 GitHub Release 页面下载对应平台的安装包，或者通过安装脚本安装，然后执行：

```bash
byi --help
byi sync status
```

确认二进制可正常启动且功能可用。
