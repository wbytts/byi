use std::io::{self, Write};

use dialoguer::{Select, theme::ColorfulTheme};

pub(super) fn prompt(label: &str) -> Result<String, String> {
    print!("{label}");
    io::stdout()
        .flush()
        .map_err(|err| format!("刷新终端输出失败: {err}"))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|err| format!("读取终端输入失败: {err}"))?;

    Ok(input.trim().to_string())
}

pub(super) fn prompt_with_default(label: &str, default: &str) -> Result<String, String> {
    let value = prompt(&format!("{label} [{default}]: "))?;

    if value.trim().is_empty() {
        Ok(default.to_string())
    } else {
        Ok(value)
    }
}

pub(super) fn select_option(label: &str, items: &[&str]) -> Result<Option<usize>, String> {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(label)
        .items(items)
        .default(0)
        .interact_opt()
        .map_err(|err| format!("读取终端选择失败: {err}"))
}
