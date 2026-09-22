//! Windows 效能模式（EcoQoS / Power Throttling，references/conventions.md「性能要求」）：
//! 主窗口不可见（托盘常驻）时对进程启用执行速度节流——任务管理器显示效能模式
//! 叶片标志，调度器倾向 E-core 并限制频率；主窗口打开（前台交互）时恢复全性能。
//! 调用点：静默/启动到托盘、关闭主窗口（→启用），显示主窗口（→关闭）。
//!
//! 注意：插电电源模式为"最佳性能"（overlay ded574b5）时，系统整体禁用电源
//! 节流——本 API 仍返回成功，但效能模式（叶子图标）不生效，属系统行为。

use std::sync::OnceLock;

use tauri::{AppHandle, Manager};
use windows::core::GUID;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Power::{
    PowerSettingRegisterNotification, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, SetProcessInformation, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
    ProcessPowerThrottling,
};
use windows::Win32::UI::WindowsAndMessaging::DEVICE_NOTIFY_CALLBACK;

/// GUID_POWERSCHEME_PERSONALITY：电源模式（含 overlay 滑块）变化通知。
/// crate 未导出，按 SDK 头文件定义。
const GUID_POWERSCHEME_PERSONALITY: GUID =
    GUID::from_u128(0x68a6e99b_5ee8_4aad_a9d6_529605fab6da);

static NOTIFY_APP: OnceLock<AppHandle> = OnceLock::new();

/// 应用/取消效能模式。旧版 Windows 不支持该 API 时返回 false 并静默忽略。
pub fn set_enabled(enable: bool) -> bool {
    let state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        StateMask: if enable {
            PROCESS_POWER_THROTTLING_EXECUTION_SPEED
        } else {
            0
        },
    };
    let result = unsafe {
        SetProcessInformation(
            GetCurrentProcess(),
            ProcessPowerThrottling,
            &state as *const PROCESS_POWER_THROTTLING_STATE as *const core::ffi::c_void,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
    };
    result.is_ok()
}

/// 电源模式变化（用户切换 overlay / 电源计划）：系统对 EcoQoS 的应用策略
/// 随之改变，但已记录的 opt-in 不会自动重放——收到通知后按当前窗口可见性
/// 重新应用，使效能模式无需重启应用即跟随电源模式生效。
unsafe extern "system" fn on_power_setting(
    _context: *const core::ffi::c_void,
    _type: u32,
    _setting: *const core::ffi::c_void,
) -> u32 {
    if let Some(app) = NOTIFY_APP.get() {
        let window_visible = app
            .get_webview_window("main")
            .and_then(|window| window.is_visible().ok())
            .unwrap_or(false);
        set_enabled(!window_visible);
    }
    0
}

/// 注册电源模式变化监听（应用启动时调用一次）。
pub fn watch_power_changes(app: &AppHandle) {
    let _ = NOTIFY_APP.set(app.clone());
    // Box::leak：参数块须在进程生命周期内存活（系统回调异步触发）
    let params = Box::leak(Box::new(DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
        Callback: Some(on_power_setting),
        Context: std::ptr::null_mut(),
    }));
    let mut registration = std::ptr::null_mut();
    let result = unsafe {
        PowerSettingRegisterNotification(
            &GUID_POWERSCHEME_PERSONALITY,
            DEVICE_NOTIFY_CALLBACK,
            HANDLE(params as *const DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS as *mut core::ffi::c_void),
            &mut registration,
        )
    };
    // 注册失败不影响功能：仅失去"切电源模式自动重放"能力
    let _ = result;
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetProcessInformation;

    /// 读回当前进程的节流状态；本机 Get 不受支持时返回 None。
    fn read_back() -> Option<(u32, u32)> {
        let mut state = PROCESS_POWER_THROTTLING_STATE {
            // Get 要求 Version 预填当前版本，否则返回参数错误
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: 0,
            StateMask: 0,
        };
        let result = unsafe {
            GetProcessInformation(
                GetCurrentProcess(),
                ProcessPowerThrottling,
                &mut state as *mut PROCESS_POWER_THROTTLING_STATE as *mut core::ffi::c_void,
                std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
            )
        };
        result.ok().map(|_| (state.ControlMask, state.StateMask))
    }

    #[test]
    fn toggling_is_supported_and_repeatable() {
        // 开发机/CI 为现代 Windows：EcoQoS API 可用；重复开关不产生副作用
        assert!(set_enabled(true));
        assert!(set_enabled(false));
        assert!(set_enabled(true));
        assert!(set_enabled(false));
    }

    #[test]
    fn enable_is_visible_to_system_query() {
        // 系统侧读回验证：置位后 ControlMask/StateMask 均为 EXECUTION_SPEED。
        // 部分 Windows 构建不支持 Get 读取（返回 None），跳过断言
        if read_back().is_none() {
            return;
        }
        assert!(set_enabled(true));
        assert_eq!(read_back(), Some((1, 1)));
        assert!(set_enabled(false));
        assert_eq!(read_back(), Some((1, 0)));
    }
}
