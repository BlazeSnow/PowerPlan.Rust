//! 设置持久化：tauri-plugin-store 单文件 JSON（应用数据目录 settings.json）。
//! 字段与默认值见 references/settings.md「持久化设置」；全部读写经 SettingsState 同步。
//! 旧 WinUI 版（LocalSettings）数据不迁移（references/settings.md）。

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Store, StoreExt};

use crate::IDENTIFIER;

pub const STORE_FILE: &str = "settings.json";

/// 设置快照（camelCase 序列化给前端）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// 显示语言："auto" 或受支持语言代码（references/i18n.md）
    pub language: String,
    pub auto_start_enabled: bool,
    pub tray_enabled: bool,
    pub launch_to_tray: bool,
    pub ultimate_performance_plan_guid: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "auto".into(),
            auto_start_enabled: false,
            tray_enabled: true,
            launch_to_tray: false,
            ultimate_performance_plan_guid: None,
        }
    }
}

pub struct SettingsState(pub Mutex<Settings>);

impl SettingsState {
    /// 在锁内修改并返回快照。
    pub fn update(&self, f: impl FnOnce(&mut Settings)) -> Settings {
        let mut guard = self.0.lock().unwrap();
        f(&mut guard);
        guard.clone()
    }
}

/// 启动时从 store 读取（无值用默认），返回初始快照。
pub fn load(app: &AppHandle) -> Settings {
    let mut settings = Settings::default();
    let Ok(store) = app.store(STORE_FILE) else {
        return settings;
    };
    if let Some(value) = get_str(&store, "language") {
        settings.language = value;
    }
    if let Some(value) = get_bool(&store, "auto_start_enabled") {
        settings.auto_start_enabled = value;
    }
    if let Some(value) = get_bool(&store, "tray_enabled") {
        settings.tray_enabled = value;
    }
    if let Some(value) = get_bool(&store, "launch_to_tray") {
        settings.launch_to_tray = value;
    }
    settings.ultimate_performance_plan_guid = get_str(&store, "ultimate_performance_plan_guid");
    settings
}

/// 将快照写回 store。
pub fn persist(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set("language", json!(settings.language));
    store.set("auto_start_enabled", json!(settings.auto_start_enabled));
    store.set("tray_enabled", json!(settings.tray_enabled));
    store.set("launch_to_tray", json!(settings.launch_to_tray));
    store.set(
        "ultimate_performance_plan_guid",
        json!(settings.ultimate_performance_plan_guid),
    );
    store.save().map_err(|e| e.to_string())
}

/// 应用开机自启动到注册表（HKCU Run，附带静默参数）。设置页与托盘共用。
pub fn apply_auto_start(app: &AppHandle, value: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt as _;
    let launcher = app.autolaunch();
    if value {
        launcher.enable()
    } else {
        launcher.disable()
    }
    .map_err(|e| e.to_string())
}

/// 启动阶段（webview 注入脚本前）读取显示语言：直接解析 store 文件，不依赖 Tauri 运行时。
pub fn read_display_language() -> Option<String> {
    let path = dirs::config_dir()?.join(IDENTIFIER).join(STORE_FILE);
    let raw = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    value
        .get("language")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn get_str(store: &Store<Wry>, key: &str) -> Option<String> {
    store.get(key)?.as_str().map(str::to_string)
}

fn get_bool(store: &Store<Wry>, key: &str) -> Option<bool> {
    store.get(key)?.as_bool()
}
