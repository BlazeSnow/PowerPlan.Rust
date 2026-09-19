//! PowerPlan Tauri 后端。
//!
//! 分层：`core`（电源计划业务核心，不依赖 Tauri 运行时）→ `settings`/`tray`/
//! `commands`（持久化、托盘与前端接口），规范见 references/ 各文档。

pub mod commands;
pub mod core;
pub mod i18n;
pub mod settings;
pub mod tray;

use std::sync::Mutex;
use tauri::Manager;

/// 应用标识（须与 tauri.conf.json 的 identifier 保持一致）。
pub const IDENTIFIER: &str = "com.blazesnow.powerplan";

/// 静默启动参数：开机自启动注册表项附带；命中且托盘启用时主窗口不显示。
pub const SILENT_ARG: &str = "--silent";

/// 构建 Tauri 应用并运行。
pub fn run() {
    // 显示语言在 webview 脚本执行前注入，保证首帧即为偏好语言
    let display_language = settings::read_display_language();
    let init_script = format!(
        "window.__POWERPLAN_LANG__ = {};",
        display_language
            .map(|l| format!("{l:?}"))
            .unwrap_or_else(|| "undefined".into())
    );

    tauri::Builder::default()
        // 单实例必须最先注册：重复启动时聚焦已存在实例
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![SILENT_ARG]),
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .append_invoke_initialization_script(&init_script)
        .setup(|app| {
            let state = settings::SettingsState(Mutex::new(settings::load(app.handle())));
            let snapshot = state.0.lock().unwrap().clone();
            app.manage(state);

            // 菜单事件统一在此注册，菜单每次由当前快照重建
            app.on_menu_event(tray::on_menu_event);
            if snapshot.tray_enabled {
                tray::create(app.handle()).map_err(|e| e.to_string())?;
            }

            // 静默启动或启动到托盘：主窗口不显示；托盘未启用时必须显示，避免无窗口僵死
            let silent = std::env::args().any(|arg| arg == SILENT_ARG);
            let start_to_tray = snapshot.tray_enabled && (silent || snapshot.launch_to_tray);
            if !start_to_tray {
                tray::show_main_window(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // 托盘启用时关闭主窗口=隐藏，应用保活；未启用时按默认直接退出
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let tray_enabled = window
                        .app_handle()
                        .state::<settings::SettingsState>()
                        .0
                        .lock()
                        .unwrap()
                        .tray_enabled;
                    if tray_enabled {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::power::power_list_plans,
            commands::power::power_set_active,
            commands::power::power_duplicate_ultimate,
            commands::power::power_restore_defaults,
            commands::power::power_open_power_options,
            commands::settings::settings_get,
            commands::settings::settings_set_language,
            commands::settings::settings_set_auto_start,
            commands::settings::settings_set_tray,
            commands::settings::settings_set_launch_to_tray,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
