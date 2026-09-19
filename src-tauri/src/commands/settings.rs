//! 设置命令：读取与修改，副作用（自启动注册表、托盘增删、菜单重建）在此触发。

use tauri::{AppHandle, State};

use crate::commands::CommandError;
use crate::settings::{self, Settings, SettingsState};

#[tauri::command]
pub fn settings_get(state: State<SettingsState>) -> Settings {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn settings_set_language(
    app: AppHandle,
    state: State<SettingsState>,
    value: String,
) -> Result<Settings, CommandError> {
    let snapshot = state.update(|s| s.language = value);
    persist(&app, &snapshot)?;
    // 语言即时生效：托盘菜单按新语言重建（前端已先 i18n.changeLanguage）
    crate::tray::update(&app);
    Ok(snapshot)
}

#[tauri::command]
pub fn settings_set_auto_start(
    app: AppHandle,
    state: State<SettingsState>,
    value: bool,
) -> Result<Settings, CommandError> {
    // 先应用到注册表，失败时不改动状态
    settings::apply_auto_start(&app, value)
        .map_err(|e| CommandError::new("App.Status.StartupSettingFailed", &[("0", &e)]))?;
    let snapshot = state.update(|s| s.auto_start_enabled = value);
    persist(&app, &snapshot)?;
    crate::tray::update(&app);
    Ok(snapshot)
}

#[tauri::command]
pub fn settings_set_tray(
    app: AppHandle,
    state: State<SettingsState>,
    value: bool,
) -> Result<Settings, CommandError> {
    let snapshot = state.update(|s| s.tray_enabled = value);
    persist(&app, &snapshot)?;
    if snapshot.tray_enabled {
        crate::tray::update(&app);
    } else {
        crate::tray::remove(&app);
    }
    Ok(snapshot)
}

#[tauri::command]
pub fn settings_set_launch_to_tray(
    app: AppHandle,
    state: State<SettingsState>,
    value: bool,
) -> Result<Settings, CommandError> {
    let snapshot = state.update(|s| s.launch_to_tray = value);
    persist(&app, &snapshot)?;
    Ok(snapshot)
}

fn persist(app: &AppHandle, snapshot: &Settings) -> Result<(), CommandError> {
    settings::persist(app, snapshot)
        .map_err(|e| CommandError::new("Settings.SaveFailed", &[("0", &e)]))
}
