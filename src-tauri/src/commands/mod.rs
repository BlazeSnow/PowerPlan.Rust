//! 前端↔后端接口（Tauri command），规范见 references/architecture.md。

pub mod power;
pub mod settings;

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
