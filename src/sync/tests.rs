use byi_storage::{GitHubRemoteConfig, RemoteConfig};
use byi_webdav::WebDavPreset;

use super::config::ByiConfig;
use super::*;

fn run(args: impl IntoIterator<Item = String>) -> Result<String, String> {
    App::default().run(args)
}

#[test]
fn default_command_outputs_hello_message() {
    let output = run(Vec::<String>::new()).expect("默认命令应该成功");

    assert_eq!(output, "byi is installed and ready.");
}

#[test]
fn version_command_outputs_package_version() {
    let output = run(["--version".to_string()]).expect("版本命令应该成功");

    assert_eq!(output, "byi 0.0.1");
}

#[test]
fn unknown_command_returns_error() {
    let error = run(["missing".to_string()]).expect_err("未知命令应该失败");

    assert!(error.contains("unrecognized subcommand 'missing'"));
}

#[test]
fn top_level_help_lists_sync_operations() {
    let output = run(["--help".to_string()]).expect("帮助命令应该成功");

    assert!(!output.contains("byi remote"));
    assert!(output.contains("byi sync config"));
    assert!(output.contains("byi sync init"));
    assert!(output.contains("byi sync init --provider webdav --preset jianguoyun"));
    assert!(output.contains("byi sync init --provider webdav --preset custom"));
    assert!(output.contains("byi sync status"));
    assert!(output.contains("byi sync test"));
    assert!(output.contains("byi sync pull"));
    assert!(output.contains("byi sync push"));
}

#[test]
fn sync_without_subcommand_shows_sync_help() {
    let output = run(["sync".to_string()]).expect("sync 帮助应该成功");
    let help = run(["sync".to_string(), "--help".to_string()]).expect("sync --help 应该成功");

    assert_eq!(output, help);
    assert!(output.contains("Usage: byi sync [COMMAND]"));
    assert!(output.contains("byi sync config"));
    assert!(output.contains("byi sync init --provider github --repo owner/repo"));
    assert!(output.contains("byi sync init --provider webdav --preset jianguoyun"));
    assert!(output.contains("byi sync init --provider webdav --preset custom"));
    assert!(output.contains("byi sync status"));
    assert!(output.contains("byi sync test"));
    assert!(output.contains("byi sync pull"));
    assert!(output.contains("byi sync push"));
}

#[test]
fn sync_status_without_config_says_remote_is_not_initialized() {
    let temp_dir = tempfile::tempdir().expect("应该能创建临时目录");
    let app = App {
        config_dir: temp_dir.path().to_path_buf(),
        check_github: false,
    };

    let output = app
        .run(["sync".to_string(), "status".to_string()])
        .expect("状态命令应该成功");

    assert_eq!(output, "Sync remote is not initialized.");
}

#[test]
fn sync_init_with_arguments_writes_local_remote_config() {
    let temp_dir = tempfile::tempdir().expect("应该能创建临时目录");
    let app = App {
        config_dir: temp_dir.path().to_path_buf(),
        check_github: false,
    };

    let output = app
        .run([
            "sync".to_string(),
            "init".to_string(),
            "--provider".to_string(),
            "github".to_string(),
            "--repo".to_string(),
            "owner/repo".to_string(),
            "--branch".to_string(),
            "dev".to_string(),
            "--base-path".to_string(),
            ".agent-config".to_string(),
        ])
        .expect("remote init 应该成功");

    assert!(output.contains("Sync remote initialized."));
    assert!(output.contains("GitHub remote: owner/repo"));
    let config = app.load_config().expect("配置应该可读取");
    let remote = config.remote.expect("remote 配置应该存在");
    let RemoteConfig::GitHub(remote) = remote else {
        panic!("应该写入 GitHub remote 配置");
    };
    assert_eq!(remote.repo, "owner/repo");
    assert_eq!(remote.branch, "dev");
    assert_eq!(remote.base_path, ".agent-config");
    assert_eq!(remote.auth, "github-cli");
}

#[test]
fn sync_init_with_jianguoyun_webdav_writes_webdav_config() {
    let temp_dir = tempfile::tempdir().expect("应该能创建临时目录");
    let app = App {
        config_dir: temp_dir.path().to_path_buf(),
        check_github: false,
    };

    let output = app
        .run([
            "sync".to_string(),
            "init".to_string(),
            "--provider".to_string(),
            "webdav".to_string(),
            "--preset".to_string(),
            "jianguoyun".to_string(),
            "--username".to_string(),
            "name@example.com".to_string(),
            "--base-path".to_string(),
            ".byi".to_string(),
        ])
        .expect("webdav init 应该成功");

    assert!(output.contains("Sync remote initialized."));
    assert!(output.contains("WebDAV remote: https://dav.jianguoyun.com/dav/"));
    assert!(output.contains("Preset: jianguoyun"));

    let config = app.load_config().expect("配置应该可读取");
    let remote = config.remote.expect("remote 配置应该存在");
    let RemoteConfig::WebDav(remote) = remote else {
        panic!("应该写入 WebDAV remote 配置");
    };
    assert_eq!(remote.preset, WebDavPreset::Jianguoyun);
    assert_eq!(remote.endpoint_url, "https://dav.jianguoyun.com/dav/");
    assert_eq!(remote.username.as_deref(), Some("name@example.com"));
    assert_eq!(remote.base_path, ".byi");
}

#[test]
fn sync_init_with_custom_webdav_requires_url_and_writes_config() {
    let temp_dir = tempfile::tempdir().expect("应该能创建临时目录");
    let app = App {
        config_dir: temp_dir.path().to_path_buf(),
        check_github: false,
    };

    let missing_url = app
        .run([
            "sync".to_string(),
            "init".to_string(),
            "--provider".to_string(),
            "webdav".to_string(),
            "--preset".to_string(),
            "custom".to_string(),
        ])
        .expect_err("自定义 WebDAV 缺少 URL 应该失败");
    assert!(missing_url.contains("自定义 WebDAV 需要 --url"));

    let output = app
        .run([
            "sync".to_string(),
            "init".to_string(),
            "--provider".to_string(),
            "webdav".to_string(),
            "--preset".to_string(),
            "custom".to_string(),
            "--url".to_string(),
            "https://example.com/dav/".to_string(),
            "--username".to_string(),
            "name".to_string(),
            "--base-path".to_string(),
            ".custom".to_string(),
        ])
        .expect("自定义 WebDAV init 应该成功");

    assert!(output.contains("WebDAV remote: https://example.com/dav/"));
    assert!(output.contains("Preset: custom"));

    let config = app.load_config().expect("配置应该可读取");
    let remote = config.remote.expect("remote 配置应该存在");
    let RemoteConfig::WebDav(remote) = remote else {
        panic!("应该写入 WebDAV remote 配置");
    };
    assert_eq!(remote.preset, WebDavPreset::Custom);
    assert_eq!(remote.endpoint_url, "https://example.com/dav/");
    assert_eq!(remote.username.as_deref(), Some("name"));
    assert_eq!(remote.base_path, ".custom");
}

#[test]
fn sync_config_without_remote_shows_initialize_menu() {
    let temp_dir = tempfile::tempdir().expect("应该能创建临时目录");
    let app = App {
        config_dir: temp_dir.path().to_path_buf(),
        check_github: false,
    };

    let output = app
        .run(["sync".to_string(), "config".to_string()])
        .expect("sync config 应该成功");

    assert!(output.contains("Sync remote is not initialized."));
    assert!(output.contains("a. 初始化同步配置"));
    assert!(output.contains("b. 退出"));
    assert!(output.contains("github仓库"));
    assert!(output.contains("webdav"));
}

#[test]
fn sync_config_with_remote_shows_manage_menu() {
    let temp_dir = tempfile::tempdir().expect("应该能创建临时目录");
    let app = App {
        config_dir: temp_dir.path().to_path_buf(),
        check_github: false,
    };
    app.save_config(&ByiConfig {
        remote: Some(RemoteConfig::GitHub(GitHubRemoteConfig {
            repo: "owner/repo".to_string(),
            branch: "main".to_string(),
            base_path: ".byi".to_string(),
            auth: "github-cli".to_string(),
        })),
    })
    .expect("配置应该能保存");

    let output = app
        .run(["sync".to_string(), "config".to_string()])
        .expect("sync config 应该成功");

    assert!(output.contains("GitHub remote: owner/repo"));
    assert!(output.contains("a. 更改同步配置"));
    assert!(output.contains("b. 测试同步配置"));
    assert!(output.contains("c. 退出"));
}
