//! 前端↔后端接口（Tauri command），规范见 references/architecture.md。

pub mod power;
pub mod settings;
pub mod theme;

use std::collections::HashMap;

use serde::Serialize;

/// 命令错误：本地化键名 + 位置参数（前端 i18next 以 {{0}} 插值渲染）。
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub key: String,
    pub args: HashMap<String, String>,
}

impl CommandError {
    pub fn new(key: &str, args: &[(&str, &str)]) -> Self {
        Self {
            key: key.to_string(),
            args: args
                .iter()
                .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_error_builds_named_args() {
        let error = CommandError::new("Main.Status.SwitchFailed", &[("0", "denied")]);
        assert_eq!(error.key, "Main.Status.SwitchFailed");
        assert_eq!(error.args.get("0").map(String::as_str), Some("denied"));
        assert_eq!(error.args.len(), 1);
    }

    #[test]
    fn command_error_without_args_serializes_key_only_payload() {
        let error = CommandError::new("PowerPlan.Error.EmptyName", &[]);
        let json = serde_json::to_value(&error).expect("serialize");
        assert_eq!(json["key"], "PowerPlan.Error.EmptyName");
        assert_eq!(json["args"], serde_json::json!({}));
    }
}
