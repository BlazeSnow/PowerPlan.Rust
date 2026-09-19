//! 主题命令：当前系统深浅色（前端挂载时兜底查询，实时变化走 system-theme 事件）。

use crate::system_theme;

#[tauri::command]
pub fn system_theme() -> &'static str {
    system_theme::current_system_theme()
}
