# byi 命令使用指南

本文档整理 `byi` 命令行工具的所有子命令、参数和典型用法。

## 目录

- [全局用法](#全局用法)
- [hello](#hello)
- [sync](#sync)
- [skill](#skill)
- [tui](#tui)
- [数据目录](#数据目录)
- [快速示例](#快速示例)

---

## 全局用法

```text
byi [COMMAND]
```

顶层命令：

| 命令 | 说明 |
|---|---|
| `hello` | 输出问候信息 |
| `sync` | 配置远端并同步本地数据 |
| `skill` | 管理本地 skill（别名：`byi skills`） |
| `tui` | 打开 TUI 界面 |

全局选项：

| 选项 | 说明 |
|---|---|
| `-h`, `--help` | 打印帮助信息 |
| `-V`, `--version` | 打印版本号 |

---

## hello

输出问候信息。

```bash
byi hello
```

---

## sync

管理同步远端，把本地数据同步到远端存储。

```text
byi sync [COMMAND]
```

### 子命令

| 子命令 | 说明 |
|---|---|
| `config` | 打开同步配置菜单；未配置时可初始化，已配置时可更改或测试 |
| `init` | 写入同步配置 |
| `status` | 查看当前同步配置 |
| `test` | 测试当前同步远端是否可访问 |
| `pull` | 从当前同步远端拉取数据到本地 |
| `push` | 将本地数据推送到当前同步远端 |

### sync config

交互式配置入口。

```bash
byi sync config
```

- 尚未配置时：提示初始化同步配置或退出
- 已配置时：提示更改同步配置、测试配置或退出

### sync init

写入同步配置。支持两种远端类型：`github` 和 `webdav`。

```text
byi sync init [OPTIONS]
```

| 选项 | 说明 | 默认值 |
|---|---|---|
| `--provider <PROVIDER>` | 远端类型：`github` 或 `webdav` | `github` |
| `--repo <REPO>` | GitHub 仓库，格式 `owner/repo` | — |
| `--branch <BRANCH>` | GitHub 分支 | `main` |
| `--base-path <BASE_PATH>` | 仓库内相对路径 | `.byi` |
| `--preset <PRESET>` | WebDAV 配置方式：`jianguoyun` 或 `custom` | — |
| `--url <URL>` | 自定义 WebDAV URL | — |
| `--username <USERNAME>` | WebDAV 用户名 | — |

配置 GitHub 远端：

```bash
byi sync init --provider github --repo owner/repo --branch main --base-path .byi
```

配置坚果云 WebDAV：

```bash
byi sync init --provider webdav --preset jianguoyun --username name@example.com --base-path .byi
```

配置自定义 WebDAV：

```bash
byi sync init --provider webdav --preset custom \
  --url https://example.com/dav/ \
  --username name \
  --base-path .byi
```

### sync status

查看当前同步配置。

```bash
byi sync status
```

### sync test

测试当前同步远端是否可访问。

```bash
byi sync test
```

### sync pull

从远端拉取数据到本地。

```bash
byi sync pull
```

### sync push

将本地数据推送到远端。

```bash
byi sync push
```

---

## skill

管理本地 skill。skill 真实落地在数据目录的 `skills/` 下，注册表保存在 `registry/` 下。

```text
byi skill [COMMAND]
```

### 子命令

| 子命令 | 说明 | 别名 |
|---|---|---|
| `add` | 添加本地或 GitHub skill | — |
| `list` | 列出当前 skill 实例 | `ls` |
| `view` | 查看 skill 详情 | — |
| `edit` | 编辑 skill 元数据 | — |
| `remove` | 删除某个本地实例 | `rm` |
| `enable` | 启用某个实例 | — |
| `disable` | 停用某个实例 | — |
| `instances` | 查看实例级别详情 | — |
| `doctor` | 检查 skill 管理状态 | — |
| `rescan` | 重新扫描并修正 skill 注册表 | — |

### skill add

把 skill 纳入本地管理体系。

```text
byi skill add [OPTIONS] [PATH]
```

| 选项 / 参数 | 说明 |
|---|---|
| `[PATH]` | 本地 skill 目录路径 |
| `--github <GITHUB>` | GitHub 仓库，格式 `owner/repo` |
| `--ref <REF>` | GitHub 分支或 tag，默认 `main` |
| `--subdir <SUBDIR>` | GitHub 仓库内的子目录 |

添加本地 skill：

```bash
byi skill add ./my-skill
byi skill add ~/skills/python-helper
```

从 GitHub 添加：

```bash
byi skill add --github owner/repo
byi skill add --github owner/repo --ref main
byi skill add --github owner/repo --ref main --subdir skills/review
```

注意：`--github` 和本地 `[PATH]` 不能同时指定。

### skill list

列出当前已管理的 skill 实例。

```text
byi skill list [OPTIONS]
```

| 选项 | 说明 |
|---|---|
| `--json` | 以 JSON 格式输出 |
| `--long` | 输出更详细的字段 |
| `--enabled` | 只显示启用的实例 |
| `--disabled` | 只显示停用的实例 |

```bash
byi skill list
byi skill list --long
byi skill list --enabled
byi skill list --json
```

### skill view

查看 skill 详情。`<REFERENCE>` 可以是：

- `instance_id`
- `skill_id`
- `dir_name`
- `name`（若命中多个会提示歧义）

```text
byi skill view [OPTIONS] <REFERENCE>
```

| 选项 | 说明 |
|---|---|
| `--json` | 以 JSON 格式输出 |
| `--long` | 输出完整字段 |

```bash
byi skill view python-review
byi skill view inst_123
byi skill view python-review --json
```

### skill edit

编辑 skill 元数据文件 `byi-skill.toml`。

```text
byi skill edit <REFERENCE>
```

```bash
byi skill edit python-review
```

如果设置了 `EDITOR` 环境变量，会自动启动编辑器；否则只输出目标文件路径。

### skill remove

按实例 ID 删除本地 skill 实例。

```text
byi skill remove <INSTANCE_ID>
```

```bash
byi skill remove inst_123
```

### skill enable / disable

启用或停用某个实例。

```text
byi skill enable  <INSTANCE_ID>
byi skill disable <INSTANCE_ID>
```

```bash
byi skill enable inst_123
byi skill disable inst_123
```

### skill instances

查看实例级别的完整信息。

```text
byi skill instances [OPTIONS]
```

| 选项 | 说明 |
|---|---|
| `--json` | 以 JSON 格式输出 |
| `--long` | 输出完整字段 |

```bash
byi skill instances
byi skill instances --json
```

### skill doctor

检查当前 skill 管理状态是否健康。

```text
byi skill doctor [OPTIONS]
```

| 选项 | 说明 |
|---|---|
| `--json` | 以 JSON 格式输出 |
| `--long` | 无额外效果（保留兼容） |

```bash
byi skill doctor
byi skill doctor --json
```

可能检测到的问题包括：

- `missing-skill-md`：skill 目录缺少 `SKILL.md`
- `missing-directory`：注册表存在但目录已删除
- `copied-instance`：检测到复制目录，已重建实例 ID
- `duplicate-install-path`：目录被重复引用

### skill rescan

显式触发全量扫描，修正注册表与实际目录之间的差异。

```text
byi skill rescan [OPTIONS]
```

| 选项 | 说明 |
|---|---|
| `--json` | 以 JSON 格式输出扫描摘要 |
| `--long` | 无额外效果（保留兼容） |

```bash
byi skill rescan
byi skill rescan --json
```

---

## tui

打开 TUI 界面。

```bash
byi tui
```

---

## 数据目录

默认目录结构：

```text
~/.config/byi/
  └── config.toml          # 同步等配置

~/.byi/
  ├── skills/              # skill 实体目录
  │   ├── python-review/
  │   ├── python-review~2/
  │   └── review/
  └── registry/
      ├── skills.toml      # skill 逻辑定义注册表
      └── installed.toml   # 本地安装实例注册表
```

同名 skill 通过目录后缀区分，例如 `python-review~2`。

---

## 快速示例

初始化并测试同步：

```bash
byi sync config
byi sync init --provider github --repo owner/repo --branch main --base-path .byi
byi sync test
byi sync push
```

添加并管理 skill：

```bash
byi skill add ./my-skill
byi skill list --long
byi skill view my-skill
byi skill disable inst_xxx
byi skill doctor
byi skill rescan
```

从 GitHub 导入 skill：

```bash
byi skill add --github wbytts/byi-skill-review --ref main --subdir skills/review
```

打开 TUI：

```bash
byi tui
```
