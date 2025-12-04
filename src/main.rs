mod ai;
mod cli;
mod commands;
mod config;
mod git;
mod prompts;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use std::process::exit;

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();

    // 处理子命令
    if let Some(command) = cli.command {
        match command {
            Commands::Install => {
                commands::install().await?;
                return Ok(());
            }
            Commands::Uninstall => {
                commands::uninstall()?;
                return Ok(());
            }
            Commands::Update => {
                commands::update().await?;
                return Ok(());
            }
            _ => {}
        }
    }

    // 默认运行检查
    run_check().await
}

async fn run_check() -> Result<()> {
    // 加载配置文件
    let config = match config::Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{} {}", "配置错误:".red().bold(), e);
            eprintln!("请检查您的 .env 文件或环境变量。");
            exit(1);
        }
    };

    println!("{}", "AI Git Pre-Commit 检查已启动...".blue().bold());

    // 2. 获取暂存区的代码变更 diff
    // 如果当前是合并提交，则跳过检查
    if let Ok(true) = git::is_merge_in_progress() {
        println!("{}", "检测到合并操作。跳过 AI 检查以避免分析大量合并代码。".yellow());
        exit(0);
    }

    let diff = match git::get_staged_diff(&config) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{} {}", "Git 错误:".red().bold(), e);
            exit(1);
        }
    };

    if diff.trim().is_empty() {
        println!(
            "{}",
            "在监控的文件中未发现暂存的更改。跳过检查。".yellow()
        );
        exit(0);
    }

    println!(
        "{} 正在分析 {} 个字符的代码变更...",
        "处理中:".cyan(),
        diff.len()
    );

    // 3. 生成提示词
    let system_prompt = prompts::generate_system_prompt(&config);

    // 4. 调用 AI
    let result = match ai::call_ai_check(&config, system_prompt, diff).await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{} {}", "AI 检查失败:".red().bold(), e);
            exit(1);
        }
    };

    // 5. 显示结果
    println!("\n{}", "分析结果:".bold().underline());
    
    // 显示 Token 使用情况
    if let Some(usage) = &result.usage {
        println!(
            "{} 输入: {} tokens, 输出: {} tokens, 总计: {} tokens",
            "Token 使用:".purple(),
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens
        );
    }

    // 显示 AI 锐评
    if let Some(comment) = &result.meme_comment {
        println!("\n{}", "AI 锐评:".magenta().bold());
        println!("{}", comment.italic());
    }

    for issue in &result.list {
        let severity_color = match issue.severity.to_lowercase().as_str() {
            "high" => "red",
            "medium" => "yellow",
            "low" => "green",
            _ => "white",
        };

        println!(
            "\n[{}] [{}] {}",
            issue.severity.color(severity_color).bold(),
            issue.perspective.cyan(),
            issue.location.white().italic()
        );
        println!("  Description: {}", issue.description);
        println!("  Suggestion:  {}", issue.suggestion);
    }

    println!();

    if result.result.to_uppercase().contains("YES") {
        println!("{}", "✅ 代码已通过！".green().bold());
        exit(0);
    } else {
        println!(
            "{}",
            "❌ 代码被拒绝，发现严重问题。".red().bold()
        );
        exit(1);
    }
}
