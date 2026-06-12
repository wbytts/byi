use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use byi_utils::{collect_files_recursive, ensure_parent_dir, remove_dir_if_exists};

const SKILL_METADATA_FILE: &str = "byi-skill.toml";
const REMOTE_MANIFEST_FILE: &str = ".byi-sync-manifest.toml";

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domains: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct SkillPack {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domains: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct InstalledSkill {
    pub instance_id: String,
    pub skill_id: String,
    pub dir_name: String,
    pub install_path: String,
    pub enabled: bool,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Default)]
struct SkillRegistryFile {
    #[serde(default)]
    skills: Vec<Skill>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Default)]
struct InstalledRegistryFile {
    #[serde(default, rename = "installed_skills")]
    installed_skills: Vec<InstalledSkill>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct SkillMetadata {
    instance_id: String,
    skill_id: String,
    name: String,
    description: String,
    source: String,
    created_at: String,
    updated_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    domains: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    modules: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Default)]
struct RemoteSyncManifest {
    #[serde(default)]
    files: Vec<String>,
}

pub struct SkillEntry {
    pub skill: Skill,
    pub installed: InstalledSkill,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct SkillDoctorIssue {
    pub code: String,
    pub level: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct ScanResult {
    pub skills: Vec<Skill>,
    pub installed_skills: Vec<InstalledSkill>,
    pub issues: Vec<SkillDoctorIssue>,
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
}

#[derive(Clone, Debug)]
pub struct SkillManager {
    data_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub enum SkillCommand {
    Add(SkillAddCommand),
    List(SkillListCommand),
    View(SkillViewCommand),
    Edit(SkillEditCommand),
    Remove(SkillInstanceCommand),
    Enable(SkillInstanceCommand),
    Disable(SkillInstanceCommand),
    Instances(SkillInstancesCommand),
    Doctor(SkillFormatCommand),
    Rescan(SkillFormatCommand),
}

#[derive(Clone, Debug)]
pub struct SkillAddCommand {
    pub path: Option<String>,
    pub github: Option<String>,
    pub r#ref: Option<String>,
    pub subdir: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SkillListCommand {
    pub format: SkillFormatCommand,
    pub enabled: bool,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
pub struct SkillViewCommand {
    pub reference: String,
    pub format: SkillFormatCommand,
}

#[derive(Clone, Debug)]
pub struct SkillEditCommand {
    pub reference: String,
}

#[derive(Clone, Debug)]
pub struct SkillInstanceCommand {
    pub instance_id: String,
}

#[derive(Clone, Debug)]
pub struct SkillInstancesCommand {
    pub format: SkillFormatCommand,
}

#[derive(Clone, Debug, Default)]
pub struct SkillFormatCommand {
    pub json: bool,
    pub long: bool,
}

impl SkillManager {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn run_command(&self, command: Option<SkillCommand>) -> Result<String, String> {
        match command {
            None => Err("缺少 skill 子命令。运行 `byi skill --help` 查看用法。".to_string()),
            Some(SkillCommand::Add(command)) => self.skill_add(command),
            Some(SkillCommand::List(command)) => self.skill_list(command),
            Some(SkillCommand::View(command)) => self.skill_view(command),
            Some(SkillCommand::Edit(command)) => self.skill_edit(command),
            Some(SkillCommand::Remove(command)) => self.skill_remove(command),
            Some(SkillCommand::Enable(command)) => self.skill_set_enabled(command, true),
            Some(SkillCommand::Disable(command)) => self.skill_set_enabled(command, false),
            Some(SkillCommand::Instances(command)) => self.skill_instances(command),
            Some(SkillCommand::Doctor(format)) => self.skill_doctor(format),
            Some(SkillCommand::Rescan(format)) => self.skill_rescan(format),
        }
    }

    pub fn sync_pull_from_storage(
        &self,
        storage: &dyn byi_storage::RemoteStorage,
    ) -> Result<(), String> {
        let manifest = self.read_remote_manifest(storage)?;
        self.reset_skill_data()?;

        for relative in &manifest.files {
            let bytes = storage.read_file(relative)?;
            self.write_data_file(relative, &bytes)?;
        }

        self.scan_and_reconcile_skills().map(|_| ())
    }

    pub fn sync_push_to_storage(
        &self,
        storage: &dyn byi_storage::RemoteStorage,
    ) -> Result<(), String> {
        self.scan_and_reconcile_skills()?;
        let local_files = self.collect_sync_files()?;
        let existing_manifest = self.read_remote_manifest(storage).unwrap_or_default();
        let local_set: BTreeSet<_> = local_files.iter().cloned().collect();

        for relative in &local_files {
            let bytes = fs::read(self.data_dir.join(relative)).map_err(|err| {
                format!(
                    "读取本地文件失败 {}: {err}",
                    self.data_dir.join(relative).display()
                )
            })?;
            storage.write_file(relative, &bytes)?;
        }

        for relative in existing_manifest.files {
            if !local_set.contains(&relative) {
                storage.delete_file(&relative)?;
            }
        }

        let manifest = RemoteSyncManifest { files: local_files };
        let contents = toml::to_string_pretty(&manifest)
            .map_err(|err| format!("序列化远端 manifest 失败: {err}"))?;
        storage.write_file(REMOTE_MANIFEST_FILE, contents.as_bytes())
    }

    fn skill_add(&self, command: SkillAddCommand) -> Result<String, String> {
        self.ensure_skill_layout()?;

        match (&command.path, &command.github) {
            (Some(_), Some(_)) => {
                return Err("`byi skill add` 不能同时指定本地路径和 --github。".to_string());
            }
            (None, None) => return Err("`byi skill add` 需要本地路径或 --github。".to_string()),
            _ => {}
        }

        let source_kind;
        let skill_namespace;
        let inferred_name_hint;
        let temp_root = std::env::temp_dir().join(format!("byi-skill-add-{}", new_instance_id()));
        let source_dir = temp_root.join("source");
        remove_dir_if_exists(&temp_root)?;
        fs::create_dir_all(&source_dir)
            .map_err(|err| format!("创建临时目录失败 {}: {err}", source_dir.display()))?;

        if let Some(path) = command.path {
            let source_path = expand_tilde(&path)?;
            if !source_path.is_dir() {
                return Err(format!(
                    "skill 路径不存在或不是目录: {}",
                    source_path.display()
                ));
            }
            copy_dir_recursive(&source_path, &source_dir)?;
            source_kind = format!("local:{}", source_path.display());
            skill_namespace = "local".to_string();
            inferred_name_hint = read_skill_name(&source_dir)
                .or_else(|| infer_skill_name(&source_path))
                .unwrap_or_else(|| "skill".to_string());
        } else {
            let repo = command.github.expect("github source should exist");
            validate_repo(&repo)?;
            let git_ref = command.r#ref.unwrap_or_else(|| "main".to_string());
            let subdir = command.subdir.unwrap_or_default();
            self.download_github_skill(&repo, &git_ref, &subdir, &source_dir)?;
            let repo_namespace = sanitize_name(&repo.replace('/', "-"));
            source_kind = if subdir.is_empty() {
                format!("github:{repo}")
            } else {
                format!("github:{repo}?ref={git_ref}&subdir={subdir}")
            };
            skill_namespace = format!("github-{repo_namespace}");
            inferred_name_hint = read_skill_name(&source_dir)
                .unwrap_or_else(|| infer_github_skill_name(&repo, &subdir));
        }

        let mut metadata = self
            .read_skill_metadata_from_dir(&source_dir)?
            .unwrap_or_else(|| SkillMetadata {
                instance_id: new_instance_id(),
                skill_id: new_skill_id(&skill_namespace, &inferred_name_hint),
                name: inferred_name_hint.clone(),
                description: read_skill_description(&source_dir).unwrap_or_default(),
                source: source_kind.clone(),
                created_at: current_timestamp(),
                updated_at: current_timestamp(),
                domains: Vec::new(),
                modules: Vec::new(),
            });
        metadata.instance_id = new_instance_id();
        metadata.source = source_kind.clone();
        metadata.updated_at = current_timestamp();

        if metadata.skill_id.trim().is_empty()
            || source_namespace(&metadata.skill_id) != skill_namespace.as_str()
        {
            metadata.skill_id = new_skill_id(&skill_namespace, &metadata.name);
        }
        if metadata.description.trim().is_empty() {
            metadata.description = read_skill_description(&source_dir).unwrap_or_default();
        }

        let target_dir_name = self.next_available_dir_name(&metadata.name)?;
        let target_dir = self.skills_dir().join(&target_dir_name);
        copy_dir_recursive(&source_dir, &target_dir)?;
        self.write_skill_metadata(&target_dir, &metadata)?;
        let scan = self.scan_and_reconcile_skills()?;
        remove_dir_if_exists(&temp_root)?;

        Ok(format!(
            "已添加 skill。\n名称: {}\n目录: {}\n实例: {}\n来源: {}\n扫描结果: 新增 {}, 更新 {}, 移除 {}",
            metadata.name,
            target_dir_name,
            metadata.instance_id,
            source_kind,
            scan.added,
            scan.updated,
            scan.removed
        ))
    }

    fn skill_list(&self, command: SkillListCommand) -> Result<String, String> {
        let scan = self.scan_and_reconcile_skills()?;
        let mut entries = join_skill_entries(&scan.skills, &scan.installed_skills);
        if command.enabled {
            entries.retain(|entry| entry.installed.enabled);
        }
        if command.disabled {
            entries.retain(|entry| !entry.installed.enabled);
        }
        format_skill_entries(&entries, &command.format, false)
    }

    fn skill_view(&self, command: SkillViewCommand) -> Result<String, String> {
        let scan = self.scan_and_reconcile_skills()?;
        let entry =
            resolve_skill_reference(&scan.skills, &scan.installed_skills, &command.reference)?;
        if command.format.json {
            serde_json::to_string_pretty(&entry_json(&entry, true))
                .map_err(|err| format!("序列化 skill 详情失败: {err}"))
        } else {
            Ok(format!(
                "skill_id: {}\ninstance_id: {}\nname: {}\ndir_name: {}\ndescription: {}\nsource: {}\npath: {}\nenabled: {}",
                entry.skill.id,
                entry.installed.instance_id,
                entry.skill.name,
                entry.installed.dir_name,
                entry.skill.description,
                entry.installed.source,
                entry.installed.install_path,
                yes_no(entry.installed.enabled),
            ))
        }
    }

    fn skill_edit(&self, command: SkillEditCommand) -> Result<String, String> {
        let path = self.edit_target(&command.reference)?;

        if let Ok(editor) = std::env::var("EDITOR") {
            let status = Command::new(&editor)
                .arg(&path)
                .status()
                .map_err(|err| format!("启动编辑器失败 `{editor}`: {err}"))?;
            if !status.success() {
                return Err(format!("编辑器退出异常: {status}"));
            }
            self.scan_and_reconcile_skills()?;
            Ok(format!("已更新 skill 元数据: {}", path.display()))
        } else {
            Ok(format!("请编辑文件: {}", path.display()))
        }
    }

    pub fn edit_target(&self, reference: &str) -> Result<PathBuf, String> {
        let scan = self.scan_and_reconcile_skills()?;
        let entry = resolve_skill_reference(&scan.skills, &scan.installed_skills, reference)?;
        Ok(PathBuf::from(&entry.installed.install_path).join(SKILL_METADATA_FILE))
    }

    fn skill_remove(&self, command: SkillInstanceCommand) -> Result<String, String> {
        let scan = self.scan_and_reconcile_skills()?;
        let installed = scan
            .installed_skills
            .iter()
            .find(|item| item.instance_id == command.instance_id)
            .ok_or_else(|| format!("未找到实例: {}", command.instance_id))?;
        remove_dir_if_exists(Path::new(&installed.install_path))?;
        let scan = self.scan_and_reconcile_skills()?;

        Ok(format!(
            "已删除实例 {}。\n扫描结果: 新增 {}, 更新 {}, 移除 {}",
            command.instance_id, scan.added, scan.updated, scan.removed
        ))
    }

    fn skill_set_enabled(
        &self,
        command: SkillInstanceCommand,
        enabled: bool,
    ) -> Result<String, String> {
        let scan = self.scan_and_reconcile_skills()?;
        let _installed = scan
            .installed_skills
            .iter()
            .find(|item| item.instance_id == command.instance_id)
            .ok_or_else(|| format!("未找到实例: {}", command.instance_id))?;
        let mut registry = self.load_installed_registry()?;
        let item = registry
            .installed_skills
            .iter_mut()
            .find(|item| item.instance_id == command.instance_id)
            .ok_or_else(|| format!("注册表缺少实例: {}", command.instance_id))?;
        item.enabled = enabled;
        item.updated_at = current_timestamp();
        self.save_installed_registry(&registry)?;
        Ok(format!(
            "实例 {} 已{}。",
            command.instance_id,
            if enabled { "启用" } else { "停用" }
        ))
    }

    fn skill_instances(&self, command: SkillInstancesCommand) -> Result<String, String> {
        let scan = self.scan_and_reconcile_skills()?;
        let entries = join_skill_entries(&scan.skills, &scan.installed_skills);
        format_skill_entries(&entries, &command.format, true)
    }

    fn skill_doctor(&self, format: SkillFormatCommand) -> Result<String, String> {
        let scan = self.scan_and_reconcile_skills()?;
        if format.json {
            return serde_json::to_string_pretty(&scan.issues)
                .map_err(|err| format!("序列化 doctor 结果失败: {err}"));
        }

        if scan.issues.is_empty() {
            Ok("skill 状态正常。".to_string())
        } else {
            Ok(scan
                .issues
                .iter()
                .map(|issue| format!("[{}] {}: {}", issue.level, issue.code, issue.message))
                .collect::<Vec<_>>()
                .join("\n"))
        }
    }

    fn skill_rescan(&self, format: SkillFormatCommand) -> Result<String, String> {
        let scan = self.scan_and_reconcile_skills()?;
        if format.json {
            return serde_json::to_string_pretty(&serde_json::json!({
                "added": scan.added,
                "updated": scan.updated,
                "removed": scan.removed,
                "issues": scan.issues,
            }))
            .map_err(|err| format!("序列化 rescan 结果失败: {err}"));
        }

        Ok(format!(
            "rescan 完成。\n新增: {}\n更新: {}\n移除: {}\n问题: {}",
            scan.added,
            scan.updated,
            scan.removed,
            scan.issues.len()
        ))
    }

    pub fn scan_and_reconcile_skills(&self) -> Result<ScanResult, String> {
        self.ensure_skill_layout()?;
        let previous_installed = self.load_installed_registry()?.installed_skills;

        let mut issues = Vec::new();
        let mut skills = BTreeMap::<String, Skill>::new();
        let mut installed_map = BTreeMap::<String, InstalledSkill>::new();
        let mut seen_install_paths = BTreeSet::new();
        let directories = self.skill_directories()?;
        let duplicate_instance_ids = detect_duplicate_instance_ids(self, &directories)?;

        for dir in directories {
            let dir_name = dir
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or_else(|| format!("无效 skill 目录名: {}", dir.display()))?
                .to_string();
            let metadata = self.reconcile_skill_metadata(
                &dir,
                &dir_name,
                duplicate_instance_ids.contains(&dir),
                &mut issues,
            )?;

            let install_path = dir.display().to_string();
            if !seen_install_paths.insert(install_path.clone()) {
                issues.push(SkillDoctorIssue {
                    code: "duplicate-install-path".to_string(),
                    level: "warn".to_string(),
                    message: format!("目录被重复引用: {}", install_path),
                });
            }

            let previous = previous_installed
                .iter()
                .find(|item| item.instance_id == metadata.instance_id)
                .cloned();
            let skill = Skill {
                id: metadata.skill_id.clone(),
                name: metadata.name.clone(),
                description: metadata.description.clone(),
                source: metadata.source.clone(),
                domains: metadata.domains.clone(),
                modules: metadata.modules.clone(),
            };
            let installed = InstalledSkill {
                instance_id: metadata.instance_id.clone(),
                skill_id: metadata.skill_id.clone(),
                dir_name,
                install_path,
                enabled: previous.as_ref().map(|item| item.enabled).unwrap_or(true),
                source: metadata.source.clone(),
                created_at: previous
                    .as_ref()
                    .map(|item| item.created_at.clone())
                    .unwrap_or_else(|| metadata.created_at.clone()),
                updated_at: current_timestamp(),
            };

            skills.insert(skill.id.clone(), skill);
            installed_map.insert(installed.instance_id.clone(), installed);
        }

        for installed in &previous_installed {
            let path = Path::new(&installed.install_path);
            if !path.exists() {
                issues.push(SkillDoctorIssue {
                    code: "missing-directory".to_string(),
                    level: "warn".to_string(),
                    message: format!("注册表存在但目录已删除: {}", installed.install_path),
                });
            }
        }

        let skills = skills.into_values().collect::<Vec<_>>();
        let installed_skills = installed_map.into_values().collect::<Vec<_>>();
        let result = ScanResult {
            added: installed_skills
                .iter()
                .filter(|item| {
                    previous_installed
                        .iter()
                        .all(|old| old.instance_id != item.instance_id)
                })
                .count(),
            updated: installed_skills
                .iter()
                .filter(|item| {
                    previous_installed.iter().any(|old| {
                        old.instance_id == item.instance_id
                            && (old.dir_name != item.dir_name
                                || old.install_path != item.install_path
                                || old.skill_id != item.skill_id
                                || old.source != item.source)
                    })
                })
                .count(),
            removed: previous_installed
                .iter()
                .filter(|item| {
                    installed_skills
                        .iter()
                        .all(|new| new.instance_id != item.instance_id)
                })
                .count(),
            issues,
            skills,
            installed_skills,
        };

        self.save_skill_registry(&SkillRegistryFile {
            skills: result.skills.clone(),
        })?;
        self.save_installed_registry(&InstalledRegistryFile {
            installed_skills: result.installed_skills.clone(),
        })?;

        Ok(result)
    }

    fn reconcile_skill_metadata(
        &self,
        dir: &Path,
        dir_name: &str,
        duplicate_instance_id: bool,
        issues: &mut Vec<SkillDoctorIssue>,
    ) -> Result<SkillMetadata, String> {
        let existing = self.read_skill_metadata_from_dir(dir)?;
        let name = existing
            .as_ref()
            .map(|item| item.name.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| normalize_display_name(dir_name));
        let description = existing
            .as_ref()
            .map(|item| item.description.clone())
            .filter(|value| !value.trim().is_empty())
            .or_else(|| read_skill_description(dir))
            .unwrap_or_default();
        let source = existing
            .as_ref()
            .map(|item| item.source.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "local".to_string());
        let mut metadata = SkillMetadata {
            instance_id: existing
                .as_ref()
                .map(|item| item.instance_id.clone())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(new_instance_id),
            skill_id: existing
                .as_ref()
                .map(|item| item.skill_id.clone())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| new_skill_id(source_namespace(&source), &name)),
            name,
            description,
            source,
            created_at: existing
                .as_ref()
                .map(|item| item.created_at.clone())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(current_timestamp),
            updated_at: current_timestamp(),
            domains: existing
                .as_ref()
                .map(|item| item.domains.clone())
                .unwrap_or_default(),
            modules: existing
                .as_ref()
                .map(|item| item.modules.clone())
                .unwrap_or_default(),
        };

        if duplicate_instance_id {
            metadata.instance_id = new_instance_id();
            issues.push(SkillDoctorIssue {
                code: "copied-instance".to_string(),
                level: "warn".to_string(),
                message: format!("检测到复制目录，已重建实例 ID: {}", dir.display()),
            });
        }

        if !dir.join("SKILL.md").exists() {
            issues.push(SkillDoctorIssue {
                code: "missing-skill-md".to_string(),
                level: "warn".to_string(),
                message: format!("缺少 SKILL.md: {}", dir.display()),
            });
        }

        self.write_skill_metadata(dir, &metadata)?;
        Ok(metadata)
    }

    fn skill_directories(&self) -> Result<Vec<PathBuf>, String> {
        let mut dirs = Vec::new();
        for entry in fs::read_dir(self.skills_dir())
            .map_err(|err| format!("读取 skill 目录失败 {}: {err}", self.skills_dir().display()))?
        {
            let entry = entry.map_err(|err| format!("读取 skill 目录项失败: {err}"))?;
            if entry
                .file_type()
                .map_err(|err| format!("读取文件类型失败 {}: {err}", entry.path().display()))?
                .is_dir()
            {
                dirs.push(entry.path());
            }
        }
        dirs.sort();
        Ok(dirs)
    }

    fn save_skill_registry(&self, file: &SkillRegistryFile) -> Result<(), String> {
        write_toml_file(self.registry_dir().join("skills.toml"), file)
    }

    fn load_installed_registry(&self) -> Result<InstalledRegistryFile, String> {
        read_toml_file(self.registry_dir().join("installed.toml"))
    }

    fn save_installed_registry(&self, file: &InstalledRegistryFile) -> Result<(), String> {
        write_toml_file(self.registry_dir().join("installed.toml"), file)
    }

    fn read_skill_metadata_from_dir(&self, dir: &Path) -> Result<Option<SkillMetadata>, String> {
        let path = dir.join(SKILL_METADATA_FILE);
        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&path)
            .map_err(|err| format!("读取 skill 元数据失败 {}: {err}", path.display()))?;
        Ok(toml::from_str(&contents).ok())
    }

    fn write_skill_metadata(&self, dir: &Path, metadata: &SkillMetadata) -> Result<(), String> {
        let path = dir.join(SKILL_METADATA_FILE);
        write_toml_file(path, metadata)
    }

    fn ensure_skill_layout(&self) -> Result<(), String> {
        fs::create_dir_all(self.skills_dir())
            .map_err(|err| format!("创建 skill 目录失败 {}: {err}", self.skills_dir().display()))?;
        fs::create_dir_all(self.registry_dir()).map_err(|err| {
            format!(
                "创建 registry 目录失败 {}: {err}",
                self.registry_dir().display()
            )
        })?;
        Ok(())
    }

    fn skills_dir(&self) -> PathBuf {
        self.data_dir.join("skills")
    }

    fn registry_dir(&self) -> PathBuf {
        self.data_dir.join("registry")
    }

    fn next_available_dir_name(&self, name: &str) -> Result<String, String> {
        let base = sanitize_name(name);
        let candidate = if base.is_empty() {
            "skill".to_string()
        } else {
            base
        };
        if !self.skills_dir().join(&candidate).exists() {
            return Ok(candidate);
        }

        for index in 2..1000 {
            let next = format!("{}~{}", candidate, index);
            if !self.skills_dir().join(&next).exists() {
                return Ok(next);
            }
        }

        Err(format!("无法为 skill 分配目录名: {name}"))
    }

    fn collect_sync_files(&self) -> Result<Vec<String>, String> {
        self.ensure_skill_layout()?;
        let mut files = Vec::new();
        for root_name in ["skills", "registry"] {
            let root = self.data_dir.join(root_name);
            for relative in collect_files_recursive(&root)? {
                files.push(format!("{root_name}/{}", relative.display()));
            }
        }
        files.sort();
        Ok(files)
    }

    fn read_remote_manifest(
        &self,
        storage: &dyn byi_storage::RemoteStorage,
    ) -> Result<RemoteSyncManifest, String> {
        let bytes = storage.read_file(REMOTE_MANIFEST_FILE)?;
        let contents = String::from_utf8(bytes)
            .map_err(|err| format!("远端 manifest 不是有效 UTF-8: {err}"))?;
        toml::from_str(&contents).map_err(|err| format!("解析远端 manifest 失败: {err}"))
    }

    fn reset_skill_data(&self) -> Result<(), String> {
        remove_dir_if_exists(&self.skills_dir())?;
        remove_dir_if_exists(&self.registry_dir())?;
        self.ensure_skill_layout()
    }

    fn write_data_file(&self, relative: &str, contents: &[u8]) -> Result<(), String> {
        let path = self.data_dir.join(relative);
        ensure_parent_dir(&path)?;
        fs::write(&path, contents)
            .map_err(|err| format!("写入本地数据失败 {}: {err}", path.display()))
    }

    fn download_github_skill(
        &self,
        repo: &str,
        git_ref: &str,
        subdir: &str,
        output_dir: &Path,
    ) -> Result<(), String> {
        let remote = byi_github::GitHubRemote {
            repo: repo.to_string(),
            branch: git_ref.to_string(),
            base_path: String::new(),
        };
        fs::create_dir_all(output_dir)
            .map_err(|err| format!("创建 GitHub 下载目录失败 {}: {err}", output_dir.display()))?;
        self.download_github_skill_recursive(&remote, subdir.trim_matches('/'), output_dir)
    }

    fn download_github_skill_recursive(
        &self,
        remote: &byi_github::GitHubRemote,
        remote_dir: &str,
        output_dir: &Path,
    ) -> Result<(), String> {
        let entries = byi_github::GitHubCli::list_directory(remote, remote_dir)?;
        if entries.is_empty() {
            return Err(format!("GitHub 目录不存在或为空: {}", remote_dir));
        }

        for entry in entries {
            if entry.kind == "dir" {
                let next_output = output_dir.join(
                    Path::new(&entry.path)
                        .file_name()
                        .unwrap_or_else(|| OsStr::new("dir")),
                );
                fs::create_dir_all(&next_output)
                    .map_err(|err| format!("创建目录失败 {}: {err}", next_output.display()))?;
                self.download_github_skill_recursive(remote, &entry.path, &next_output)?;
            } else if entry.kind == "file" {
                let bytes = byi_github::GitHubCli::get_file_bytes(remote, &entry.path)?;
                let file_name = Path::new(&entry.path)
                    .file_name()
                    .ok_or_else(|| format!("无效 GitHub 文件路径: {}", entry.path))?;
                let path = output_dir.join(file_name);
                ensure_parent_dir(&path)?;
                fs::write(&path, bytes).map_err(|err| {
                    format!("写入 GitHub skill 文件失败 {}: {err}", path.display())
                })?;
            }
        }

        Ok(())
    }
}

pub fn resolve_skill_reference<'a>(
    skills: &'a [Skill],
    installed_skills: &'a [InstalledSkill],
    reference: &str,
) -> Result<SkillEntry, String> {
    if let Some(installed) = installed_skills
        .iter()
        .find(|item| item.instance_id == reference)
    {
        return build_skill_entry(skills, installed.clone());
    }
    if installed_skills
        .iter()
        .any(|item| item.dir_name == reference)
    {
        let installed = installed_skills
            .iter()
            .find(|item| item.dir_name == reference)
            .expect("dir_name checked above");
        return build_skill_entry(skills, installed.clone());
    }
    if skills.iter().any(|item| item.id == reference) {
        let matches = installed_skills
            .iter()
            .filter(|item| item.skill_id == reference)
            .cloned()
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(format!(
                "skill_id `{reference}` 对应 {} 个实例，请改用 instance_id 或 dir_name。",
                matches.len()
            ));
        }
        return build_skill_entry(skills, matches[0].clone());
    }
    let matches = installed_skills
        .iter()
        .filter(|item| {
            skills
                .iter()
                .find(|skill| skill.id == item.skill_id)
                .map(|skill| skill.name == reference)
                .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();
    match matches.len() {
        0 => Err(format!("未找到 skill: {reference}")),
        1 => build_skill_entry(skills, matches[0].clone()),
        _ => Err(format!(
            "名称 `{reference}` 命中多个实例，请改用 instance_id、dir_name 或 skill_id。"
        )),
    }
}

pub fn build_skill_entry(skills: &[Skill], installed: InstalledSkill) -> Result<SkillEntry, String> {
    let skill_id = installed.skill_id.clone();
    let skill = skills
        .iter()
        .find(|item| item.id == skill_id)
        .cloned()
        .ok_or_else(|| format!("注册表缺少 skill 定义: {skill_id}"))?;
    Ok(SkillEntry { skill, installed })
}


pub fn join_skill_entries(skills: &[Skill], installed_skills: &[InstalledSkill]) -> Vec<SkillEntry> {
    let skill_map = skills
        .iter()
        .map(|skill| (skill.id.clone(), skill.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut entries = installed_skills
        .iter()
        .filter_map(|installed| {
            skill_map
                .get(&installed.skill_id)
                .cloned()
                .map(|skill| SkillEntry {
                    skill,
                    installed: installed.clone(),
                })
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.skill
            .name
            .cmp(&right.skill.name)
            .then(left.installed.dir_name.cmp(&right.installed.dir_name))
    });
    entries
}

fn format_skill_entries(
    entries: &[SkillEntry],
    format: &SkillFormatCommand,
    include_instance_fields: bool,
) -> Result<String, String> {
    if format.json {
        let items = entries
            .iter()
            .map(|entry| entry_json(entry, include_instance_fields || format.long))
            .collect::<Vec<_>>();
        return serde_json::to_string_pretty(&items)
            .map_err(|err| format!("序列化 skill 列表失败: {err}"));
    }

    let headers = if include_instance_fields || format.long {
        vec!["NAME", "DIR_NAME", "INSTANCE_ID", "ENABLED", "SOURCE", "PATH"]
    } else {
        vec!["NAME", "DIR_NAME", "INSTANCE_ID", "ENABLED", "SOURCE", "PATH"]
    };
    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|entry| {
            vec![
                entry.skill.name.clone(),
                entry.installed.dir_name.clone(),
                entry.installed.instance_id.clone(),
                yes_no(entry.installed.enabled).to_string(),
                entry.installed.source.clone(),
                entry.installed.install_path.clone(),
            ]
        })
        .collect();

    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.len());
        }
    }

    let mut lines = Vec::new();
    lines.push(
        headers
            .iter()
            .enumerate()
            .map(|(i, h)| format!("{:<width$}", h, width = widths[i].saturating_add(2)))
            .collect::<String>()
            .trim_end()
            .to_string(),
    );
    for row in &rows {
        lines.push(
            row.iter()
                .enumerate()
                .map(|(i, cell)| format!("{:<width$}", cell, width = widths[i].saturating_add(2)))
                .collect::<String>()
                .trim_end()
                .to_string(),
        );
    }

    Ok(lines.join("\n"))
}

fn entry_json(entry: &SkillEntry, include_instance_fields: bool) -> serde_json::Value {
    if include_instance_fields {
        serde_json::json!({
            "skill_id": entry.skill.id,
            "instance_id": entry.installed.instance_id,
            "name": entry.skill.name,
            "dir_name": entry.installed.dir_name,
            "description": entry.skill.description,
            "source": entry.installed.source,
            "path": entry.installed.install_path,
            "enabled": entry.installed.enabled,
            "created_at": entry.installed.created_at,
            "updated_at": entry.installed.updated_at,
        })
    } else {
        serde_json::json!({
            "name": entry.skill.name,
            "dir_name": entry.installed.dir_name,
            "instance_id": entry.installed.instance_id,
            "enabled": entry.installed.enabled,
            "source": entry.installed.source,
            "path": entry.installed.install_path,
        })
    }
}

fn write_toml_file<T: serde::Serialize>(path: PathBuf, value: &T) -> Result<(), String> {
    ensure_parent_dir(&path)?;
    let contents = toml::to_string_pretty(value)
        .map_err(|err| format!("序列化 TOML 失败 {}: {err}", path.display()))?;
    fs::write(&path, contents).map_err(|err| format!("写入文件失败 {}: {err}", path.display()))
}

fn read_toml_file<T: serde::de::DeserializeOwned + Default>(path: PathBuf) -> Result<T, String> {
    if !path.exists() {
        return Ok(T::default());
    }
    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("读取文件失败 {}: {err}", path.display()))?;
    toml::from_str(&contents).map_err(|err| format!("解析 TOML 失败 {}: {err}", path.display()))
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target)
        .map_err(|err| format!("创建目录失败 {}: {err}", target.display()))?;
    for entry in
        fs::read_dir(source).map_err(|err| format!("读取目录失败 {}: {err}", source.display()))?
    {
        let entry = entry.map_err(|err| format!("读取目录项失败 {}: {err}", source.display()))?;
        let path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry
            .file_type()
            .map_err(|err| format!("读取文件类型失败 {}: {err}", path.display()))?
            .is_dir()
        {
            copy_dir_recursive(&path, &target_path)?;
        } else {
            ensure_parent_dir(&target_path)?;
            fs::copy(&path, &target_path).map_err(|err| {
                format!(
                    "复制文件失败 {} -> {}: {err}",
                    path.display(),
                    target_path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn read_skill_description(dir: &Path) -> Option<String> {
    let path = dir.join("SKILL.md");
    let contents = fs::read_to_string(path).ok()?;
    contents
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToString::to_string)
}

fn read_skill_name(dir: &Path) -> Option<String> {
    let path = dir.join("SKILL.md");
    let contents = fs::read_to_string(path).ok()?;
    contents.lines().find_map(|line| {
        let title = line.trim().trim_start_matches('#').trim();
        if line.trim_start().starts_with('#') && !title.is_empty() {
            Some(title.to_string())
        } else {
            None
        }
    })
}

fn infer_skill_name(dir: &Path) -> Option<String> {
    dir.file_name()
        .and_then(OsStr::to_str)
        .map(normalize_display_name)
}

fn infer_github_skill_name(repo: &str, subdir: &str) -> String {
    if !subdir.trim().is_empty() {
        return Path::new(subdir)
            .file_name()
            .and_then(OsStr::to_str)
            .map(normalize_display_name)
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "skill".to_string());
    }

    repo.rsplit('/')
        .next()
        .map(normalize_display_name)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "skill".to_string())
}

fn normalize_display_name(dir_name: &str) -> String {
    dir_name
        .split('~')
        .next()
        .unwrap_or(dir_name)
        .trim()
        .to_string()
}

fn sanitize_name(name: &str) -> String {
    let mut output = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            output.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() || matches!(ch, '/' | '\\' | ':') {
            output.push('-');
        }
    }
    while output.contains("--") {
        output = output.replace("--", "-");
    }
    output.trim_matches('-').to_string()
}

fn current_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

fn new_skill_id(namespace: &str, name: &str) -> String {
    format!("skill:{}/{}", sanitize_name(namespace), sanitize_name(name))
}

fn new_instance_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("inst_{now:x}{counter:x}")
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn source_namespace(source: &str) -> &str {
    if let Some(namespace) = source
        .strip_prefix("skill:")
        .and_then(|value| value.split('/').next())
    {
        return namespace;
    }
    source.split(':').next().unwrap_or("local")
}

fn detect_duplicate_instance_ids(
    manager: &SkillManager,
    directories: &[PathBuf],
) -> Result<BTreeSet<PathBuf>, String> {
    let mut counts = BTreeMap::<String, usize>::new();
    let mut by_dir = Vec::new();
    for dir in directories {
        let instance_id = manager
            .read_skill_metadata_from_dir(dir)?
            .map(|item| item.instance_id)
            .unwrap_or_default();
        if !instance_id.is_empty() {
            *counts.entry(instance_id.clone()).or_insert(0) += 1;
        }
        by_dir.push((dir.clone(), instance_id));
    }

    Ok(by_dir
        .into_iter()
        .filter(|(_, instance_id)| {
            !instance_id.is_empty() && counts.get(instance_id).copied().unwrap_or(0) > 1
        })
        .map(|(dir, _)| dir)
        .collect())
}

fn expand_tilde(path: &str) -> Result<PathBuf, String> {
    if let Some(stripped) = path.strip_prefix("~/") {
        let home = dirs::home_dir().ok_or_else(|| "无法确定用户 home 目录。".to_string())?;
        Ok(home.join(stripped))
    } else {
        Ok(PathBuf::from(path))
    }
}

fn validate_repo(repo: &str) -> Result<(), String> {
    let parts = repo.split('/').collect::<Vec<_>>();
    if parts.len() == 2 && parts.iter().all(|part| !part.trim().is_empty()) {
        Ok(())
    } else {
        Err("GitHub repo must use owner/repo format.".to_string())
    }
}

#[cfg(test)]
mod tests;
