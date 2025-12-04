use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageContent,
}

#[derive(Deserialize)]
struct OpenAIMessageContent {
    content: String,
}

#[derive(Deserialize, Debug)]
pub struct Issue {
    pub severity: String,
    pub perspective: String,
    pub description: String,
    pub suggestion: String,
    pub location: String,
}

#[derive(Deserialize, Debug)]
pub struct AiCheckResult {
    pub result: String,
    #[serde(default)]
    pub meme_comment: Option<String>,
    pub list: Vec<Issue>,
    #[serde(skip)]
    pub usage: Option<TokenUsage>,
}

#[derive(Debug)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub async fn call_ai_check(
    config: &Config,
    system_prompt: String,
    diff: String,
) -> Result<AiCheckResult> {
    let client = Client::new();

    // 如果 diff 太长，根据 max_chunk_size 进行截断（粗略估计）
    // 用户配置了 "maxChunkSize"，我们将其作为 diff 的字符限制。
    let diff_content = if diff.len() > config.max_chunk_size {
        // 在实际应用中，我们可能会拆分或总结。这里我们进行截断并在日志/输出中警告。
        // 目前，只取前 N 个字符。
        format!(
            "{}\n\n[Diff truncated due to size limit...]",
            &diff[..config.max_chunk_size]
        )
    } else {
        diff
    };

    let request_body = OpenAIRequest {
        model: config.model.clone(),
        messages: vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: format!("这是需要审查的 git diff:\n\n{}", diff_content),
            },
        ],
    };

    // 构造 URL。处理尾部斜杠。
    let url = if config.base_url.ends_with('/') {
        format!("{}chat/completions", config.base_url)
    } else {
        format!("{}/chat/completions", config.base_url)
    };

    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .context("发送请求给 AI 提供商失败")?;

    if !res.status().is_success() {
        let error_text = res.text().await?;
        return Err(anyhow::anyhow!("AI API 请求失败: {}", error_text));
    }

    let response_body: OpenAIResponse = res
        .json()
        .await
        .context("解析 AI 响应 JSON 失败")?;

    let content = response_body
        .choices
        .first()
        .context("AI 响应中没有 choices")?
        .message
        .content
        .trim();
    // 内容应该是 JSON。
    // 有时 AI 会将其包装在 ```json ... ``` 中。我们需要去除它。
    let json_str = if let Some(start) = content.find("```json") {
        if content[start..].find("```").is_some() {
            // 如果有多个块，这个逻辑有点缺陷，但通常只有一个。
            // 让我们尝试找到第一个 '{' 和最后一个 '}'
            find_json_bounds(content)
        } else {
            find_json_bounds(content)
        }
    } else {
        find_json_bounds(content)
    };

    let mut check_result: AiCheckResult = serde_json::from_str(json_str)
        .context(format!("无法将 AI 输出解析为 JSON。内容: {}", content))?;
    
    if let Some(usage) = response_body.usage {
        check_result.usage = Some(TokenUsage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        });
    }

    Ok(check_result)
}

fn find_json_bounds(s: &str) -> &str {
    let start = s.find('{').unwrap_or(0);
    let end = s.rfind('}').map(|i| i + 1).unwrap_or(s.len());
    &s[start..end]
}
