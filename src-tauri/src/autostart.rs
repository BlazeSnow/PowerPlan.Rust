//! 开机自启动：双模式适配（references/settings.md「开机自启动」）。
//! - MSIX 打包：`StartupTask`（uap5 清单声明，TaskId=`PowerPlanStartupTask`，
//!   对齐旧版 StartupService）；注册表 Run 在 MSIX 下被虚拟化，不可用。
//! - 未打包（开发构建）：注册表 HKCU Run（tauri-plugin-autostart，附静默参数）。
//! 设置与托盘共用 [`set_enabled`]；[`state`] 返回系统侧实际状态供前端显示。

use serde::Serialize;
use tauri::AppHandle;
use windows::core::HSTRING;
use windows::Win32::Foundation::{ERROR_SUCCESS, APPMODEL_ERROR_NO_PACKAGE};
use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
use windows::Win32::System::Registry::{
    RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ,
};

/// StartupTask 标识（须与 msix/AppxManifest.template.xml 的声明一致）。
pub const STARTUP_TASK_ID: &str = "PowerPlanStartupTask";

/// 系统侧实际状态（区别于设置中的期望开关；前端按此显示）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoStartState {
    Enabled,
    Disabled,
    DisabledByUser,
    DisabledByPolicy,
    Unsupported,
}

impl AutoStartState {
    pub fn as_str(&self) -> &'static str {
        match self {
            AutoStartState::Enabled => "enabled",
            AutoStartState::Disabled => "disabled",
            AutoStartState::DisabledByUser => "disabled_by_user",
            AutoStartState::DisabledByPolicy => "disabled_by_policy",
            AutoStartState::Unsupported => "unsupported",
        }
    }
}

/// 是否以 MSIX 打包身份运行。
pub fn is_packaged() -> bool {
    let mut length = 0u32;
    let error = unsafe { GetCurrentPackageFullName(&mut length, None) };
    error != APPMODEL_ERROR_NO_PACKAGE
}

/// 系统侧实际状态。
pub fn state(_app: &AppHandle) -> AutoStartState {
    if is_packaged() {
        state_startup_task()
    } else {
        if registry_enabled() {
            AutoStartState::Enabled
        } else {
            AutoStartState::Disabled
        }
    }
}

/// 应用/取消开机自启动。打包版启用请求可能被系统（用户已在启动设置中
/// 禁用过）拒绝，此时返回携带原因的错误文案键。
pub fn set_enabled(value: bool) -> Result<(), String> {
    if is_packaged() {
        set_startup_task(value)
    } else {
        // 未打包路径由调用方经 tauri-plugin-autostart 处理（需要 AppHandle）
        Err("unsupported".into())
    }
}

/// 经 tauri-plugin-autostart 应用注册表（未打包模式）。
pub fn set_registry_enabled(app: &AppHandle, value: bool) -> Result<(), String> {
    crate::settings::apply_auto_start(app, value)
}

/// 本次启动是否由登录时的 StartupTask 激活（打包版静默启动依据）。
/// 未打包时 WinRT 调用失败，返回 false（开发构建用 --silent 参数模拟）。
pub fn is_startup_task_launch() -> bool {
    use windows::ApplicationModel::Activation::ActivationKind;
    use windows::ApplicationModel::AppInstance;

    if !is_packaged() {
        return false;
    }
    // GetActivatedEventArgs 仅打包环境可用；Kind 返回登录激活类型
    AppInstance::GetActivatedEventArgs()
        .ok()
        .and_then(|args| args.Kind().ok())
        .is_some_and(|kind| kind == ActivationKind::StartupTask)
}

/// 等待 WinRT 异步操作完成（同步命令上下文；IAsyncOperation 实现 IntoFuture）。
fn wait<T: windows::core::RuntimeType>(
    operation: windows_future::IAsyncOperation<T>,
) -> windows::core::Result<T> {
    use std::future::IntoFuture as _;
    tauri::async_runtime::block_on(operation.into_future())
}

fn state_startup_task() -> AutoStartState {
    use windows::ApplicationModel::StartupTask;
    use windows::ApplicationModel::StartupTaskState;

    let task = StartupTask::GetAsync(&HSTRING::from(STARTUP_TASK_ID)).and_then(wait);
    let Ok(task) = task else {
        return AutoStartState::Unsupported;
    };
    match task.State() {
        Ok(StartupTaskState::Enabled) | Ok(StartupTaskState::EnabledByPolicy) => {
            AutoStartState::Enabled
        }
        Ok(StartupTaskState::DisabledByUser) => AutoStartState::DisabledByUser,
        Ok(StartupTaskState::DisabledByPolicy) => AutoStartState::DisabledByPolicy,
        _ => AutoStartState::Disabled,
    }
}

fn set_startup_task(value: bool) -> Result<(), String> {
    use windows::ApplicationModel::StartupTask;
    use windows::ApplicationModel::StartupTaskState;

    let task = StartupTask::GetAsync(&HSTRING::from(STARTUP_TASK_ID))
        .and_then(wait)
        .map_err(|e| e.to_string())?;
    if value {
        let state = task.State().map_err(|e| e.to_string())?;
        if matches!(state, StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy) {
            return Ok(());
        }
        let result = task
            .RequestEnableAsync()
            .and_then(wait)
            .map_err(|e| e.to_string())?;
        if matches!(result, StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy) {
            Ok(())
        } else {
            // 系统拒绝（常见：用户曾在任务管理器/启动设置中禁用）
            Err("disabled_by_user".into())
        }
    } else {
        task.Disable().map_err(|e| e.to_string())
    }
}

/// 未打包模式：读注册表 Run 项的实际状态（值名 = 产品名，auto-launch 写入）。
fn registry_enabled() -> bool {
    static RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let value_name = HSTRING::from("PowerPlan");
    let mut buffer = [0u16; 512];
    let mut size = u32::try_from(buffer.len() * std::mem::size_of::<u16>()).unwrap_or(u32::MAX);
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            &HSTRING::from(RUN_KEY),
            &value_name,
            RRF_RT_REG_SZ,
            None,
            Some(buffer.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    result == ERROR_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_serializes_snake_case() {
        // 前端状态键契约：snake_case
        assert_eq!(
            serde_json::to_value(AutoStartState::DisabledByUser).unwrap(),
            "disabled_by_user"
        );
        assert_eq!(
            serde_json::to_value(AutoStartState::Enabled).unwrap(),
            "enabled"
        );
    }

    #[test]
    fn as_str_matches_serde() {
        for state in [
            AutoStartState::Enabled,
            AutoStartState::Disabled,
            AutoStartState::DisabledByUser,
            AutoStartState::DisabledByPolicy,
            AutoStartState::Unsupported,
        ] {
            assert_eq!(
                serde_json::to_value(state).unwrap(),
                serde_json::Value::String(state.as_str().into())
            );
        }
    }

    #[test]
    fn unpackaged_dev_build_is_not_startup_task_launch() {
        // 开发构建未打包：登录激活检测必须返回 false（否则静默逻辑误触发）
        assert!(!is_startup_task_launch());
    }

    #[test]
    fn startup_task_id_matches_manifest() {
        // 与 msix/AppxManifest.template.xml 的 TaskId 保持一致
        assert_eq!(STARTUP_TASK_ID, "PowerPlanStartupTask");
    }
}
