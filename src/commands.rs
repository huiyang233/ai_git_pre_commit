use anyhow::{Context, Result};
use colored::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const BASE_URL: &str = "http://47.108.203.93/releases";
#[cfg(windows)]
const BINARY_NAME: &str = "ai_git_pre_commit-windows-amd64.exe";
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const BINARY_NAME: &str = "ai_git_pre_commit-linux-amd64";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const BINARY_NAME: &str = "ai_git_pre_commit-darwin-amd64";
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const BINARY_NAME: &str = "ai_git_pre_commit-darwin-arm64";
#[cfg(not(any(
    windows,
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64")
)))]
const BINARY_NAME: &str = "ai_git_pre_commit"; // Fallback name

// 默认配置文件名
const CONFIG_NAME: &str = ".env";

pub async fn install() -> Result<()> {
    println!("{}", "🚀 开始安装...".blue().bold());

    // 1. 安装配置文件
    install_config().await?;

    // 2. 安装钩子
    install_hook()?;

    // 3. 添加到 PATH 提示
    check_path();

    println!("\n{}", "✅ 安装成功完成！".green().bold());
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let hook_path = Path::new(".git").join("hooks").join("pre-commit");
    
    if hook_path.exists() {
        // 通过读取内容检查是否为我们的钩子
        let content = fs::read_to_string(&hook_path).unwrap_or_default();
        if content.contains("AI Git Pre-Commit Hook") {
            fs::remove_file(&hook_path).context("移除 pre-commit 钩子失败")?;
            println!("{}", "✅ pre-commit 钩子已移除。".green());
        } else {
            println!("{}", "⚠️  发现 pre-commit 钩子，但看起来不是我们的。跳过移除。".yellow());
        }
    } else {
        println!("{}", "ℹ️  未发现 pre-commit 钩子。".blue());
    }
    Ok(())
}

pub async fn update() -> Result<()> {
    println!("{}", "🔄 正在检查更新...".blue().bold());
    
    let current_exe = env::current_exe().context("获取当前可执行文件路径失败")?;
    let download_url = format!("{}/{}", BASE_URL, BINARY_NAME);
    
    println!("正在从 {} 下载", download_url);

    // 下载新二进制文件
    let response = reqwest::get(&download_url).await.context("下载更新失败")?;
    if !response.status().is_success() {
        anyhow::bail!("下载更新失败: HTTP {}", response.status());
    }
    let new_bytes = response.bytes().await.context("读取更新数据失败")?;

    // 处理更新（重命名旧文件，写入新文件）
    update_binary(&current_exe, &new_bytes)?;

    println!("{}", "✅ 更新成功完成！".green().bold());
    println!("{}", "ℹ️  配置文件 (.env) 未更新。如果丢失，请使用 'install' 下载配置文件。".blue());
    Ok(())
}

async fn install_config() -> Result<()> {
    let current_exe = env::current_exe()?;
    let exe_dir = current_exe.parent().context("获取可执行文件目录失败")?;
    let config_path = exe_dir.join(CONFIG_NAME);

    if !config_path.exists() {
        println!("正在下载配置文件...");
        let config_url = format!("{}/{}", BASE_URL, CONFIG_NAME);
        
        let response = reqwest::get(&config_url).await.context("下载配置失败")?;
        if response.status().is_success() {
            let content = response.bytes().await?;
            fs::write(&config_path, content).context("写入配置文件失败")?;
            println!("✅ 配置已安装到 {:?}", config_path);
        } else {
            println!("{}", "❌ 下载配置文件失败。".red());
        }
    } else {
        println!("✅ 配置文件已存在于 {:?}", config_path);
    }
    Ok(())
}

fn install_hook() -> Result<()> {
    if !Path::new(".git").exists() {
        println!("{}", "⚠️  当前目录不是 git 仓库。跳过钩子安装。".yellow());
        return Ok(());
    }

    let hooks_dir = Path::new(".git").join("hooks");
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir).context("创建 hooks 目录失败")?;
    }

    let hook_path = hooks_dir.join("pre-commit");
    let current_exe = env::current_exe()?;
    let exe_dir = current_exe.parent().unwrap(); // Safe unwrap

    // 仅在 Windows 上将路径转换为使用正斜杠，以避免 shell 脚本中的转义问题
    let current_exe_str = if cfg!(windows) {
        current_exe.to_string_lossy().replace('\\', "/")
    } else {
        current_exe.to_string_lossy().to_string()
    };

    let exe_dir_str = if cfg!(windows) {
        exe_dir.to_string_lossy().replace('\\', "/")
    } else {
        exe_dir.to_string_lossy().to_string()
    };

    // 钩子脚本内容
    // 我们使用二进制文件的绝对路径并设置 AI_GIT_ENV_DIR
    let hook_content = format!(
r#"#!/bin/sh
# AI Git Pre-Commit Hook
# Executing binary from: {}

# Set environment to look for .env file in binary directory
if [ -f "{}" ]; then
    export AI_GIT_ENV_DIR="{}"
    "{}"
else
    echo "Error: AI Check binary not found at {}"
    exit 1
fi
"#,
        current_exe_str,
        current_exe_str,
        exe_dir_str,
        current_exe_str,
        current_exe_str
    );

    fs::write(&hook_path, hook_content).context("写入钩子文件失败")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    println!("✅ Pre-commit 钩子已安装到 {:?}", hook_path);
    Ok(())
}

fn check_path() {
    if let Ok(path_var) = env::var("PATH") {
        if let Ok(current_exe) = env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let parent_str = parent.to_string_lossy();
                // 简单检查目录是否在 PATH 中（Windows 不区分大小写？）
                // 为简单起见，我们只做包含检查。
                if !path_var.contains(&*parent_str) {
                    println!("\n{}", "⚠️  警告：二进制目录不在您的 PATH 中。".yellow());
                    println!("   请将 '{}' 添加到您的 PATH 环境变量中", parent_str);
                    println!("   以便您可以从任何地方运行 'ai_git_pre_commit'。");
                }
            }
        }
    }
}

fn update_binary(current_path: &PathBuf, new_bytes: &[u8]) -> Result<()> {
    // 在 Windows 上，我们无法覆盖正在运行的可执行文件。
    // 我们将当前文件重命名为 .old 并写入新文件。
    
    let old_path = current_path.with_extension("old");
    
    // 尝试删除旧备份（如果存在）
    if old_path.exists() {
        let _ = fs::remove_file(&old_path);
    }

    // 将当前文件重命名为 old
    fs::rename(current_path, &old_path).context("Failed to rename current binary")?;
    
    // 写入新二进制文件
    match fs::write(current_path, new_bytes) {
        Ok(_) => {
            // Restore permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&old_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(current_path, perms)?;
            }
            Ok(())
        }
        Err(e) => {
            // Rollback
            let _ = fs::rename(&old_path, current_path);
            Err(anyhow::anyhow!("Failed to write new binary: {}", e))
        }
    }
}
