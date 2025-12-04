use crate::config::Config;
use serde_json::json;

pub fn generate_system_prompt(config: &Config) -> String {
    let mut rules = serde_json::Map::new();
    let mut perspectives = vec!["general".to_string()];

    // 通用检查总是包含的
    rules.insert("general".to_string(), json!({
        "name": "通用检查:",
        "checks": [
            "代码中的潜在Bug",
            "代码的可读性",
            "改进建议",
        ],
        "severity_guidance": "严重问题使用 high，中等问题使用 medium，轻微建议使用 low"
    }));

    if config.check_security {
        perspectives.push("security".to_string());
        rules.insert(
            "security".to_string(),
            json!({
                "name": "安全性:",
                "checks": [
                    "XSS 漏洞",
                    "CSRF 保护",
                    "CORS 配置",
                    "第三方脚本安全",
                    "多线程下的潜在问题"
                ],
                "severity_guidance": "严重漏洞使用 high，潜在风险使用 medium"
            }),
        );
    }

    if config.check_performance {
        perspectives.push("performance".to_string());
        rules.insert("performance".to_string(), json!({
            "name": "性能:",
            "checks": [
                "算法变更的影响",
                "内存使用模式",
                "I/O 操作变更",
                "并发修改",
            ],
            "severity_guidance": "严重瓶颈（如死循环、栈溢出等）使用 high，优化机会使用 medium"
        }));
    }

    if config.check_style {
        perspectives.push("style".to_string());
        rules.insert(
            "style".to_string(),
            json!({
                "name": "代码风格:",
                "checks": [
                    "命名一致性",
                    "代码组织变更",
                    "文档更新",
                    "风格指南遵循情况"
                ],
                "severity_guidance": "风格建议使用 low"
            }),
        );
    }

    if config.check_sql {
        perspectives.push("database".to_string());
        rules.insert(
            "database".to_string(),
            json!({
                "name": "数据库:",
                "checks": [
                    "SQL 注入漏洞",
                    "SQL 语句正确性",
                    "查询性能优化",
                    "事务使用正确性",
                    "数据库连接管理",
                    "数据一致性",
                ],
                "severity_guidance": "严重漏洞（如 SQL 注入、数据库连接泄露）使用 high，性能问题使用 medium，规范问题使用 low"
            }),
        );
    }

    let prompt_structure = json!({
        "system": "你是一位专业的代码审查专家，正在分析 git diff -U0 格式的代码变更。你的主要关注点应是新增和修改的代码部分，忽略已删除的部分。请严格按照以下维度进行审查，不要引入无关的视角：",
        "instruction": "从这些视角进行分析",
        "rules": rules,
        "response": {
            "requirement": "输出要求：\n请返回包含以下字段的 JSON：",
            "fields": {
                "result": "如果有高严重性 (high severity) 问题，返回 NO (rejected)；否则返回 YES (approved)",
                "meme_comment": "使用中国网络热梗对代码进行简短、幽默且犀利的整体评价（可以不用友好）",
                "list": "发现的问题列表，包含以下详情："
            },
            "itemFields": {
                "severity": "high/medium/low",
                "perspective": perspectives.join("/"),
                "description": format!("用{}描述问题", config.language),
                "suggestion": format!("用{}给出修复建议", config.language),
                "location": "文件路径和行号，格式为：'path:line_number' (例如 src/utils.js:15)"
            }
        }
    });

    // 将结构转换为可读字符串以指导 AI
    // 我们希望 AI 将此结构视为其指令。
    // 用户的提示是一个 JS 对象，所以我们可以将此 JSON 转储为系统消息。
    // 或者我们可以将其格式化得很好。AI 通常能很好地理解 JSON 指令。

    serde_json::to_string_pretty(&prompt_structure).unwrap()
}
