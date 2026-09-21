//! 设置命令：读取与修改，副作用（自启动注册表/StartupTask、托盘增删、菜单重建）在此触发。

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::autostart;
use crate::commands::CommandError;
use crate::settings::{self, Settings, SettingsState};

/// 设置视图：持久化设置 + 系统侧实际自启动状态（供前端显示信息）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    #[serde(flatten)]
    pub settings: Settings,
    pub auto_start_state: String,
}

impl SettingsView {
    pub fn new(app: &AppHandle, settings: Settings) -> Self {
        Self {
            settings,
            auto_start_state: autostart::state(app).as_str().into(),
        }
    }
}

#[tauri::command]
pub fn settings_get(app: AppHandle, state: State<SettingsState>) -> SettingsView {
    SettingsView::new(&app, state.0.lock().unwrap().clone())
}

#[tauri::command]
pub fn settings_set_language(
    app: AppHandle,
    state: State<SettingsState>,
    value: String,
) -> Result<SettingsView, CommandError> {
    let snapshot = state.update(|s| s.language = value);
    persist(&app, &snapshot)?;
    // 语言即时生效：托盘菜单按新语言重建（前端已先 i18n.changeLanguage）
    crate::tray::update(&app);
    Ok(SettingsView::new(&app, snapshot))
}

#[tauri::command]
pub fn settings_set_auto_start(
    app: AppHandle,
    state: State<SettingsState>,
    value: bool,
) -> Result<SettingsView, CommandError> {
    // 双模式分发：打包版 StartupTask / 未打包注册表；失败时系统状态未变
    if autostart::is_packaged() {
        autostart::set_enabled(value).map_err(|reason| {
            if reason == "disabled_by_user" {
                CommandError::new("App.Status.StartupSettingDisabledByUser", &[])
            } else {
                CommandError::new("App.Status.StartupSettingFailed", &[("0", &reason)])
            }
        })?;
    } else {
        // 先应用到注册表，失败时不改动状态
        autostart::set_registry_enabled(&app, value)
            .map_err(|e| CommandError::new("App.Status.StartupSettingFailed", &[("0", &e)]))?;
    }
    let snapshot = state.update(|s| s.auto_start_enabled = value);
    persist(&app, &snapshot)?;
    crate::tray::update(&app);
    Ok(SettingsView::new(&app, snapshot))
}

#[tauri::command]
pub fn settings_set_tray(
    app: AppHandle,
    state: State<SettingsState>,
    value: bool,
) -> Result<SettingsView, CommandError> {
    let snapshot = state.update(|s| s.tray_enabled = value);
    persist(&app, &snapshot)?;
    if snapshot.tray_enabled {
        crate::tray::update(&app);
    } else {
        crate::tray::remove(&app);
    }
    Ok(SettingsView::new(&app, snapshot))
}

#[tauri::command]
pub fn settings_set_launch_to_tray(
    app: AppHandle,
    state: State<SettingsState>,
    value: bool,
) -> Result<SettingsView, CommandError> {
    let snapshot = state.update(|s| s.launch_to_tray = value);
    persist(&app, &snapshot)?;
    Ok(SettingsView::new(&app, snapshot))
}

fn persist(app: &AppHandle, snapshot: &Settings) -> Result<(), CommandError> {
    settings::persist(app, snapshot)
        .map_err(|e| CommandError::new("Settings.SaveFailed", &[("0", &e)]))
}
