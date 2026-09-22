//! 电源计划命令：列表、切换、复制、创建卓越性能、恢复默认、打开电源选项。
//! 行为对齐旧版 PowerPlanService：所有写操作后失效计划缓存并重建托盘菜单。

use tauri::{AppHandle, Manager};

use crate::commands::CommandError;
use crate::core::power::{self, PlanError, PlanInfo, Win32Error};
use crate::settings::{self, SettingsState};

#[tauri::command]
pub fn power_list_plans(force: Option<bool>) -> Result<Vec<PlanInfo>, CommandError> {
    power::list_plans_cached(force.unwrap_or(false)).map_err(win32("PowerPlan.Error.EnumerateFailed"))
}

#[tauri::command]
pub fn power_set_active(app: AppHandle, guid: String) -> Result<(), CommandError> {
    let guid = parse_guid(&guid, "PowerPlan.Error.InvalidPlanGuid")?;
    power::set_active_scheme(guid).map_err(win32("PowerPlan.Error.SetActiveFailed"))?;
    power::invalidate_plans_cache();
    crate::tray::update(&app);
    Ok(())
}

#[tauri::command]
pub fn power_copy_plan(
    app: AppHandle,
    source_guid: String,
    new_name: String,
) -> Result<String, CommandError> {
    let source = parse_guid(&source_guid, "PowerPlan.Error.InvalidSourcePlanGuid")?;
    // 名称校验先于创建副本，避免留下无名副本（旧版 changelog 修复项）
    let name = new_name.trim();
    if name.is_empty() {
        return Err(CommandError::new("PowerPlan.Error.EmptyName", &[]));
    }
    let guid = power::copy_plan(source, name).map_err(|error| match error {
        PlanError::Win32(e) => win32("PowerPlan.Error.DuplicateFailed")(e),
        PlanError::MissingDuplicateGuid => missing_guid(),
    })?;
    power::invalidate_plans_cache();
    crate::tray::update(&app);
    Ok(guid.to_string())
}

#[tauri::command]
pub fn power_duplicate_ultimate(app: AppHandle) -> Result<String, CommandError> {
    let guid = power::duplicate_ultimate_performance()
        .map_err(|error| match error {
            PlanError::Win32(e) => win32("PowerPlan.Error.DuplicateFailed")(e),
            PlanError::MissingDuplicateGuid => missing_guid(),
        })?
        .to_string();
    let snapshot = app.state::<SettingsState>().update(|s| {
        s.ultimate_performance_plan_guid = Some(guid.clone());
    });
    persist(&app, &snapshot)?;
    power::invalidate_plans_cache();
    crate::tray::update(&app);
    Ok(guid)
}

/// 激活储存的隐藏卓越性能计划失败后清空 UUID（旧版 MainPage 行为）。
#[tauri::command]
pub fn power_clear_saved_ultimate(app: AppHandle) -> Result<(), CommandError> {
    let snapshot = app
        .state::<SettingsState>()
        .update(|s| s.ultimate_performance_plan_guid = None);
    persist(&app, &snapshot)?;
    crate::tray::update(&app);
    Ok(())
}

#[tauri::command]
pub fn power_restore_defaults(app: AppHandle) -> Result<(), CommandError> {
    power::restore_default_schemes()
        .map_err(win32("PowerPlan.Error.RestoreDefaultsFailed"))?;
    // 恢复默认后清空储存的卓越性能 UUID（references/power-plans.md「卓越性能计划存在性」）
    let snapshot = app
        .state::<SettingsState>()
        .update(|s| s.ultimate_performance_plan_guid = None);
    persist(&app, &snapshot)?;
    power::invalidate_plans_cache();
    crate::tray::update(&app);
    Ok(())
}

#[tauri::command]
pub fn power_open_power_options() -> Result<(), CommandError> {
    #[cfg(windows)]
    {
        std::process::Command::new("control")
            .args(["/name", "Microsoft.PowerOptions"])
            .spawn()
            .map_err(|e| {
                CommandError::new("Main.Status.SwitchFailed", &[("0", &e.to_string())])
            })?;
    }
    Ok(())
}

fn parse_guid(value: &str, error_key: &str) -> Result<uuid::Uuid, CommandError> {
    uuid::Uuid::parse_str(value.trim())
        .map_err(|_| CommandError::new(error_key, &[]))
}

/// Win32 错误的两级格式（对齐旧版 LocalizedPowerPlanErrorFormatter）：
/// 外层键 `PowerPlan.Error.Win32`，args 携带具体错误键与错误码，由前端渲染。
fn win32(
    label: &'static str,
) -> impl Fn(Win32Error) -> CommandError {
    move |error| {
        CommandError::new(
            "PowerPlan.Error.Win32",
            &[("label", label), ("code", &error.0.to_string())],
        )
    }
}

fn missing_guid() -> CommandError {
    CommandError::new("PowerPlan.Error.DuplicateMissingGuid", &[])
}

fn persist(app: &AppHandle, snapshot: &crate::settings::Settings) -> Result<(), CommandError> {
    settings::persist(app, snapshot)
        .map_err(|e| CommandError::new("Settings.SaveFailed", &[("0", &e)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_guid_accepts_valid_and_trims_whitespace() {
        let guid = parse_guid(" e9a42b02-d5df-448d-aa00-03f14749eb61 ", "PowerPlan.Error.InvalidPlanGuid")
            .expect("valid guid with whitespace");
        assert_eq!(
            guid.to_string(),
            "e9a42b02-d5df-448d-aa00-03f14749eb61"
        );
    }

    #[test]
    fn parse_guid_accepts_uppercase() {
        // 托盘/前端可能传大写形式
        let guid = parse_guid("E9A42B02-D5DF-448D-AA00-03F14749EB61", "PowerPlan.Error.InvalidPlanGuid")
            .expect("uppercase guid");
        assert_eq!(guid.to_string(), "e9a42b02-d5df-448d-aa00-03f14749eb61");
    }

    #[test]
    fn parse_guid_rejects_invalid_with_given_key() {
        let error = parse_guid("not-a-guid", "PowerPlan.Error.InvalidSourcePlanGuid")
            .expect_err("invalid guid");
        assert_eq!(error.key, "PowerPlan.Error.InvalidSourcePlanGuid");
    }

    #[test]
    fn win32_error_wraps_label_and_code_for_frontend() {
        // 两级错误格式：外层包装键 + 具体错误键 + Win32 代码
        let error = win32("PowerPlan.Error.SetActiveFailed")(Win32Error(5));
        assert_eq!(error.key, "PowerPlan.Error.Win32");
        assert_eq!(
            error.args.get("label").map(String::as_str),
            Some("PowerPlan.Error.SetActiveFailed")
        );
        assert_eq!(error.args.get("code").map(String::as_str), Some("5"));
    }
}
