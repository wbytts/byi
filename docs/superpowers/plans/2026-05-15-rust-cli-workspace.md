# Rust CLI Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 初始化一个根包为 CLI、`crates/` 为辅助包的 Cargo workspace，并提供 GitHub Release 最新版安装脚本。

**Architecture:** 根目录 `byi` 是可执行主包，`crates/core` 提供可复用逻辑。安装脚本通过 GitHub latest release asset URL 下载当前平台资产，Release workflow 在 tag 推送时构建并上传资产。

**Tech Stack:** Rust 2024 edition、Cargo workspace、POSIX shell、PowerShell、GitHub Actions。

---

### Task 1: Workspace and CLI

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`
- Modify: `crates/core/src/lib.rs`

- [x] 创建 root package + workspace 配置，并让主包依赖 `byi-core`。
- [x] 实现 `byi`、`byi hello`、`byi --help`、`byi --version` 的最小 CLI 行为。
- [x] 为 CLI 命令解析和 core 输出补单元测试。

### Task 2: Installer Scripts

**Files:**
- Create: `scripts/install.sh`
- Create: `scripts/install.ps1`

- [x] 实现 macOS / Linux 平台识别和 latest release tar.gz 下载安装，平台范围与发布资产保持一致。
- [x] 实现 Windows PowerShell 平台识别和 latest release zip 下载安装，平台范围与发布资产保持一致。
- [x] 支持 `BYI_INSTALL_REPO` 和 `BYI_INSTALL_DIR` 覆盖。

### Task 3: GitHub Release Workflow and Docs

**Files:**
- Create: `.github/workflows/release.yml`
- Create: `README.md`

- [x] 配置 tag `v*.*.*` 触发 Release 构建，使用 GitHub CLI 创建 Release。
- [x] 构建 macOS、Linux、Windows 的 x86_64 和 aarch64 资产。
- [x] 文档说明本地开发、安装和发布方式。

### Task 4: Verification

**Files:**
- Verify: whole workspace

- [x] 运行 `cargo fmt --check`。
- [x] 运行 `cargo test --workspace`。
- [x] 运行 `cargo build --release --locked`。
