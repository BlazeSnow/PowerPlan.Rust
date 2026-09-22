//! PowerPlan Tauri 后端。
//!
//! 分层：`core`（电源计划业务核心，不依赖 Tauri 运行时）→ `settings`/`tray`/
//! `commands`（持久化、托盘与前端接口），规范见 references/ 各文档。

pub mod autostart;
pub mod commands;
pub mod core;
pub mod efficiency;
pub mod i18n;
pub mod settings;
pub mod system_theme;
pub mod tray;
pub mod tray_theme;
pub mod window;

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
        // 单实例必须最先注册：重复启动时聚焦已存在实例（重建在 ensure_main_window
        // 内异步执行，回调立即返回，避免与插件的同步 SendMessage 死锁）
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        // 保存并恢复用户手动调整的窗口大小与位置；窗口默认隐藏（静默启动），
        // 必须排除 VISIBLE 标志，避免恢复出可见状态破坏静默启动
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED
                        | tauri_plugin_window_state::StateFlags::FULLSCREEN,
                )
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![SILENT_ARG]),
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .append_invoke_initialization_script(&init_script)
        .setup(|app| {
            // 深色兼容层须早于首个托盘菜单创建
            tray_theme::init();
            let state = settings::SettingsState(Mutex::new(settings::load(app.handle())));
            let snapshot = state.0.lock().unwrap().clone();
            app.manage(state);

            // 菜单事件统一在此注册，菜单每次由当前快照重建
            app.on_menu_event(tray::on_menu_event);
            if snapshot.tray_enabled {
                tray::create(app.handle()).map_err(|e| e.to_string())?;
            }

            // 静默启动或启动到托盘：主窗口不显示；托盘未启用时必须显示，避免无窗口僵死
            // 静默依据：--silent 参数（未打包自启动），或登录时 StartupTask 激活（MSIX 打包版）
            let silent = std::env::args().any(|arg| arg == SILENT_ARG)
                || autostart::is_startup_task_launch();
            let start_to_tray = snapshot.tray_enabled && (silent || snapshot.launch_to_tray);
            if !start_to_tray {
                tray::show_main_window(app.handle());
            } else {
                // 托盘常驻（无主窗口）：进入效能模式，等窗口打开时恢复
                efficiency::set_enabled(true);
            }
            // 电源模式变化（如最佳性能↔平衡）后自动重放 EcoQoS
            efficiency::watch_power_changes(app.handle());
            window::fit_main_window(app.handle());
            // 系统深浅色监听：变化时设置窗口原生主题并通知前端
            system_theme::start_theme_watcher(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // 托盘启用时关闭主窗口=保存几何后销毁webview（不再占用其内存），
            // 之后由单实例回调或托盘"打开主窗口"按配置重建；
            // 托盘未启用时放行关闭，窗口销毁后应用自然退出
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && window.label() == "main" {
                    // 销毁前把用户调整的几何写盘（插件的保存入口在应用句柄上）
                    use tauri_plugin_window_state::{AppHandleExt as _, StateFlags};
                    let _ = window.app_handle().save_window_state(
                        StateFlags::SIZE
                            | StateFlags::POSITION
                            | StateFlags::MAXIMIZED
                            | StateFlags::FULLSCREEN,
                    );
                    let tray_enabled = window
                        .app_handle()
                        .state::<settings::SettingsState>()
                        .0
                        .lock()
                        .unwrap()
                        .tray_enabled;
                    if tray_enabled {
                        api.prevent_close();
                        let _ = window.destroy();
                        // 回到托盘常驻（无主窗口）：进入效能模式
                        efficiency::set_enabled(true);
                    }
                }
        })
        .invoke_handler(tauri::generate_handler![
            commands::power::power_list_plans,
            commands::power::power_set_active,
            commands::power::power_copy_plan,
            commands::power::power_duplicate_ultimate,
            commands::power::power_clear_saved_ultimate,
            commands::power::power_restore_defaults,
            commands::power::power_open_power_options,
            commands::settings::settings_get,
            commands::settings::settings_set_language,
            commands::settings::settings_set_auto_start,
            commands::settings::settings_set_tray,
            commands::settings::settings_set_launch_to_tray,
            commands::theme::system_theme,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| match event {
            // 托盘启用时窗口全部销毁不应退出应用：code=None 表示窗口关闭触发，
            // 按托盘开关决定是否阻止；code=Some 表示显式 app.exit（托盘退出等），照常退出
            tauri::RunEvent::ExitRequested { code, api, .. }
                if code.is_none() => {
                    let tray_enabled = app
                        .try_state::<settings::SettingsState>()
                        .map(|state| state.0.lock().unwrap().tray_enabled)
                        .unwrap_or(false);
                    if tray_enabled {
                        api.prevent_exit();
                    }
                }
            _ => {}
        });
}
