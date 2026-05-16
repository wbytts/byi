# 构建与发布流程

这份文档描述 `byi` 当前完整的构建、GitHub Release 和 npm 发布流程，和仓库中的 `.github/workflows/release.yml` 保持一致。

## 目标产物

一次完整发布会产出两类内容：

- GitHub Release 二进制安装包
- npm 包 `@wbytts/byi`

当前 GitHub Release 会上传这些平台资产：

```text
byi-x86_64-apple-darwin.tar.gz
byi-aarch64-apple-darwin.tar.gz
byi-x86_64-unknown-linux-gnu.tar.gz
byi-aarch64-unknown-linux-gnu.tar.gz
byi-x86_64-pc-windows-msvc.zip
byi-aarch64-pc-windows-msvc.zip
```

同一次 Release workflow 还会生成一个 npm 安装包资产：

```text
wbytts-byi-<version>.tgz
```

如果启用了 npm 发布，还会把同版本发布到 npm registry：

```text
@wbytts/byi
```

## 前置条件

正式发布前需要满足：

- 待发布提交已经在默认分支历史上
- `packages/pkg/package.json` 中的 `version` 已更新到目标版本
- 发布 tag 与 npm 包版本一致
  例如 `package.json` 是 `0.0.1`，则 tag 必须是 `v0.0.1`

如果要发布到 npm，还需要在 GitHub 仓库 Actions secrets 中配置：

```text
NPM_TOKEN
```

建议使用 npm automation token。

## 本地发布前检查

发布前建议先在本地做最小验证：

```bash
cargo test
cargo run -- --help
pnpm install
pnpm --filter local-web build
```

如果只想本地验证 npm 包内容，也可以执行：

```bash
cd packages/pkg
npm pack --dry-run
```

## GitHub Actions 工作流概览

当前发布 workflow 文件：

```text
.github/workflows/release.yml
```

它有两种触发方式：

1. 推送 `v*.*.*` tag 自动触发
2. 在 GitHub Actions 页面手动触发 `Release` workflow

workflow 由 4 类 job 组成：

1. `prepare`
2. `build`
3. `release`
4. `publish-npm`

### prepare

负责解析发布参数并做发布门禁检查：

- 解析 `ref`
- 解析 `release_tag`
- 检查 `release_tag` 是否匹配 `v*.*.*`
- 检查 tag 版本是否等于 `packages/pkg/package.json` 的 `version`
- 检查待发布提交是否在默认分支历史上
- 检查启用 npm 发布时仓库是否已配置 `NPM_TOKEN`

### build

对 6 个目标平台并行构建 Rust CLI：

- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `aarch64-pc-windows-msvc`

每个平台会做两件事：

- 打包成 GitHub Release 资产
- 产出 npm 打包所需的二进制 artifact

### release

当 `publish_release=true` 时执行：

- 下载各平台 release asset artifact
- 下载各平台 npm binary artifact
- 在 `packages/pkg` 执行 `npm pack`
- 将生成的 `.tgz` 和各平台二进制一起上传到 GitHub Release

如果目标 tag 已存在，则覆盖上传资产；否则创建新的 Release。

### publish-npm

当 `publish_npm=true` 时执行：

- 下载各平台 npm binary artifact
- 在 `packages/pkg` 执行 `npm publish --access public`

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
- `publish_npm`

### 1. 只测试构建

用途：

- 验证当前提交能否在 6 个平台上通过构建
- 不创建 GitHub Release
- 不发布到 npm

填写方式：

- `ref`: 目标分支、commit 或 tag
- 其他项保持默认

### 2. 只发布 GitHub Release

用途：

- 发布原生安装包
- 生成 `wbytts-byi-<version>.tgz` 作为 Release 资产
- 不发布到 npm registry

填写方式：

- `ref`: 要发布的提交，通常为 `main`
- `release_tag`: 例如 `v0.0.1`
- `publish_release`: `true`
- `publish_npm`: `false`

### 3. 同时发布 GitHub Release 和 npm

用途：

- 发布 GitHub Release
- 发布 npm 包 `@wbytts/byi`

填写方式：

- `ref`: 要发布的提交，通常为 `main`
- `release_tag`: 例如 `v0.0.1`
- `publish_release`: `true`
- `publish_npm`: `true`

这是当前推荐的正式发布方式。

## tag 自动触发流程

也可以通过推送版本 tag 自动触发：

```bash
git tag v0.0.1
git push origin v0.0.1
```

自动触发时的行为：

- 自动创建或更新 GitHub Release
- 如果仓库配置了 `NPM_TOKEN`，还会自动发布到 npm

注意：

- 自动触发依然要求 tag 对应版本与 `packages/pkg/package.json` 的 `version` 一致
- tag 对应提交必须位于默认分支历史上

## npm 发布后的安装命令

当 npm 发布成功后，用户安装命令为：

```bash
npm install -g @wbytts/byi
```

验证：

```bash
byi --help
```

## GitHub Release 安装方式

除了 npm 安装，也支持通过安装脚本直接从 GitHub Release 获取平台二进制：

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

- `release_tag` 与 `packages/pkg/package.json` 的 `version` 不一致

修复：

- 先更新 `packages/pkg/package.json`
- 再使用匹配版本的 tag，例如 `v0.0.1`

### 2. 提交不在默认分支历史上

现象：

- workflow 在 `prepare` 失败

原因：

- 发布了一个不属于默认分支历史的 commit

修复：

- 将改动合入默认分支后再发布

### 3. 缺少 `NPM_TOKEN`

现象：

- 手动勾选 `publish_npm=true` 后在 `prepare` 失败

原因：

- 仓库没有配置 `NPM_TOKEN`

修复：

- 在仓库 Actions secrets 中新增 `NPM_TOKEN`

### 4. 构建平台失败

现象：

- `build` 某个平台 job 失败

原因通常是：

- 目标平台依赖缺失
- Rust target 构建错误
- 平台特定路径或权限问题

修复：

- 先在对应 job 日志中确认具体失败步骤
- 修复后重新执行 workflow

### 5. npm 发布失败

现象：

- `publish-npm` 失败

常见原因：

- `NPM_TOKEN` 无效
- 当前版本已经发布过
- npm 权限或 package scope 配置不正确

修复：

- 校验 token 是否有效
- 检查 npm 上该版本是否已存在
- 必要时提升版本后重发

## 当前推荐发布清单

每次正式发布前，按这个顺序操作：

1. 更新代码并合入默认分支
2. 更新 `packages/pkg/package.json` 的版本号
3. 本地执行最小验证
4. 确认仓库已配置 `NPM_TOKEN`
5. 在 Actions 手动触发 `Release`
6. 设置 `ref=main`
7. 设置 `release_tag=vX.Y.Z`
8. 打开 `publish_release=true`
9. 打开 `publish_npm=true`
10. 等待 workflow 全绿
11. 验证 GitHub Release 资产
12. 验证 `npm view @wbytts/byi version`

## 结果验证命令

发布完成后可用以下命令确认结果：

查看 npm 最新版本：

```bash
npm view @wbytts/byi version
```

查看 npm dist-tag：

```bash
npm view @wbytts/byi dist-tags --json
```

查看 GitHub Release：

```bash
gh release view v0.0.1
```
