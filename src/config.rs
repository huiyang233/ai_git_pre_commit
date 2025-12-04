use anyhow::Result;
use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub max_chunk_size: usize,
    pub language: String,
    pub check_security: bool,
    pub check_performance: bool,
    pub check_style: bool,
    pub check_sql: bool,
    pub enabled_extensions: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut loaded = false;

        // 1. 尝试从环境变量中获取配置文件路径
        if let Ok(env_dir) = std::env::var("AI_GIT_ENV_DIR") {
            let env_path = std::path::Path::new(&env_dir).join(".env");
            if env_path.exists() {
                dotenv::from_path(&env_path).ok();
                println!("📄 Loaded configuration from: {:?}", env_path);
                loaded = true;
            }
        }

        // 2. 尝试从当前可执行文件所在目录加载配置文件
        if !loaded {
            if let Ok(current_exe) = env::current_exe() {
                if let Some(exe_dir) = current_exe.parent() {
                    let env_path = exe_dir.join(".env");
                    if env_path.exists() {
                        dotenv::from_path(&env_path).ok();
                        println!("📄 Loaded configuration from: {:?}", env_path);
                        loaded = true;
                    }
                }
            }
        }

        // 3. 如果以上都没有加载到配置文件，则尝试从当前目录加载配置文件
        if !loaded {
             // dotenv() 会搜索父目录，所以我们不能确定确切的路径。
             // 但是为了简单起见，我们可以检查当前目录是否存在 .env 文件。
             if std::path::Path::new(".env").exists() {
                 println!("📄 Loaded configuration from current directory (.env)");
             }
             dotenv().ok();
        }

        let api_key = env::var("AI_CHECK_API_KEY").unwrap_or_else(|_| "sk-c586d498347c4428830f974f367463b4".to_string());

        let model = env::var("AI_CHECK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string());

        let base_url = env::var("AI_CHECK_BASE_URL")
            .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string());

        let max_chunk_size = env::var("AI_CHECK_MAX_CHUNK_SIZE")
            .unwrap_or_else(|_| "4000".to_string())
            .parse()
            .unwrap_or(4000);

        let language = env::var("AI_CHECK_LANGUAGE").unwrap_or_else(|_| "chinese".to_string());

        let check_security = env::var("AI_CHECK_SECURITY")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let check_performance = env::var("AI_CHECK_PERFORMANCE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let check_style = env::var("AI_CHECK_STYLE")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let check_sql = env::var("AI_CHECK_SQL")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let extensions_str = env::var("AI_CHECK_EXTENSIONS")
            .unwrap_or_else(|_| ".html,.js,.jsx,.ts,.tsx,.vue,.java,.rs,.py".to_string());

        let enabled_extensions = extensions_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Config {
            api_key,
            model,
            base_url,
            max_chunk_size,
            language,
            check_security,
            check_performance,
            check_style,
            check_sql,
            enabled_extensions,
        })
    }
}
