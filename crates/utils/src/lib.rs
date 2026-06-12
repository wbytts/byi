use std::fs;
use std::path::{Path, PathBuf};

pub fn normalize_line_endings(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn collect_files_recursive(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    if !root.exists() {
        return Ok(files);
    }

    collect_files_recursive_inner(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_recursive_inner(
    root: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in
        fs::read_dir(current).map_err(|err| format!("读取目录失败 {}: {err}", current.display()))?
    {
        let entry = entry.map_err(|err| format!("读取目录项失败 {}: {err}", current.display()))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|err| format!("计算相对路径失败 {}: {err}", path.display()))?
            .to_path_buf();

        if entry
            .file_type()
            .map_err(|err| format!("读取文件类型失败 {}: {err}", path.display()))?
            .is_dir()
        {
            collect_files_recursive_inner(root, &path, files)?;
        } else {
            files.push(relative);
        }
    }

    Ok(())
}

pub fn remove_dir_if_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|err| format!("删除目录失败 {}: {err}", path.display()))
    } else {
        Ok(())
    }
}

pub fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("创建目录失败 {}: {err}", parent.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_line_endings_converts_windows_and_old_mac_endings() {
        let input = "a\r\nb\rc\n";

        assert_eq!(normalize_line_endings(input), "a\nb\nc\n");
    }

    #[test]
    fn collect_files_recursive_returns_relative_paths() {
        let temp_dir = tempfile::tempdir().expect("应该能创建临时目录");
        fs::create_dir_all(temp_dir.path().join("nested")).expect("应该能创建子目录");
        fs::write(temp_dir.path().join("a.txt"), "a").expect("应该能写入文件");
        fs::write(temp_dir.path().join("nested/b.txt"), "b").expect("应该能写入文件");

        let files = collect_files_recursive(temp_dir.path()).expect("应该能遍历文件");

        assert_eq!(
            files,
            vec![PathBuf::from("a.txt"), PathBuf::from("nested/b.txt")]
        );
    }
}
