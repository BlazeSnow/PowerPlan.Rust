//! 托盘：Tauri 2 内置 tray-icon，菜单每次由当前快照重建（references/tray.md）。
//! 菜单文案经后端 Fluent（crate::i18n）本地化，跟随设置语言即时刷新。
//! 菜单构建见 [`menu`]，提示文本见 [`tooltip`]。

mod menu;
mod tooltip;

use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager};

use crate::core::power;
use crate::i18n::Lang;
use crate::settings::SettingsState;

pub const TRAY_ID: &str = "powerplan-tray";
pub(super) const TITLE_ID: &str = "menu-title";
pub(super) const OPEN_ID: &str = "open-main-window";
pub(super) const HIDDEN_ULTIMATE_ID: &str = "activate-hidden-ultimate";
pub(super) const REFRESH_ID: &str = "refresh-plans";
pub(super) const AUTOSTART_ID: &str = "autostart-toggle";
pub(super) const QUIT_ID: &str = "quit";
pub(super) const PLAN_PREFIX: &str = "plan-";

/// 应用标题（读自 tauri.conf.json 的 productName，不硬编码可见字符串）。
pub(super) fn product_name(app: &AppHandle) -> String {
    app.config()
        .product_name
        .clone()
        .unwrap_or_else(|| "PowerPlan".into())
}

/// 菜单与提示共用的文案语言（读自设置）。
pub(super) fn language(app: &AppHandle) -> Lang {
    Lang::from_config(&app.state::<SettingsState>().0.lock().unwrap().language)
}

/// 创建托盘（仅托盘启用时调用）。
pub fn create(app: &AppHandle) -> Result<(), String> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "missing window icon".to_string())?;
    let menu = menu::build_menu(app).map_err(|e| e.to_string())?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .build(app)
        .map_err(|e| e.to_string())?;
    tooltip::update(app);
    // 菜单创建后刷新深浅色主题缓存（对齐旧版动态菜单刷新）
    crate::tray_theme::refresh();
    Ok(())
}

/// 重建菜单与提示：设置、计划或语言变化后调用；托盘不存在时按设置创建。
pub fn update(app: &AppHandle) {
    if app.tray_by_id(TRAY_ID).is_none() {
        let tray_enabled = app
            .state::<SettingsState>()
            .0
            .lock()
            .unwrap()
            .tray_enabled;
        if tray_enabled {
            let _ = create(app);
        }
        return;
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Ok(menu) = menu::build_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
        tooltip::update(app);
        crate::tray_theme::refresh();
    }
}

/// 移除托盘（关闭托盘开关 / 退出）。
pub fn remove(app: &AppHandle) {
    let _ = app.remove_tray_by_id(TRAY_ID);
}

/// 显示并聚焦主窗口：已销毁时按配置重建，并通知前端刷新计划状态。
pub fn show_main_window(app: &AppHandle) {
    crate::window::ensure_main_window(app);
    let _ = app.emit("plans-changed", ());
}

/// 托盘菜单事件（setup 时经 app.on_menu_event 注册）。
pub fn on_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    if let Some(guid) = id.strip_prefix(PLAN_PREFIX) {
        if let Ok(guid) = uuid::Uuid::parse_str(guid)
            && power::set_active_scheme(guid).is_ok() {
                power::invalidate_plans_cache();
                update(app);
                // 主窗口可能正处于打开状态，通知其刷新计划状态
                let _ = app.emit("plans-changed", ());
            }
        return;
    }
    match id {
        OPEN_ID => show_main_window(app),
        HIDDEN_ULTIMATE_ID => activate_hidden_ultimate(app),
        REFRESH_ID => {
            // 强制刷新缓存并重建菜单，通知已打开的主窗口
            power::invalidate_plans_cache();
            update(app);
            let _ = app.emit("plans-changed", ());
        }
        AUTOSTART_ID => {
            let next = !app
                .state::<SettingsState>()
                .0
                .lock()
                .unwrap()
                .auto_start_enabled;
            // 双模式分发：打包版 StartupTask / 未打包注册表
            let result = if crate::autostart::is_packaged() {
                crate::autostart::set_enabled(next)
            } else {
                crate::autostart::set_registry_enabled(app, next)
            };
            if result.is_ok() {
                let snapshot = app
                    .state::<SettingsState>()
                    .update(|s| s.auto_start_enabled = next);
                let _ = crate::settings::persist(app, &snapshot);
                update(app);
            }
        }
        QUIT_ID => quit(app),
        _ => {}
    }
}

/// 激活储存的隐藏卓越性能计划；失败说明计划已被删除，清空 UUID（对齐旧版 TrayCoordinator）。
fn activate_hidden_ultimate(app: &AppHandle) {
    let saved = app
        .state::<SettingsState>()
        .0
        .lock()
        .unwrap()
        .ultimate_performance_plan_guid
        .clone();
    let Some(saved) = saved.filter(|value| !value.trim().is_empty()) else {
        return;
    };
    let activated = uuid::Uuid::parse_str(saved.trim())
        .is_ok_and(|guid| power::set_active_scheme(guid).is_ok());
    if !activated {
        let snapshot = app
            .state::<SettingsState>()
            .update(|s| s.ultimate_performance_plan_guid = None);
        let _ = crate::settings::persist(app, &snapshot);
    }
    power::invalidate_plans_cache();
    update(app);
    let _ = app.emit("plans-changed", ());
}

/// 退出：先移除托盘与菜单资源，再退出应用。
fn quit(app: &AppHandle) {
    remove(app);
    app.exit(0);
}
