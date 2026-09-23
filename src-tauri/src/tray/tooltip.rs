//! 托盘提示文本：两行式（标题 / 当前计划）；自启动行随托盘自启动
//! 开关一并移除（references/tray.md）。

use tauri::AppHandle;

use super::{language, product_name, TRAY_ID};
use crate::core::power;

/// 提示文本两行式：标题 / 当前计划。
pub(super) fn update(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let lang = language(app);

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
    let tooltip = format!("{}\n{}", title, plan_text);
    // Windows 提示文本上限 128 字符（含 NUL），对齐旧版截断长度
    let tooltip: String = tooltip.chars().take(127).collect();
    let _ = tray.set_tooltip(Some(tooltip));
}
