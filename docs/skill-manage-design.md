# skill 管理设计

本文档只讨论 `byi skill` 的本地 skill 管理能力，不讨论领域、模块、技能包等组合层级设计。组合相关内容单独放在 [docs/组合.md](/Users/wby/codes/github/byi/docs/组合.md)。

## 目标

`byi skill` 的目标不是做一个公开 registry 或网络包管理器，而是做一个个人、本地优先、可同步的 skill 管理器。

它优先解决这些问题：

- skill 直接落到当前 `byi` 的配置目录 `~/.byi/skills`
- 用户可以自由修改这些本地 skill
- skill 允许重名
- 系统可以通过注册表维护元数据
- 系统也可以动态扫描 `~/.byi/skills` 修正或补全 skill 信息
- 这些内容可以通过 `byi sync` 同步到云端

它暂时不解决这些问题：

- 组合层级建模
- 技能包编排
- 领域和模块分类体系
- 公共 skill 市场

## 设计原则

### 本地优先

skill 的真实落地位置就是：

```text
~/.byi/skills
```

`byi skill` 的核心不是“安装一个远端包”，而是“把 skill 纳入我的本地管理体系”。

所以主入口应该是：

```bash
byi skill add ...
```

而不是：

```bash
byi skill install ...
```

### 用户可直接修改

`~/.byi/skills` 下的内容不应被视为只读内部缓存，而应视为用户真实拥有的数据目录。

这意味着：

- 用户可以手动修改 skill 文件
- 用户可以手动重命名 skill 目录
- 用户可以直接复制、删除、覆盖某个 skill

`byi` 不应强依赖“所有修改都必须经过命令完成”。

### 注册表和扫描并存

系统可以维护 skill 注册表，但注册表不能成为唯一真相来源。

建议采用“双轨模型”：

- 注册表负责记录已知 skill 的元数据和索引
- 文件系统扫描负责发现实际状态变化

也就是：

- 以 `~/.byi/skills` 的实际内容为最终依据
- 以注册表作为加速索引和附加元数据存储

### 可同步

skill 既然直接放在 `~/.byi` 下，就应天然纳入 `byi sync` 的同步范围。

也就是说：

- `byi sync push` 可以把 skill 及其注册表同步到远端
- `byi sync pull` 可以把远端 skill 及其注册表同步回本地

## 本地目录设计

### 固定根目录

skill 根目录固定为：

```text
~/.byi/skills
```

建议后续统一约定：

```text
~/.byi/
  skills/
  registry/
```

其中：

- `skills/` 保存 skill 实体目录
- `registry/` 保存索引和状态文件

### skill 目录允许重名语义

由于 skill 允许重名，因此目录名不能直接作为唯一标识。

但文件系统下同级目录不能真的同名，所以建议采用“名称 + 后缀”的方式区分。

例如两个都展示为 `python-review` 的 skill，可以落地成：

```text
~/.byi/skills/python-review
~/.byi/skills/python-review~2
```

或者：

```text
~/.byi/skills/python-review
~/.byi/skills/python-review@github
```

或者：

```text
~/.byi/skills/python-review
~/.byi/skills/python-review#01hq4k
```

推荐第一阶段用最简单、最稳定的策略：

- 首个实例用原名
- 冲突时追加递增后缀，例如 `~2`、`~3`

例如：

```text
~/.byi/skills/review
~/.byi/skills/review~2
~/.byi/skills/review~3
```

这样有几个好处：

- 用户肉眼容易理解
- 手动操作成本低
- 不依赖内部 ID 才能找到目录

### 目录名与展示名分离

需要明确：

- 目录名是本地唯一路径名
- 展示名是 skill 的逻辑名称

例如：

- 目录名：`python-review~2`
- 展示名：`python-review`

也就是说：

- 展示名可以重名
- 目录名必须唯一

## 核心模型

本文档只保留两类核心对象：

- `Skill`
- `InstalledSkill`

### Skill

`Skill` 表示一个 skill 的逻辑定义。

最少应包含：

- `id`
- `name`
- `description`
- `source`

要求：

- `id` 唯一
- `name` 可重复
- `source` 记录来源

推荐 ID 形式：

```text
skill:<namespace>/<name>
```

例如：

```text
skill:local/python-review
skill:github/wbytts/python-review
```

### InstalledSkill

`InstalledSkill` 表示一个 skill 在本地 `~/.byi/skills` 中的实际实例。

最少应包含：

- `instance_id`
- `skill_id`
- `dir_name`
- `install_path`
- `enabled`
- `source`

要求：

- `instance_id` 唯一
- `install_path` 唯一
- 同一个 `skill_id` 可以有多个实例

推荐实例 ID 形式：

```text
inst_<random_or_hash>
```

## 为什么不能只靠目录名

因为这些情况都会发生：

- 用户手动复制一个 skill
- 用户手动重命名 skill 目录
- 不同来源的 skill 展示名相同
- 用户删除注册表后，仍然保留 skill 目录

所以必须明确：

- 目录名不能作为逻辑主键
- 注册表不能作为唯一真相
- 文件系统和注册表需要相互校正

## 注册表设计

建议保留 skill 注册表，但把它设计成“索引层”，不是“唯一数据源”。

推荐放在：

```text
~/.byi/registry/skills.toml
~/.byi/registry/installed.toml
```

### skill 定义表

记录逻辑定义：

```toml
[[skills]]
id = "skill:github/wbytts/python-review"
name = "python-review"
description = "Python review skill"
source = "github:wbytts/python-review"
```

### 安装实例表

记录本地实例：

```toml
[[installed_skills]]
instance_id = "inst_01hq4k8w4e"
skill_id = "skill:github/wbytts/python-review"
dir_name = "python-review~2"
install_path = "/Users/wby/.byi/skills/python-review~2"
source = "github:wbytts/python-review"
enabled = true
```

### 注册表职责

注册表主要负责：

- 快速列出已知 skill
- 保存逻辑 ID 与目录路径映射
- 保存启用/停用状态
- 保存来源信息
- 保存无法仅靠目录扫描推断的附加元数据

注册表不应负责：

- 垄断 skill 是否存在的真相
- 阻止用户直接编辑目录

## 动态扫描设计

建议 `byi skill` 支持动态扫描 `~/.byi/skills`，并在需要时刷新注册表。

### 扫描触发时机

建议在这些时机触发扫描：

- `byi skill list`
- `byi skill view`
- `byi skill doctor`
- `byi sync pull` 完成后
- `byi skill add` 完成后

同时可以预留显式命令：

```bash
byi skill rescan
```

### 扫描目标

扫描时应完成这些事情：

1. 枚举 `~/.byi/skills` 下的目录
2. 识别哪些目录是有效 skill
3. 为新增目录补建注册表记录
4. 为被手动删除的目录清理或标记失效记录
5. 修正目录名、路径等变更
6. 尝试从 skill 内容中读取基础元数据

### 扫描优先级

建议优先级如下：

1. 文件系统实际存在
2. skill 内部元数据
3. 注册表中的历史记录

这意味着：

- 如果目录存在而注册表缺失，应补注册表
- 如果注册表存在而目录不存在，应标记失效或清理
- 如果用户修改了 skill 内部元数据，扫描应尽量吸收这些变化

### 用户手改场景

需要明确支持这些场景：

- 用户直接复制一个目录到 `~/.byi/skills`
- 用户直接删除某个 skill 目录
- 用户手动修改 skill 名称或描述
- 用户直接重命名目录

系统行为建议：

- 不报“非法外部修改”
- 而是在下次扫描时尽可能自动修复注册表

## CLI 设计

主命令：

```bash
byi skill <subcommand>
```

建议别名：

- `byi skills` -> `byi skill`
- `byi skill ls` -> `byi skill list`
- `byi skill rm` -> `byi skill remove`

## 推荐最小命令集

第一阶段建议只做下面这组：

```bash
byi skill add <path>
byi skill add --github <owner/repo>
byi skill list
byi skill view <ref>
byi skill edit <ref>
byi skill remove <instance-id>
byi skill enable <instance-id>
byi skill disable <instance-id>
byi skill instances
byi skill doctor
byi skill rescan
```

## 子命令设计

### `add`

`add` 是主入口，用于把 skill 纳入当前 BYI 管理体系，并最终落到：

```text
~/.byi/skills
```

建议支持多种来源，但统一成一个命令。

#### 本地路径

```bash
byi skill add ./my-skill
byi skill add ../shared-skills/reviewer
byi skill add ~/skills/python-helper
```

建议行为：

1. 复制或导入 skill 到 `~/.byi/skills/<dir_name>`
2. 如果目录名冲突，则自动追加后缀
3. 生成或更新注册表
4. 写入来源信息

#### GitHub

```bash
byi skill add --github owner/repo
byi skill add --github owner/repo --ref main
byi skill add --github owner/repo --subdir skills/python-review
```

建议行为：

1. 从 GitHub 拉取指定仓库或子目录
2. 将 skill 内容落地到 `~/.byi/skills`
3. 如果名称冲突，则自动追加后缀
4. 写入来源为 `github:owner/repo`
5. 更新注册表

这里的语义仍然是“纳入我的本地 skill 管理”，而不是“从远端市场安装一个包”。

### `list`

`list` 默认展示当前 `~/.byi/skills` 中已纳入管理的 skill 实例。

建议输出字段：

- `name`
- `dir_name`
- `instance_id`
- `enabled`
- `source`
- `path`

例如：

```text
NAME           DIR_NAME           ENABLED   SOURCE                PATH
python-review  python-review      yes       local                 ~/.byi/skills/python-review
python-review  python-review~2    yes       github:wbytts/repo    ~/.byi/skills/python-review~2
```

建议支持：

```bash
byi skill list --enabled
byi skill list --disabled
byi skill list --json
byi skill list --long
```

执行 `list` 前可以先进行轻量扫描，保证输出尽量反映真实目录状态。

### `view`

`view` 用于查看 skill 定义和实例详情。

建议支持以下引用方式：

- `instance_id`
- `skill_id`
- `dir_name`
- `name`

如果使用 `name` 命中多个结果，应提示用户改用 `instance_id` 或 `dir_name`。

建议展示：

- `skill_id`
- `instance_id`
- `name`
- `dir_name`
- `description`
- `source`
- `path`
- `enabled`

### `edit`

`edit` 用于修改本地 skill 元数据。

第一阶段建议只支持打开 skill 目录中的元数据文件或注册表记录进行编辑。

例如：

```bash
byi skill edit inst_01hq4k8w4e
```

这里要接受一个前提：

- 用户可能不用 `byi skill edit`
- 用户也可能直接去 `~/.byi/skills/...` 里手改

两种方式都应被系统接受。

### `remove`

`remove` 用于移除本地安装实例。

建议行为：

1. 删除 `~/.byi/skills/<dir_name>`
2. 更新注册表
3. 如果需要，保留最小历史记录或直接清理

建议只接受：

```bash
byi skill remove <instance-id>
```

这样可以避免同名 skill 误删。

### `enable` / `disable`

用于控制某个实例是否参与当前可用集合。

建议：

- `enable` / `disable` 主要更新注册表中的状态字段
- 不直接删除 skill 文件

### `instances`

`instances` 用于查看实例级别的完整信息，比 `list` 更偏底层。

推荐字段：

- `instance_id`
- `skill_id`
- `name`
- `dir_name`
- `source`
- `install_path`
- `enabled`
- `created_at`
- `updated_at`

### `doctor`

`doctor` 用于检查当前 skill 管理状态是否健康。

建议检查：

- 注册表中存在但目录不存在
- 目录存在但注册表缺失
- 路径记录不一致
- 同一目录被多个实例重复引用
- skill 基础元数据缺失
- 无法识别来源信息

### `rescan`

`rescan` 用于显式触发一次全量扫描，并刷新注册表。

建议行为：

1. 扫描 `~/.byi/skills`
2. 修正路径映射
3. 补建新目录对应的记录
4. 清理或标记失效记录
5. 输出变更摘要

## 参数建议

建议统一支持这些参数：

- `--json`
- `--long`

对 `add --github` 建议支持：

- `--ref <git-ref>`
- `--subdir <path>`

## 与 `byi sync` 的关系

这是这套设计里的关键约束。

由于 skill 数据直接放在：

```text
~/.byi
```

所以它应该天然属于 `byi sync` 的同步范围。

### 同步范围建议

建议同步这些内容：

```text
~/.byi/skills
~/.byi/registry/skills.toml
~/.byi/registry/installed.toml
```

也就是：

- skill 实体目录同步
- skill 注册表同步

### `sync push`

`byi sync push` 的作用是把本地 skill 状态推到远端。

建议流程：

1. 先执行一次轻量扫描或等价校验
2. 确保注册表与 `~/.byi/skills` 尽量一致
3. 再将整个 `~/.byi` 下需要同步的部分推送到远端

### `sync pull`

`byi sync pull` 的作用是把远端 skill 状态拉回本地。

建议流程：

1. 拉取远端 `~/.byi` 数据
2. 覆盖或合并本地文件
3. 执行一次 `skill rescan`
4. 重新修正注册表和目录映射

### 同步后的修复

由于用户可能在多端手动修改 skill，`sync pull` 后必须允许存在：

- 目录变化
- 注册表漂移
- 同名 skill 新增

所以拉取之后重新扫描是必要步骤。

## 本地目录布局建议

推荐：

```text
~/.byi/
  skills/
    python-review/
    python-review~2/
    review/
  registry/
    skills.toml
    installed.toml
```

说明：

- `skills/` 保存用户实际拥有和可直接修改的 skill
- `registry/` 保存索引信息
- 同名 skill 通过目录后缀区分

## 一句话总结

`byi skill` 先做成一个“个人、本地优先、可扫描、可同步”的 skill 管理器。

核心原则是：

- skill 真实落地在 `~/.byi/skills`
- 允许重名，靠目录后缀区分
- 允许用户自由手改
- 用注册表做索引，但不把注册表当唯一真相
- 通过动态扫描修正状态
- 通过 `byi sync` 把整个 `~/.byi` 的 skill 数据同步到云端
