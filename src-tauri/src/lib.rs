//! PowerPlan Tauri 后端。
//!
//! 分层：`core`（电源计划业务核心，不依赖 Tauri 运行时）→ `settings`/`tray`/
//! `commands`（持久化、托盘与前端接口），规范见 references/ 各文档。

pub mod commands;
pub mod core;
pub mod i18n;
pub mod settings;
pub mod system_theme;
pub mod tray;
pub mod tray_theme;

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
            let silent = std::env::args().any(|arg| arg == SILENT_ARG);
            let start_to_tray = snapshot.tray_enabled && (silent || snapshot.launch_to_tray);
            if !start_to_tray {
                            tray::show_main_window(app.handle());
            }
            fit_main_window(app.handle());
            // 系统深浅色监听：变化时设置窗口原生主题并通知前端
            system_theme::start_theme_watcher(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // 托盘启用时关闭主窗口=保存几何后销毁webview（不再占用其内存），
            // 之后由单实例回调或托盘"打开主窗口"按配置重建；
            // 托盘未启用时放行关闭，窗口销毁后应用自然退出
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
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
                    }
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
            tauri::RunEvent::ExitRequested { code, api, .. } => {
                if code.is_none() {
                    let tray_enabled = app
                        .try_state::<settings::SettingsState>()
                        .map(|state| state.0.lock().unwrap().tray_enabled)
                        .unwrap_or(false);
                    if tray_enabled {
                        api.prevent_exit();
                    }
                }
            }
            _ => {}
        });
}

/// 确保主窗口存在并显示：已销毁（托盘常驻下用户关闭后）时按配置重建。
///
/// 重建不能在主线程消息处理（单实例 WM_COPYDATA、托盘菜单点击）中同步执行：
/// WebView2 创建需要泵消息，嵌套等待会死锁。因此销毁后的重建移到独立线程，
/// tauri 会把窗口创建派发回主线程，届时消息处理已结束；窗口已存在时仅同步
/// 显示聚焦。重建的窗口由 window-state 插件恢复几何，前端挂载后自行拉取状态。
pub fn ensure_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }
    if REBUILDING.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let config = app
            .config()
            .app
            .windows
            .iter()
            .find(|window| window.label == "main")
            .cloned();
        let Some(config) = config else {
            REBUILDING.store(false, std::sync::atomic::Ordering::SeqCst);
            return;
        };
        let result = tauri::webview::WebviewWindowBuilder::from_config(&app, &config)
            .and_then(|builder| builder.build());
        REBUILDING.store(false, std::sync::atomic::Ordering::SeqCst);
        if let Ok(window) = result {
            let _ = window.show();
            let _ = window.set_focus();
            fit_main_window(&app);
        }
    });
}

/// 重建防重入：单实例与托盘可能并发触发
static REBUILDING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 主窗口启动适配：钳制在当前显示器工作区（去除任务栏）内。
///
/// 窗口几何在此时已被 window-state 插件恢复（或为 tauri.conf.json 默认值）：
/// 尺寸超限收缩、位置越界回位，用户已保存的几何尽量保留；无状态文件的
/// 首次启动才居中。内置 center() 以整块显示器为基准，任务栏在下方时仍会
/// 压入，故按 work_area 自行计算（对齐参考实现的成熟做法）。
fn fit_main_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_maximized().unwrap_or(false) {
        return;
    }
    let Ok(Some(monitor)) = window.current_monitor() else {
        return;
    };
    let scale = monitor.scale_factor();
    let work_area = monitor.work_area();
    let avail_w = work_area.size.width as f64 / scale;
    let avail_h = work_area.size.height as f64 / scale;

    // 尺寸：恢复值/默认值超出工作区才收缩
    let Ok(size) = window.outer_size() else {
        return;
    };
    let cur_w = size.width as f64 / scale;
    let cur_h = size.height as f64 / scale;
    let width = cur_w.min(avail_w);
    let height = cur_h.min(avail_h);
    if width != cur_w || height != cur_h {
        let _ = window.set_size(tauri::LogicalSize::new(width, height));
    }

    // 位置：越界回位（负坐标、压任务栏、被副屏甩出等）
    let win_w = (width * scale).round() as i32;
    let win_h = (height * scale).round() as i32;
    let max_x = (work_area.size.width as i32 - win_w).max(0) + work_area.position.x;
    let max_y = (work_area.size.height as i32 - win_h).max(0) + work_area.position.y;
    let Ok(pos) = window.outer_position() else {
        return;
    };
    let x = pos.x.clamp(work_area.position.x, max_x);
    let y = pos.y.clamp(work_area.position.y, max_y);
    if x != pos.x || y != pos.y {
        let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
        return;
    }

    // 位置在工作区内且无已保存状态（首次启动）：在工作区居中
    let first_run = app
        .path()
        .app_config_dir()
        .ok()
        .is_some_and(|dir| !dir.join(tauri_plugin_window_state::DEFAULT_FILENAME).exists());
    if first_run {
        let _ = window.set_position(tauri::PhysicalPosition::new(
            work_area.position.x
                + ((work_area.size.width as f64 - width * scale) / 2.0).round() as i32,
            work_area.position.y
                + ((work_area.size.height as f64 - height * scale) / 2.0).round() as i32,
        ));
    }
}
