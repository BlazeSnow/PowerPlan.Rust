//! 托盘：Tauri 2 内置 tray-icon，菜单每次由当前快照重建（references/tray.md）。
//! 菜单文案经后端 Fluent（crate::i18n）本地化，跟随设置语言即时刷新。

use tauri::menu::{CheckMenuItem, Menu, MenuBuilder, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::core::power;
use crate::i18n::Lang;
use crate::settings::SettingsState;

pub const TRAY_ID: &str = "powerplan-tray";
const OPEN_ID: &str = "open-main-window";
const AUTOSTART_ID: &str = "autostart-toggle";
const QUIT_ID: &str = "quit";
const PLAN_PREFIX: &str = "plan-";

/// 创建托盘（仅托盘启用时调用）。
pub fn create(app: &AppHandle) -> Result<(), String> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "missing window icon".to_string())?;
    let menu = build_menu(app).map_err(|e| e.to_string())?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .build(app)
        .map_err(|e| e.to_string())?;
    update_tooltip(app);
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
        if let Ok(menu) = build_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
        update_tooltip(app);
    }
}

/// 移除托盘（关闭托盘开关 / 退出）。
pub fn remove(app: &AppHandle) {
    let _ = app.remove_tray_by_id(TRAY_ID);
}

/// 显示并聚焦主窗口，并通知前端刷新计划状态。
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        let _ = app.emit("plans-changed", ());
    }
}

/// 托盘菜单事件（setup 时经 app.on_menu_event 注册）。
pub fn on_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    if let Some(guid) = id.strip_prefix(PLAN_PREFIX) {
        if let Ok(guid) = uuid::Uuid::parse_str(guid) {
            if power::set_active_scheme(guid).is_ok() {
                update(app);
            }
        }
        return;
    }
    match id {
        OPEN_ID => show_main_window(app),
        AUTOSTART_ID => {
            let next = !app
                .state::<SettingsState>()
                .0
                .lock()
                .unwrap()
                .auto_start_enabled;
            if crate::settings::apply_auto_start(app, next).is_ok() {
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

/// 退出：先移除托盘与菜单资源，再退出应用。
fn quit(app: &AppHandle) {
    remove(app);
    app.exit(0);
}

fn language(app: &AppHandle) -> Lang {
    Lang::from_config(&app.state::<SettingsState>().0.lock().unwrap().language)
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let lang = language(app);
    let auto_start_enabled = app
        .state::<SettingsState>()
        .0
        .lock()
        .unwrap()
        .auto_start_enabled;

    let mut builder = MenuBuilder::new(app);
    match power::list_plans() {
        Ok(plans) if !plans.is_empty() => {
            for plan in plans {
                // 空名计划（如节能模式）回退到本地化默认名称
                let name = if plan.name.is_empty() {
                    lang.message("tray-plan-default")
                } else {
                    plan.name
                };
                let item = CheckMenuItem::with_id(
                    app,
                    format!("{PLAN_PREFIX}{}", plan.guid),
                    name,
                    true,
                    plan.is_active,
                    None::<&str>,
                )?;
                builder = builder.item(&item);
            }
        }
        _ => {
            // 计划不可读时显示禁用项，避免空菜单
            let item = MenuItem::with_id(
                app,
                "plans-unavailable",
                lang.message("tray-tooltip-plan-unavailable"),
                false,
                None::<&str>,
            )?;
            builder = builder.item(&item);
        }
    }
    builder = builder.item(&PredefinedMenuItem::separator(app)?);

    let autostart = CheckMenuItem::with_id(
        app,
        AUTOSTART_ID,
        lang.message(if auto_start_enabled {
            "tray-menu-disable-autostart"
        } else {
            "tray-menu-enable-autostart"
        }),
        true,
        auto_start_enabled,
        None::<&str>,
    )?;
    builder = builder.item(&autostart);

    let open = MenuItem::with_id(
        app,
        OPEN_ID,
        lang.message("tray-menu-open-main-window"),
        true,
        None::<&str>,
    )?;
    builder = builder.item(&open);
    builder = builder.item(&PredefinedMenuItem::separator(app)?);
    let quit = MenuItem::with_id(app, QUIT_ID, lang.message("tray-menu-exit"), true, None::<&str>)?;
    builder = builder.item(&quit);
    builder.build()
}

fn update_tooltip(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let lang = language(app);
    let text = match power::active_scheme() {
        Ok(guid) => match power::friendly_name(guid) {
            Ok(name) if !name.is_empty() => {
                lang.message_with("tray-tooltip-plan", &[("plan", &name)])
            }
            _ => lang.message("tray-plan-default"),
        },
        Err(_) => lang.message("tray-tooltip-plan-unavailable"),
    };
    let _ = tray.set_tooltip(Some(text));
}
