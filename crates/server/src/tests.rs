use std::path::PathBuf;

use tokio::net::TcpListener;

use super::*;

#[tokio::test]
async fn router_serves_health_api() {
    let app = router(PathBuf::from("missing-dist"));
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("应该能监听随机端口");
    let addr = listener.local_addr().expect("应该能获取监听地址");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server should run");
    });

    let response = reqwest::get(format!("http://{addr}/api/health"))
        .await
        .expect("健康检查请求应该成功")
        .text()
        .await
        .expect("响应应该能读取");

    assert!(response.contains("\"status\":\"ok\""));
    assert!(response.contains("\"service\":\"byi-server\""));
}

#[tokio::test]
async fn sync_status_reports_uninitialized_remote() {
    let temp_dir = tempfile::tempdir().expect("应该能创建临时目录");
    let app = router_with_config_dir(PathBuf::from("missing-dist"), temp_dir.path().join("byi"));
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("应该能监听随机端口");
    let addr = listener.local_addr().expect("应该能获取监听地址");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server should run");
    });

    let response = reqwest::get(format!("http://{addr}/api/sync/status"))
        .await
        .expect("sync status 请求应该成功")
        .text()
        .await
        .expect("响应应该能读取");

    assert!(response.contains("\"configured\":false"));
    assert!(response.contains("Sync remote is not initialized."));
}

#[tokio::test]
async fn sync_github_config_can_be_saved_and_read_back() {
    let temp_dir = tempfile::tempdir().expect("应该能创建临时目录");
    let app = router_with_config_dir(PathBuf::from("missing-dist"), temp_dir.path().join("byi"));
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("应该能监听随机端口");
    let addr = listener.local_addr().expect("应该能获取监听地址");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server should run");
    });

    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{addr}/api/sync/config/github"))
        .json(&serde_json::json!({
            "repo": "owner/repo",
            "branch": "dev",
            "base_path": ".agent-config"
        }))
        .send()
        .await
        .expect("sync config 请求应该成功")
        .text()
        .await
        .expect("响应应该能读取");

    assert!(response.contains("\"configured\":true"));
    assert!(response.contains("\"provider\":\"github\""));
    assert!(response.contains("\"repo\":\"owner/repo\""));
    assert!(response.contains("\"branch\":\"dev\""));
    assert!(response.contains("\"base_path\":\".agent-config\""));

    let status = reqwest::get(format!("http://{addr}/api/sync/status"))
        .await
        .expect("sync status 请求应该成功")
        .text()
        .await
        .expect("响应应该能读取");

    assert!(status.contains("\"configured\":true"));
    assert!(status.contains("\"repo\":\"owner/repo\""));
}
