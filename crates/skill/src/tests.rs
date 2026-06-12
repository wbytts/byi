use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;

use super::*;

fn test_manager() -> (tempfile::TempDir, SkillManager) {
    let data_dir = tempfile::tempdir().expect("应该能创建数据目录");
    let manager = SkillManager::new(data_dir.path().to_path_buf());
    (data_dir, manager)
}

fn create_sample_skill(dir: &Path, name: &str, description: &str) {
    fs::create_dir_all(dir).expect("应该能创建 skill 目录");
    fs::write(dir.join("SKILL.md"), format!("# {name}\n\n{description}\n"))
        .expect("应该能写入 SKILL.md");
}

fn add_local_skill(manager: &SkillManager, path: &Path) -> String {
    manager
        .run_command(Some(SkillCommand::Add(SkillAddCommand {
            path: Some(path.display().to_string()),
            github: None,
            r#ref: None,
            subdir: None,
        })))
        .expect("添加本地 skill 应该成功")
}

// ========================================================================
// add
// ========================================================================

#[test]
fn add_local_skill_creates_instance_and_registry() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "python-review", "Python review skill");

    let output = add_local_skill(&manager, source_dir.path());

    assert!(output.contains("已添加 skill"));
    let list = manager
        .run_command(Some(SkillCommand::List(SkillListCommand {
            format: SkillFormatCommand::default(),
            enabled: false,
            disabled: false,
        })))
        .expect("skill list 应该成功");
    assert!(list.contains("python-review"));
    assert!(manager.skills_dir().join("python-review").exists());
    assert!(manager.registry_dir().join("skills.toml").exists());
    assert!(manager.registry_dir().join("installed.toml").exists());
}

#[test]
fn add_rejects_both_path_and_github() {
    let (_data_dir, manager) = test_manager();
    let err = manager
        .run_command(Some(SkillCommand::Add(SkillAddCommand {
            path: Some("/tmp/fake".to_string()),
            github: Some("owner/repo".to_string()),
            r#ref: None,
            subdir: None,
        })))
        .expect_err("同时指定 path 和 github 应该失败");
    assert!(err.contains("不能同时指定"));
}

#[test]
fn add_rejects_neither_path_nor_github() {
    let (_data_dir, manager) = test_manager();
    let err = manager
        .run_command(Some(SkillCommand::Add(SkillAddCommand {
            path: None,
            github: None,
            r#ref: None,
            subdir: None,
        })))
        .expect_err("不指定 path 和 github 应该失败");
    assert!(err.contains("需要本地路径或 --github"));
}

#[test]
fn add_rejects_nonexistent_path() {
    let (_data_dir, manager) = test_manager();
    let err = manager
        .run_command(Some(SkillCommand::Add(SkillAddCommand {
            path: Some("/nonexistent/path/to/skill".to_string()),
            github: None,
            r#ref: None,
            subdir: None,
        })))
        .expect_err("不存在的路径应该失败");
    assert!(err.contains("不存在"));
}

#[test]
fn add_resolves_name_collision_with_suffix() {
    let (_data_dir, manager) = test_manager();
    let first = tempfile::tempdir().expect("应该能创建源目录");
    let second = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(first.path(), "review", "First review");
    create_sample_skill(second.path(), "review", "Second review");

    add_local_skill(&manager, first.path());
    add_local_skill(&manager, second.path());

    assert!(manager.skills_dir().join("review").exists());
    assert!(manager.skills_dir().join("review~2").exists());
}

#[test]
fn add_preserves_existing_metadata() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "custom", "Custom skill");
    fs::write(
        source_dir.path().join("byi-skill.toml"),
        r#"
instance_id = "inst_custom_123"
skill_id = "skill:custom/my-skill"
name = "my-skill"
description = "From metadata"
source = "local:~/my-skill"
created_at = "2024-01-01T00:00:00Z"
updated_at = "2024-01-01T00:00:00Z"
"#,
    )
    .expect("应该能写入元数据");

    add_local_skill(&manager, source_dir.path());

    let scan = manager
        .scan_and_reconcile_skills()
        .expect("扫描应该成功");
    let skill = scan
        .skills
        .iter()
        .find(|s| s.name == "my-skill")
        .expect("应该保留元数据中的名称");
    assert_eq!(skill.description, "From metadata");
}

// ========================================================================
// remove
// ========================================================================

#[test]
fn remove_deletes_instance_and_updates_registry() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "to-delete", "Will be removed");
    add_local_skill(&manager, source_dir.path());

    let scan = manager
        .scan_and_reconcile_skills()
        .expect("扫描应该成功");
    let instance_id = scan.installed_skills[0].instance_id.clone();

    let output = manager
        .run_command(Some(SkillCommand::Remove(SkillInstanceCommand {
            instance_id: instance_id.clone(),
        })))
        .expect("remove 应该成功");

    assert!(output.contains("已删除实例"));
    assert!(!manager.skills_dir().join("to-delete").exists());

    let rescan = manager
        .scan_and_reconcile_skills()
        .expect("重新扫描应该成功");
    assert!(rescan.installed_skills.is_empty());
}

#[test]
fn remove_rejects_unknown_instance_id() {
    let (_data_dir, manager) = test_manager();
    let err = manager
        .run_command(Some(SkillCommand::Remove(SkillInstanceCommand {
            instance_id: "inst_nonexistent".to_string(),
        })))
        .expect_err("删除不存在的实例应该失败");
    assert!(err.contains("未找到实例"));
}

// ========================================================================
// view
// ========================================================================

#[test]
fn view_by_instance_id() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "inspect", "Inspect skill");
    add_local_skill(&manager, source_dir.path());

    let scan = manager
        .scan_and_reconcile_skills()
        .expect("扫描应该成功");
    let instance_id = scan.installed_skills[0].instance_id.clone();

    let output = manager
        .run_command(Some(SkillCommand::View(SkillViewCommand {
            reference: instance_id,
            format: SkillFormatCommand::default(),
        })))
        .expect("view 应该成功");
    assert!(output.contains("inspect"));
}

#[test]
fn view_by_dir_name() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "inspect", "Inspect skill");
    add_local_skill(&manager, source_dir.path());

    let output = manager
        .run_command(Some(SkillCommand::View(SkillViewCommand {
            reference: "inspect".to_string(),
            format: SkillFormatCommand::default(),
        })))
        .expect("view 应该成功");
    assert!(output.contains("inspect"));
}

#[test]
fn view_by_skill_id_single_instance() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "single", "Single instance");
    add_local_skill(&manager, source_dir.path());

    let scan = manager
        .scan_and_reconcile_skills()
        .expect("扫描应该成功");
    let skill_id = scan.skills[0].id.clone();

    let output = manager
        .run_command(Some(SkillCommand::View(SkillViewCommand {
            reference: skill_id,
            format: SkillFormatCommand::default(),
        })))
        .expect("view 应该成功");
    assert!(output.contains("single"));
}

#[test]
fn view_rejects_unknown_reference() {
    let (_data_dir, manager) = test_manager();
    let err = manager
        .run_command(Some(SkillCommand::View(SkillViewCommand {
            reference: "no-such-skill".to_string(),
            format: SkillFormatCommand::default(),
        })))
        .expect_err("view 不存在的 skill 应该失败");
    assert!(err.contains("未找到 skill"));
}

#[test]
fn view_by_name_rejects_ambiguous_matches() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "review", "Review helper");
    manager
        .run_command(Some(SkillCommand::Add(SkillAddCommand {
            path: Some(source_dir.path().display().to_string()),
            github: None,
            r#ref: None,
            subdir: None,
        })))
        .expect("第一次 add 应该成功");
    let first = manager.skills_dir().join("review");
    let renamed = manager.skills_dir().join("review-a");
    fs::rename(&first, &renamed).expect("应该能重命名目录");
    copy_dir_recursive(&renamed, &manager.skills_dir().join("review-b")).expect("应该能复制目录");
    manager
        .run_command(Some(SkillCommand::Rescan(SkillFormatCommand::default())))
        .expect("rescan 应该成功");

    let error = manager
        .run_command(Some(SkillCommand::View(SkillViewCommand {
            reference: "review".to_string(),
            format: SkillFormatCommand::default(),
        })))
        .expect_err("同名 skill 应该提示歧义");

    assert!(error.contains("命中多个实例"));
}

// ========================================================================
// list / instances / format
// ========================================================================

#[test]
fn list_outputs_json() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "json-skill", "For JSON test");
    add_local_skill(&manager, source_dir.path());

    let output = manager
        .run_command(Some(SkillCommand::List(SkillListCommand {
            format: SkillFormatCommand {
                json: true,
                long: false,
            },
            enabled: false,
            disabled: false,
        })))
        .expect("list json 应该成功");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("应该是有效 JSON");
    let arr = parsed.as_array().expect("应该是数组");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "json-skill");
}

#[test]
fn instances_outputs_json_with_instance_fields() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "inst-test", "Instance test");
    add_local_skill(&manager, source_dir.path());

    let output = manager
        .run_command(Some(SkillCommand::Instances(SkillInstancesCommand {
            format: SkillFormatCommand {
                json: true,
                long: false,
            },
        })))
        .expect("instances json 应该成功");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("应该是有效 JSON");
    let arr = parsed.as_array().expect("应该是数组");
    assert_eq!(arr.len(), 1);
    assert!(arr[0].get("instance_id").is_some());
    assert!(arr[0].get("dir_name").is_some());
}

#[test]
fn list_filters_disabled_only() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "filter-test", "Filter test");
    add_local_skill(&manager, source_dir.path());

    let scan = manager
        .scan_and_reconcile_skills()
        .expect("扫描应该成功");
    let instance_id = scan.installed_skills[0].instance_id.clone();

    manager
        .run_command(Some(SkillCommand::Disable(SkillInstanceCommand {
            instance_id: instance_id.clone(),
        })))
        .expect("disable 应该成功");

    let enabled_list = manager
        .run_command(Some(SkillCommand::List(SkillListCommand {
            format: SkillFormatCommand::default(),
            enabled: true,
            disabled: false,
        })))
        .expect("list enabled 应该成功");
    assert!(!enabled_list.contains("filter-test"));

    let disabled_list = manager
        .run_command(Some(SkillCommand::List(SkillListCommand {
            format: SkillFormatCommand::default(),
            enabled: false,
            disabled: true,
        })))
        .expect("list disabled 应该成功");
    assert!(disabled_list.contains("filter-test"));
}

// ========================================================================
// doctor
// ========================================================================

#[test]
fn doctor_reports_no_issues_when_empty() {
    let (_data_dir, manager) = test_manager();
    let output = manager
        .run_command(Some(SkillCommand::Doctor(SkillFormatCommand::default())))
        .expect("doctor 应该成功");
    assert!(output.contains("正常"));
}

#[test]
fn doctor_reports_missing_skill_md() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    fs::create_dir_all(source_dir.path()).expect("应该能创建目录");
    fs::write(
        source_dir.path().join("byi-skill.toml"),
        r#"
instance_id = "inst_no_md"
skill_id = "skill:local/no-md"
name = "no-md"
description = "Missing SKILL.md"
source = "local"
created_at = "2024-01-01T00:00:00Z"
updated_at = "2024-01-01T00:00:00Z"
"#,
    )
    .expect("应该能写入元数据");
    add_local_skill(&manager, source_dir.path());

    let output = manager
        .run_command(Some(SkillCommand::Doctor(SkillFormatCommand::default())))
        .expect("doctor 应该成功");
    assert!(output.contains("missing-skill-md"));
}

#[test]
fn doctor_reports_missing_directory() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "ghost", "Ghost skill");
    add_local_skill(&manager, source_dir.path());

    // 删除目录但保留注册表
    let scan = manager
        .scan_and_reconcile_skills()
        .expect("扫描应该成功");
    let install_path = &scan.installed_skills[0].install_path;
    fs::remove_dir_all(install_path).expect("应该能删除目录");

    let output = manager
        .run_command(Some(SkillCommand::Doctor(SkillFormatCommand::default())))
        .expect("doctor 应该成功");
    assert!(output.contains("missing-directory"));
}

#[test]
fn doctor_reports_copied_directory_duplicate_id() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "dup", "Duplicate test");
    add_local_skill(&manager, source_dir.path());

    let original = manager.skills_dir().join("dup");
    let copied = manager.skills_dir().join("dup-copy");
    copy_dir_recursive(&original, &copied).expect("应该能复制目录");

    let output = manager
        .run_command(Some(SkillCommand::Doctor(SkillFormatCommand::default())))
        .expect("doctor 应该成功");
    assert!(output.contains("copied-instance"));
}

#[test]
fn doctor_outputs_json() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "doc-json", "Doctor JSON");
    add_local_skill(&manager, source_dir.path());
    fs::remove_dir_all(manager.skills_dir().join("doc-json")).expect("应该能删除目录");

    let output = manager
        .run_command(Some(SkillCommand::Doctor(SkillFormatCommand {
            json: true,
            long: false,
        })))
        .expect("doctor json 应该成功");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("应该是有效 JSON");
    let arr = parsed.as_array().expect("应该是数组");
    assert!(!arr.is_empty());
    assert!(arr[0].get("code").is_some());
    assert!(arr[0].get("level").is_some());
}

// ========================================================================
// rescan
// ========================================================================

#[test]
fn rescan_repairs_copied_directory_and_rename() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "review", "Review helper");
    manager
        .run_command(Some(SkillCommand::Add(SkillAddCommand {
            path: Some(source_dir.path().display().to_string()),
            github: None,
            r#ref: None,
            subdir: None,
        })))
        .expect("第一次 add 应该成功");

    let original = manager.skills_dir().join("review");
    let renamed = manager.skills_dir().join("review-renamed");
    fs::rename(&original, &renamed).expect("应该能重命名目录");
    let copied = manager.skills_dir().join("review-copy");
    copy_dir_recursive(&renamed, &copied).expect("应该能复制目录");

    let output = manager
        .run_command(Some(SkillCommand::Rescan(SkillFormatCommand::default())))
        .expect("rescan 应该成功");

    assert!(output.contains("rescan 完成"));
    let instances = manager
        .run_command(Some(SkillCommand::Instances(SkillInstancesCommand {
            format: SkillFormatCommand::default(),
        })))
        .expect("instances 应该成功");
    assert!(instances.contains("review-renamed"));
    assert!(instances.contains("review-copy"));
}

#[test]
fn rescan_outputs_json() {
    let (_data_dir, manager) = test_manager();
    let output = manager
        .run_command(Some(SkillCommand::Rescan(SkillFormatCommand {
            json: true,
            long: false,
        })))
        .expect("rescan json 应该成功");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("应该是有效 JSON");
    assert!(parsed.get("added").is_some());
    assert!(parsed.get("updated").is_some());
    assert!(parsed.get("removed").is_some());
    assert!(parsed.get("issues").is_some());
}

// ========================================================================
// enable / disable
// ========================================================================

#[test]
fn disable_and_enable_toggle_registry_state() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "helper", "helper skill");
    manager
        .run_command(Some(SkillCommand::Add(SkillAddCommand {
            path: Some(source_dir.path().display().to_string()),
            github: None,
            r#ref: None,
            subdir: None,
        })))
        .expect("add 应该成功");

    let scan = manager.scan_and_reconcile_skills().expect("扫描应该成功");
    let instance_id = scan.installed_skills[0].instance_id.clone();
    manager
        .run_command(Some(SkillCommand::Disable(SkillInstanceCommand {
            instance_id: instance_id.clone(),
        })))
        .expect("disable 应该成功");
    let list = manager
        .run_command(Some(SkillCommand::List(SkillListCommand {
            format: SkillFormatCommand::default(),
            enabled: false,
            disabled: true,
        })))
        .expect("list disabled 应该成功");
    assert!(list.contains(&instance_id));

    manager
        .run_command(Some(SkillCommand::Enable(SkillInstanceCommand {
            instance_id,
        })))
        .expect("enable 应该成功");
    let list = manager
        .run_command(Some(SkillCommand::List(SkillListCommand {
            format: SkillFormatCommand::default(),
            enabled: true,
            disabled: false,
        })))
        .expect("list enabled 应该成功");
    assert!(list.contains("yes"));
}

// ========================================================================
// list triggers rescan
// ========================================================================

#[test]
fn list_triggers_rescan_for_manually_copied_directory() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "review", "Review helper");
    manager
        .run_command(Some(SkillCommand::Add(SkillAddCommand {
            path: Some(source_dir.path().display().to_string()),
            github: None,
            r#ref: None,
            subdir: None,
        })))
        .expect("第一次 add 应该成功");

    let copied = manager.skills_dir().join("review-copy");
    copy_dir_recursive(&manager.skills_dir().join("review"), &copied).expect("应该能复制目录");

    let list = manager
        .run_command(Some(SkillCommand::List(SkillListCommand {
            format: SkillFormatCommand::default(),
            enabled: false,
            disabled: false,
        })))
        .expect("list 应该成功");

    assert!(list.contains("review-copy"));
}

// ========================================================================
// edit
// ========================================================================

#[test]
fn edit_returns_path_when_no_editor_set() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "editable", "Editable skill");
    add_local_skill(&manager, source_dir.path());
    // 确保没有 EDITOR 环境变量
    unsafe { std::env::remove_var("EDITOR"); }
    let output = manager
        .run_command(Some(SkillCommand::Edit(SkillEditCommand {
            reference: "editable".to_string(),
        })))
        .expect("edit 应该成功");
    assert!(output.contains("请编辑文件"));
    assert!(output.contains("byi-skill.toml"));
}

// ========================================================================
// sync
// ========================================================================

struct MockStorage {
    files: RefCell<BTreeMap<String, Vec<u8>>>,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            files: RefCell::new(BTreeMap::new()),
        }
    }

    fn list_files(&self) -> Vec<String> {
        self.files.borrow().keys().cloned().collect()
    }
}

impl byi_storage::RemoteStorage for MockStorage {
    fn read_file(&self, file_name: &str) -> Result<Vec<u8>, String> {
        self.files
            .borrow()
            .get(file_name)
            .cloned()
            .ok_or_else(|| format!("文件不存在: {file_name}"))
    }

    fn write_file(&self, file_name: &str, contents: &[u8]) -> Result<(), String> {
        self.files
            .borrow_mut()
            .insert(file_name.to_string(), contents.to_vec());
        Ok(())
    }

    fn delete_file(&self, file_name: &str) -> Result<(), String> {
        self.files.borrow_mut().remove(file_name);
        Ok(())
    }

    fn test(&self) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn sync_push_creates_manifest_and_uploads_files() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "syncable", "Sync test");
    add_local_skill(&manager, source_dir.path());

    let storage = MockStorage::new();
    manager
        .sync_push_to_storage(&storage)
        .expect("sync push 应该成功");

    let files = storage.list_files();
    assert!(
        files.iter().any(|f| f.starts_with("skills/")),
        "应该上传 skill 文件"
    );
    assert!(
        files.iter().any(|f| f.starts_with("registry/")),
        "应该上传 registry 文件"
    );
    assert!(
        files.iter().any(|f| f == ".byi-sync-manifest.toml"),
        "应该创建 manifest"
    );
}

#[test]
fn sync_pull_restores_skills_and_registry() {
    let (data_dir_a, manager_a) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "restore", "Restore test");
    add_local_skill(&manager_a, source_dir.path());

    // 先 push 到 mock storage
    let storage = MockStorage::new();
    manager_a
        .sync_push_to_storage(&storage)
        .expect("第一次 push 应该成功");

    // 创建新的空 manager，从 storage pull
    let manager_b = SkillManager::new(data_dir_a.path().join("target"));
    manager_b
        .sync_pull_from_storage(&storage)
        .expect("sync pull 应该成功");

    let list = manager_b
        .run_command(Some(SkillCommand::List(SkillListCommand {
            format: SkillFormatCommand::default(),
            enabled: false,
            disabled: false,
        })))
        .expect("list 应该成功");
    assert!(list.contains("restore"));
}

#[test]
fn sync_push_deletes_orphaned_remote_files() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "orphan-test", "Orphan test");
    add_local_skill(&manager, source_dir.path());

    let storage = MockStorage::new();
    manager
        .sync_push_to_storage(&storage)
        .expect("第一次 push 应该成功");

    // 删除本地 skill
    let scan = manager
        .scan_and_reconcile_skills()
        .expect("扫描应该成功");
    let instance_id = scan.installed_skills[0].instance_id.clone();
    manager
        .run_command(Some(SkillCommand::Remove(SkillInstanceCommand { instance_id })))
        .expect("remove 应该成功");

    // 再次 push，远端孤立文件应被删除
    manager
        .sync_push_to_storage(&storage)
        .expect("第二次 push 应该成功");

    let files = storage.list_files();
    assert!(
        !files.iter().any(|f| f.contains("orphan-test")),
        "孤立 skill 文件应该被删除"
    );
}

// ========================================================================
// utility functions
// ========================================================================

#[test]
fn resolve_skill_reference_by_dir_name() {
    let skill = Skill {
        id: "skill:local/test".to_string(),
        name: "test".to_string(),
        description: "desc".to_string(),
        source: "local".to_string(),
        domains: vec![],
        modules: vec![],
    };
    let installed = InstalledSkill {
        instance_id: "inst_abc".to_string(),
        skill_id: "skill:local/test".to_string(),
        dir_name: "test-dir".to_string(),
        install_path: "/tmp/test".to_string(),
        enabled: true,
        source: "local".to_string(),
        created_at: "2024-01-01".to_string(),
        updated_at: "2024-01-01".to_string(),
    };

    let entry = resolve_skill_reference(&[skill.clone()], &[installed.clone()], "test-dir")
        .expect("dir_name 引用应该成功");
    assert_eq!(entry.installed.dir_name, "test-dir");
}

#[test]
fn resolve_skill_reference_by_skill_id_ambiguous() {
    let skill = Skill {
        id: "skill:local/test".to_string(),
        name: "test".to_string(),
        description: "desc".to_string(),
        source: "local".to_string(),
        domains: vec![],
        modules: vec![],
    };
    let installed_a = InstalledSkill {
        instance_id: "inst_a".to_string(),
        skill_id: "skill:local/test".to_string(),
        dir_name: "test-a".to_string(),
        install_path: "/tmp/test-a".to_string(),
        enabled: true,
        source: "local".to_string(),
        created_at: "2024-01-01".to_string(),
        updated_at: "2024-01-01".to_string(),
    };
    let installed_b = InstalledSkill {
        instance_id: "inst_b".to_string(),
        skill_id: "skill:local/test".to_string(),
        dir_name: "test-b".to_string(),
        install_path: "/tmp/test-b".to_string(),
        enabled: true,
        source: "local".to_string(),
        created_at: "2024-01-01".to_string(),
        updated_at: "2024-01-01".to_string(),
    };

    let err = resolve_skill_reference(&[skill], &[installed_a, installed_b], "skill:local/test")
        .expect_err("多个实例的 skill_id 引用应该歧义");
    assert!(err.contains("对应 2 个实例"));
}

#[test]
fn normalize_display_name_strips_suffix() {
    assert_eq!(normalize_display_name("python-review"), "python-review");
    assert_eq!(normalize_display_name("python-review~2"), "python-review");
    assert_eq!(normalize_display_name("review~10"), "review");
}

#[test]
fn sanitize_name_replaces_invalid_chars() {
    assert_eq!(sanitize_name("hello world"), "hello-world");
    assert_eq!(sanitize_name("a/b\\c"), "a-b-c");
    assert_eq!(sanitize_name("foo:bar"), "foo-bar");
}

#[test]
fn expand_tilde_resolves_home() {
    let home = dirs::home_dir().expect("应该有 home 目录");
    assert_eq!(expand_tilde("~/test").unwrap(), home.join("test"));
    assert_eq!(expand_tilde("/absolute/path").unwrap(), PathBuf::from("/absolute/path"));
}

// ========================================================================
// registry / metadata resilience
// ========================================================================

#[test]
fn scan_repairs_corrupt_metadata() {
    let (_data_dir, manager) = test_manager();
    let source_dir = tempfile::tempdir().expect("应该能创建源目录");
    create_sample_skill(source_dir.path(), "corrupt", "Corrupt skill");
    add_local_skill(&manager, source_dir.path());

    // 把元数据搞坏
    let meta_path = manager.skills_dir().join("corrupt").join("byi-skill.toml");
    fs::write(&meta_path, "this is not valid toml!!!").expect("应该能写入损坏内容");

    // scan 应该能处理，重新生成元数据
    let scan = manager
        .scan_and_reconcile_skills()
        .expect("扫描应该成功");
    let skill = scan
        .skills
        .iter()
        .find(|s| s.name == "corrupt")
        .expect("应该能修复 corrupt skill");
    assert_eq!(skill.description, "Corrupt skill");
}

#[test]
fn scan_handles_empty_skills_dir() {
    let (_data_dir, manager) = test_manager();
    manager.ensure_skill_layout().expect("应该能创建目录结构");

    let scan = manager
        .scan_and_reconcile_skills()
        .expect("空目录扫描应该成功");
    assert!(scan.skills.is_empty());
    assert!(scan.installed_skills.is_empty());
    assert!(scan.issues.is_empty());
    assert_eq!(scan.added, 0);
    assert_eq!(scan.updated, 0);
    assert_eq!(scan.removed, 0);
}

#[test]
fn validate_repo_accepts_valid_format() {
    assert!(validate_repo("owner/repo").is_ok());
}

#[test]
fn validate_repo_rejects_invalid_format() {
    assert!(validate_repo("invalid").is_err());
    assert!(validate_repo("too/many/parts").is_err());
    assert!(validate_repo("").is_err());
}
