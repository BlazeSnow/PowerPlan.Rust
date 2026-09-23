//! 应用级命令：托盘「打开设置」的挂起导航标记消费。

/// 前端挂载/收到 `open-settings` 事件时调用：取走并清零挂起标记。
/// webview 销毁重建场景下事件早于监听注册，挂载时消费兜底
/// （references/tray.md「托盘」）。
#[tauri::command]
pub fn take_open_settings_request() -> bool {
    crate::tray::take_open_settings_request()
}
