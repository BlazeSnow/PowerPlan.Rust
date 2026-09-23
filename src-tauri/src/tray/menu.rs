//! 托盘菜单构建：结构对齐旧版 TrayMenuBuilder，每次由当前快照生成。

use tauri::menu::{CheckMenuItem, Menu, MenuBuilder, MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Manager, Wry};

use super::{language, product_name, HIDDEN_ULTIMATE_ID, OPEN_ID, PLAN_PREFIX, QUIT_ID, REFRESH_ID, SETTINGS_ID, TITLE_ID};
use crate::core::power;
use crate::settings::SettingsState;

// 仅电源计划项以 ⚡ 字形前缀标识类目，单色渲染随菜单深浅色自适应；
// 其余项纯文本——文本自明，字形图标跨字体渲染不一致且无信息增益
const PLAN_ICON: &str = "\u{26A1} "; // ⚡

/// 菜单结构对齐旧版 TrayMenuBuilder：禁用标题、打开主窗口、计划列表、
/// 隐藏的卓越性能（条件显示）、刷新计划、打开软件设置、退出。
/// 不设自启动开关：避免每次重建菜单都做系统状态查询（打包版为
/// WinRT StartupTask 调用，是菜单构建最重的一环），切换在设置页进行。
pub(super) fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let lang = language(app);
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
        lang.message("tray-menu-open-main-window"),
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
        lang.message("tray-menu-refresh-plans"),
        true,
        None::<&str>,
    )?;
    builder = builder.item(&refresh);

    // 打开软件设置：显示主窗口并导航到设置页（自启动等开关的唯一切换入口）
    let settings = MenuItem::with_id(
        app,
        SETTINGS_ID,
        lang.message("tray-menu-open-settings"),
        true,
        None::<&str>,
    )?;
    builder = builder.item(&settings);
    builder = builder.item(&PredefinedMenuItem::separator(app)?);
    let quit = MenuItem::with_id(
        app,
        QUIT_ID,
        lang.message("tray-menu-exit"),
        true,
        None::<&str>,
    )?;
    builder = builder.item(&quit);
    builder.build()
}
