use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ai_git_pre_commit")]
#[command(about = "AI-powered Git Pre-commit Hook", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 安装 pre-commit 钩子和配置
    Install,
    /// 卸载 pre-commit 钩子
    Uninstall,
    /// 更新二进制文件到最新版本
    Update,
    /// 运行检查（如果未提供命令，则为默认行为）
    Check,
}
