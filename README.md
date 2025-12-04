# AI Git Pre-Commit

这是一个基于 Rust 编写的 Git Pre-commit 钩子工具，它利用 AI（默认支持 DeepSeek）自动检查暂存区（staged）的代码变更。它可以帮助你在提交代码前发现潜在的安全问题、性能隐患、代码风格问题等。

## 功能特性

- **自动代码审查**：在 `git commit` 时自动触发，无需人工干预。
- **智能分析**：基于 LLM（大语言模型）分析代码 diff，提供有针对性的建议。
- **多维度检查**：
  - 🔒 **安全性**：检查潜在的安全漏洞。
  - ⚡ **性能**：识别可能导致性能问题的代码。
  - 🎨 **代码风格**：建议更好的代码风格（可选）。
  - 🗄️ **SQL 检查**：专门针对 SQL 语句的优化和安全检查。
- **高度可配置**：通过 `.env` 文件配置 API Key、模型、检查项等。
- **跨平台**：支持 Windows, macOS, Linux。
- **易于管理**：提供简单的安装、卸载和更新命令。

## 安装

### 1. 编译安装（推荐）

确保你已经安装了 Rust 工具链。

```bash
cargo install --path .
```

### 2. 初始化项目

进入你的 Git 项目根目录，运行以下命令安装钩子和配置文件：

```bash
ai_git_pre_commit install
```

这将执行以下操作：
- 在当前可执行文件目录下载/创建默认的 `.env` 配置文件。
- 在 `.git/hooks/` 目录下创建 `pre-commit` 钩子脚本。

## 配置

安装完成后，请编辑配置文件（通常位于可执行文件同级目录或项目根目录下的 `.env`）以填入你的 API Key。

**配置项说明：**

```ini
# AI API 配置
AI_CHECK_API_KEY=your_api_key_here
AI_CHECK_MODEL=deepseek-chat
AI_CHECK_BASE_URL=https://api.deepseek.com/v1

# 检查选项 (true/false)
AI_CHECK_SECURITY=true      # 启用安全检查
AI_CHECK_PERFORMANCE=true   # 启用性能检查
AI_CHECK_STYLE=false        # 启用代码风格检查 (默认关闭)
AI_CHECK_SQL=true           # 启用 SQL 检查

# 其他配置
AI_CHECK_LANGUAGE=chinese   # 输出语言 (chinese/english)
AI_CHECK_MAX_CHUNK_SIZE=4000 # 单次分析的最大字符数
AI_CHECK_EXTENSIONS=.html,.js,.jsx,.ts,.tsx,.vue,.java,.rs,.py # 需要检查的文件扩展名
```

## 使用方法

### 自动检查

配置完成后，当你执行 `git commit` 时，工具会自动运行：

```bash
git add .
git commit -m "feat: add new feature"
```

- 如果 AI 发现严重问题，提交可能会被拦截（取决于具体的实现逻辑，通常只做警告，除非脚本显式退出非零状态）。
- 如果是合并提交（Merge Commit），工具会自动跳过检查。

### 手动检查

你可以随时手动运行检查，无需提交代码：

```bash
# 检查当前暂存区的变更
ai_git_pre_commit

# 或者显式调用 check 命令
ai_git_pre_commit check
```

### 其他命令

- **更新工具**：
  ```bash
  ai_git_pre_commit update
  ```
  从服务器下载最新版本的二进制文件。

- **卸载钩子**：
  ```bash
  ai_git_pre_commit uninstall
  ```
  移除 `.git/hooks/pre-commit` 文件。

## 开发

如果你想参与开发或自行构建：

1. 克隆仓库：
   ```bash
   git clone <repository-url>
   cd ai_git_pre_commit
   ```

2. 运行：
   ```bash
   cargo run -- check
   ```

3. 构建发布版本：
   ```bash
   cargo build --release
   ```

## 常见问题

**Q: 为什么提交时没有触发检查？**
A: 请检查 `.git/hooks/pre-commit` 文件是否存在且有执行权限。另外，确保你已经把 `ai_git_pre_commit` 添加到了系统 PATH 中，或者钩子脚本中指定了正确的绝对路径。

**Q: 如何跳过检查？**
A: 使用 git 的 `--no-verify` 参数：
   ```bash
   git commit -m "quick fix" --no-verify
   ```
