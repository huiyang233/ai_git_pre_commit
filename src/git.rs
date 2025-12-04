use crate::config::Config;
use anyhow::{Context, Result};
use std::process::Command;

pub fn is_merge_in_progress() -> Result<bool> {
    let output = Command::new("git")
        .args(&["rev-parse", "-q", "--verify", "MERGE_HEAD"])
        .output()
        .context("Failed to check for MERGE_HEAD")?;

    Ok(output.status.success())
}

pub fn get_staged_diff(config: &Config) -> Result<String> {
    // 1. 获取暂存区的所有变更文件（包括新增、修改、删除的文件）
    let output = Command::new("git")
        .args(&["diff", "--cached", "--name-only", "--diff-filter=ACM"])
        .output()
        .context("Failed to execute git diff --cached --name-only")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let files: Vec<&str> = stdout.lines().collect();

    if files.is_empty() {
        return Ok(String::new());
    }

    // 2. 过滤出需要检查的文件（根据配置文件中指定的文件扩展名）
    let filtered_files: Vec<&str> = files
        .into_iter()
        .filter(|file| {
            config
                .enabled_extensions
                .iter()
                .any(|ext| file.ends_with(ext))
        })
        .collect();

    if filtered_files.is_empty() {
        return Ok(String::new());
    }

    // 3. 获取过滤后的文件的 diff 内容
    // git diff --cached -U0 -- file1 file2 ...
    let mut cmd = Command::new("git");
    cmd.args(&["diff", "--cached", "-U0", "--"]);
    cmd.args(&filtered_files);

    let output = cmd
        .output()
        .context("Failed to execute git diff for specific files")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8(output.stdout)?)
}
