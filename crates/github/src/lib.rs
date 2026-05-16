use std::process::Command;

#[derive(Clone, Debug)]
pub struct GitHubRemote {
    pub repo: String,
    pub branch: String,
    pub base_path: String,
}

pub struct GitHubCli;

impl GitHubCli {
    pub fn is_installed() -> bool {
        Command::new("gh")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn install_help() -> &'static str {
        "未检测到 GitHub CLI: gh\n\n安装方式:\n  macOS: brew install gh\n  Windows: winget install --id GitHub.cli\n  Linux: 参考 https://github.com/cli/cli/blob/trunk/docs/install_linux.md\n\n安装后运行:\n  gh auth login --web -h github.com --scopes repo"
    }

    pub fn ensure_installed() -> Result<(), String> {
        if Self::is_installed() {
            Ok(())
        } else {
            Err(Self::install_help().to_string())
        }
    }

    pub fn ensure_authenticated() -> Result<(), String> {
        Self::ensure_installed()?;

        if run_command("gh", &["auth", "status", "-h", "github.com"]).is_err() {
            run_command(
                "gh",
                &[
                    "auth",
                    "login",
                    "--web",
                    "-h",
                    "github.com",
                    "--scopes",
                    "repo",
                ],
            )?;
        }

        Ok(())
    }

    pub fn ensure_repo_access(repo: &str) -> Result<(), String> {
        Self::ensure_authenticated()?;
        run_command("gh", &["repo", "view", repo, "--json", "nameWithOwner"])?;

        Ok(())
    }

    pub fn get_file(remote: &GitHubRemote, remote_path: &str) -> Result<String, String> {
        let endpoint = format!(
            "repos/{}/contents/{}?ref={}",
            remote.repo, remote_path, remote.branch
        );
        let encoded = run_command("gh", &["api", &endpoint, "--jq", ".content"])?;
        let compact: String = encoded.chars().filter(|ch| !ch.is_whitespace()).collect();
        let bytes = {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD
                .decode(compact)
                .map_err(|err| format!("解码 GitHub 文件内容失败: {err}"))?
        };

        String::from_utf8(bytes).map_err(|err| format!("GitHub 文件不是有效 UTF-8: {err}"))
    }

    pub fn put_file(
        remote: &GitHubRemote,
        remote_path: &str,
        contents: &str,
    ) -> Result<(), String> {
        let endpoint = format!("repos/{}/contents/{}", remote.repo, remote_path);
        let ref_endpoint = format!("{endpoint}?ref={}", remote.branch);
        let existing_sha = run_command("gh", &["api", &ref_endpoint, "--jq", ".sha"]).ok();
        let encoded = {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD.encode(contents)
        };
        let message = format!("Update byi data at {remote_path}");
        let mut args = vec![
            "api".to_string(),
            "--method".to_string(),
            "PUT".to_string(),
            endpoint,
            "-f".to_string(),
            field_arg("message", &message),
            "-f".to_string(),
            field_arg("content", &encoded),
            "-f".to_string(),
            field_arg("branch", &remote.branch),
        ];

        if let Some(sha) = existing_sha.as_deref() {
            args.push("-f".to_string());
            args.push(field_arg("sha", sha.trim()));
        }

        run_command_owned("gh", &args)?;
        Ok(())
    }
}

fn field_arg(key: &str, value: &str) -> String {
    format!("{key}={value}")
}

fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| format!("执行 `{program}` 失败: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        return Err(format!("`{program} {}` 失败: {detail}", args.join(" ")));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_command_owned(program: &str, args: &[String]) -> Result<String, String> {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();

    run_command(program, &refs)
}
