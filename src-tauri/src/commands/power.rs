//! 电源计划命令：列表、切换、创建卓越性能、恢复默认、打开电源选项。

use tauri::{AppHandle, Manager};

use crate::commands::CommandError;
use crate::core::power::{self, PlanInfo, Win32Error};
use crate::settings::{self, SettingsState};

#[tauri::command]
pub fn power_list_plans() -> Result<Vec<PlanInfo>, CommandError> {
    power::list_plans().map_err(win32("PowerPlan.Error.EnumerateFailed"))
}

#[tauri::command]
pub fn power_set_active(guid: String) -> Result<(), CommandError> {
    let guid = parse_guid(&guid)?;
    power::set_active_scheme(guid).map_err(win32("PowerPlan.Error.SetActiveFailed"))
}

#[tauri::command]
pub fn power_duplicate_ultimate(app: AppHandle) -> Result<String, CommandError> {
    let guid = power::duplicate_ultimate_performance()
        .map_err(win32("PowerPlan.Error.DuplicateFailed"))?
        .to_string();
    let snapshot = app.state::<SettingsState>().update(|s| {
        s.ultimate_performance_plan_guid = Some(guid.clone());
    });
    persist(&app, &snapshot)?;
    crate::tray::update(&app);
    Ok(guid)
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

fn parse_guid(value: &str) -> Result<uuid::Uuid, CommandError> {
    uuid::Uuid::parse_str(value.trim())
        .map_err(|_| CommandError::new("PowerPlan.Error.InvalidPlanGuid", &[]))
}

fn win32(
    key: &'static str,
) -> impl Fn(Win32Error) -> CommandError {
    move |error| CommandError::new(key, &[("0", &error.0.to_string())])
}

fn persist(app: &AppHandle, snapshot: &crate::settings::Settings) -> Result<(), CommandError> {
    settings::persist(app, snapshot)
        .map_err(|e| CommandError::new("Settings.SaveFailed", &[("0", &e)]))
}
