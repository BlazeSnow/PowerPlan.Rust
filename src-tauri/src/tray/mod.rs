//! 托盘：Tauri 2 内置 tray-icon，菜单每次由当前快照重建（references/tray.md）。
//! 菜单文案经后端 Fluent（crate::i18n）本地化，跟随设置语言即时刷新。
//! 菜单构建见 [`menu`]，提示文本见 [`tooltip`]。

mod menu;
mod tooltip;

use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};

use std::sync::atomic::{AtomicBool, Ordering};

use crate::core::power;
use crate::i18n::Lang;
use crate::settings::SettingsState;

pub const TRAY_ID: &str = "powerplan-tray";
pub(super) const TITLE_ID: &str = "menu-title";
pub(super) const OPEN_ID: &str = "open-main-window";
pub(super) const HIDDEN_ULTIMATE_ID: &str = "activate-hidden-ultimate";
pub(super) const REFRESH_ID: &str = "refresh-plans";
pub(super) const SETTINGS_ID: &str = "open-settings";
pub(super) const QUIT_ID: &str = "quit";
pub(super) const PLAN_PREFIX: &str = "plan-";

/// 托盘「打开设置」挂起标记：webview 销毁重建场景下事件早于前端
/// 监听注册，由前端挂载时消费兜底；事件路径同样消费，防陈旧标记误导航。
static OPEN_SETTINGS_REQUESTED: AtomicBool = AtomicBool::new(false);

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
    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        // 自动弹出的是预构建菜单快照，外部改动（任务管理器切换自启动等）
        // 会显示延迟状态：改为点击时重建菜单后手动弹出（见 on_tray_icon_event）
        .show_menu_on_left_click(false)
        .on_tray_icon_event(on_tray_icon_event)
        .build(app)
        .map_err(|e| e.to_string())?;
    // 右键自动弹出 tauri 未暴露，经内层 tray-icon 关闭（逃生舱口依赖 tauri
    // 2.11 的内部结构，升级 tauri 后需回归验证托盘菜单）
    let _ = tray.with_inner_tray_icon(|inner| inner.set_show_menu_on_right_click(false));
    tooltip::update(app);
    // 菜单创建后刷新深浅色主题缓存（对齐旧版动态菜单刷新）
    crate::tray_theme::refresh();
    Ok(())
}

/// 托盘图标点击（左/右键抬起）：先重建菜单与提示——自启动等状态以系统侧
/// 实际状态为准——再手动弹出，保证菜单内容是打开瞬间的最新值。
/// 事件经事件循环异步分发，此时自动弹出已全部关闭，不存在双菜单竞争；
/// 回调运行在主线程，with_inner_tray_icon 内联执行，无跨线程等待。
fn on_tray_icon_event(tray: &TrayIcon<Wry>, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left | MouseButton::Right,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        let app = tray.app_handle();
        update(app);
        let _ = tray.with_inner_tray_icon(|inner| inner.show_menu());
    }
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
        SETTINGS_ID => open_settings(app),
        QUIT_ID => quit(app),
        _ => {}
    }
}

/// 打开设置：显示主窗口并导航到设置页。
///
/// 事件对已存活的 webview 即时可达；销毁重建场景下事件早于前端监听注册，
/// 由挂起标记 + 前端挂载时消费兜底。两条路径都会消费标记。
pub fn open_settings(app: &AppHandle) {
    OPEN_SETTINGS_REQUESTED.store(true, Ordering::SeqCst);
    show_main_window(app);
    let _ = app.emit("open-settings", ());
}

/// 前端消费挂起标记（命令 `take_open_settings_request`）：取值并清零。
pub fn take_open_settings_request() -> bool {
    OPEN_SETTINGS_REQUESTED.swap(false, Ordering::SeqCst)
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
