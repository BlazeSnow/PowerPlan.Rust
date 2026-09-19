//! 托盘：Tauri 2 内置 tray-icon，菜单每次由当前快照重建（references/tray.md）。
//! 菜单文案经后端 Fluent（crate::i18n）本地化，跟随设置语言即时刷新。

use tauri::menu::{CheckMenuItem, Menu, MenuBuilder, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::core::power;
use crate::i18n::Lang;
use crate::settings::SettingsState;

pub const TRAY_ID: &str = "powerplan-tray";
const TITLE_ID: &str = "menu-title";
const OPEN_ID: &str = "open-main-window";
const HIDDEN_ULTIMATE_ID: &str = "activate-hidden-ultimate";
const REFRESH_ID: &str = "refresh-plans";
const AUTOSTART_ID: &str = "autostart-toggle";
const QUIT_ID: &str = "quit";
const PLAN_PREFIX: &str = "plan-";

// 菜单图标以 Unicode 字形前缀拼入文本（对齐旧版 TrayMenuBuilder），
// 单色渲染随菜单深浅色自适应；标题项不加（旧版同）
const OPEN_ICON: &str = "\u{2302} "; // ⌂
const PLAN_ICON: &str = "\u{26A1} "; // ⚡
const REFRESH_ICON: &str = "\u{21BB} "; // ↻
const AUTOSTART_ICON: &str = "\u{23FB} "; // ⏻
const EXIT_ICON: &str = "\u{2715} "; // ✕

/// 应用标题（读自 tauri.conf.json 的 productName，不硬编码可见字符串）。
fn product_name(app: &AppHandle) -> String {
    app.config()
        .product_name
        .clone()
        .unwrap_or_else(|| "PowerPlan".into())
}

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
        if let Ok(menu) = build_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
        update_tooltip(app);
        crate::tray_theme::refresh();
    }
}

/// 移除托盘（关闭托盘开关 / 退出）。
pub fn remove(app: &AppHandle) {
    let _ = app.remove_tray_by_id(TRAY_ID);
}

/// 显示并聚焦主窗口：已销毁时按配置重建，并通知前端刷新计划状态。
pub fn show_main_window(app: &AppHandle) {
    crate::ensure_main_window(app);
    let _ = app.emit("plans-changed", ());
}

/// 托盘菜单事件（setup 时经 app.on_menu_event 注册）。
pub fn on_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    if let Some(guid) = id.strip_prefix(PLAN_PREFIX) {
        if let Ok(guid) = uuid::Uuid::parse_str(guid) {
            if power::set_active_scheme(guid).is_ok() {
                power::invalidate_plans_cache();
                update(app);
                // 主窗口可能正处于打开状态，通知其刷新计划状态
                let _ = app.emit("plans-changed", ());
            }
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

fn language(app: &AppHandle) -> Lang {
    Lang::from_config(&app.state::<SettingsState>().0.lock().unwrap().language)
}

/// 菜单结构对齐旧版 TrayMenuBuilder：禁用标题、打开主窗口、计划列表、
/// 隐藏的卓越性能（条件显示）、刷新计划、自启动开关、退出。
fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let lang = language(app);
    let auto_start_enabled = app
        .state::<SettingsState>()
        .0
        .lock()
        .unwrap()
        .auto_start_enabled;
    let saved_ultimate = app
        .state::<SettingsState>()
        .0
        .lock()
        .unwrap()
        .ultimate_performance_plan_guid
        .clone();

    let title = MenuItem::with_id(app, TITLE_ID, product_name(app), false, None::<&str>)?;
    let open = MenuItem::with_id(
        app,
        OPEN_ID,
        format!("{OPEN_ICON}{}", lang.message("tray-menu-open-main-window")),
        true,
        None::<&str>,
    )?;
    let mut builder = MenuBuilder::new(app)
        .item(&title)
        .item(&open)
        .item(&PredefinedMenuItem::separator(app)?);

    let plans = power::list_plans_cached(false);
    match &plans {
        Ok(list) if !list.is_empty() => {
            for plan in list {
                let item = CheckMenuItem::with_id(
                    app,
                    format!("{PLAN_PREFIX}{}", plan.guid),
                    format!("{PLAN_ICON}{}", plan.name),
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

    // 隐藏的卓越性能：仅当储存 UUID 存在且不在当前列表时显示
    let hidden_in_list = saved_ultimate.as_deref().is_some_and(|saved| {
        matches!(&plans, Ok(list) if list.iter().any(|plan| plan.guid.eq_ignore_ascii_case(saved.trim())))
    });
    if saved_ultimate.as_deref().is_some_and(|saved| !saved.trim().is_empty()) && !hidden_in_list {
        let item = MenuItem::with_id(
            app,
            HIDDEN_ULTIMATE_ID,
            format!("{PLAN_ICON}{}", lang.message("tray-menu-open-hidden-ultimate")),
            true,
            None::<&str>,
        )?;
        builder = builder.item(&item);
    }

    builder = builder.item(&PredefinedMenuItem::separator(app)?);

    let refresh = MenuItem::with_id(
        app,
        REFRESH_ID,
        format!("{REFRESH_ICON}{}", lang.message("tray-menu-refresh-plans")),
        true,
        None::<&str>,
    )?;
    builder = builder.item(&refresh);

    // 开机自启动不用勾选状态：文案本身区分"开启/关闭"，用户凭文本即可确认
    let autostart = MenuItem::with_id(
        app,
        AUTOSTART_ID,
        format!(
            "{AUTOSTART_ICON}{}",
            lang.message(if auto_start_enabled {
                "tray-menu-disable-autostart"
            } else {
                "tray-menu-enable-autostart"
            })
        ),
        true,
        None::<&str>,
    )?;
    builder = builder.item(&autostart);
    builder = builder.item(&PredefinedMenuItem::separator(app)?);
    let quit = MenuItem::with_id(
        app,
        QUIT_ID,
        format!("{EXIT_ICON}{}", lang.message("tray-menu-exit")),
        true,
        None::<&str>,
    )?;
    builder = builder.item(&quit);
    builder.build()
}

/// 提示文本三行式，对齐旧版 TrayTooltipFormatter：标题 / 当前计划 / 自启动状态。
fn update_tooltip(app: &AppHandle) {
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
