//! Windows 效能模式（EcoQoS / Power Throttling，references/conventions.md「性能要求」）：
//! 主窗口不可见（托盘常驻）时对进程启用执行速度节流——任务管理器显示效能模式
//! 叶片标志，调度器倾向 E-core 并限制频率；主窗口打开（前台交互）时恢复全性能。
//! 调用点：静默/启动到托盘、关闭主窗口（→启用），显示主窗口（→关闭）。

use windows::Win32::System::Threading::{
    GetCurrentProcess, SetProcessInformation,
    PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
    PROCESS_POWER_THROTTLING_STATE, ProcessPowerThrottling,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggling_is_supported_and_repeatable() {
        // 开发机/CI 为现代 Windows：EcoQoS API 可用；重复开关不产生副作用
        assert!(set_enabled(true));
        assert!(set_enabled(false));
        assert!(set_enabled(true));
        assert!(set_enabled(false));
    }
}
