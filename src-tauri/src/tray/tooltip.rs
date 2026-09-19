//! 托盘提示文本：三行式对齐旧版 TrayTooltipFormatter。

use tauri::{AppHandle, Manager};

use super::{language, product_name, TRAY_ID};
use crate::core::power;
use crate::settings::SettingsState;

/// 提示文本三行式：标题 / 当前计划 / 自启动状态。
pub(super) fn update(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let lang = language(app);
    let auto_start_enabled = app
        .state::<SettingsState>()
        .0
        .lock()
        .unwrap()
        .auto_start_enabled;

    let title = product_name(app);
    let plan_text = match power::active_scheme() {
        Ok(guid) => match power::friendly_name(guid) {
            Ok(name) if !name.trim().is_empty() => {
                lang.message_with("tray-tooltip-plan", &[("plan", &name)])
            }
            _ => lang.message("tray-tooltip-plan-unavailable"),
        },
        Err(_) => lang.message("tray-tooltip-plan-unavailable"),
    };
    let state = lang.message(if auto_start_enabled {
        "tray-tooltip-state-on"
    } else {
        "tray-tooltip-state-off"
    });
    let tooltip = format!(
        "{}\n{}\n{}",
        title,
        plan_text,
        lang.message_with("tray-tooltip-autostart", &[("state", &state)])
    );
    // Windows 提示文本上限 128 字符（含 NUL），对齐旧版截断长度
    let tooltip: String = tooltip.chars().take(127).collect();
    let _ = tray.set_tooltip(Some(tooltip));
}
