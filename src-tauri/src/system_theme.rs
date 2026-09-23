//! Windows 系统明暗主题监听。
//!
//! WebView2 的 prefers-color-scheme 不保证随系统主题实时更新，故由后端
//! 监听系统设置变更广播（WM_SETTINGCHANGE，深浅色切换为 ImmersiveColorSet）
//! 触发时核对注册表（AppsUseLightTheme），变化时设置窗口原生主题并 emit
//! `system-theme` 事件（"light"/"dark"），前端据此切换文档类。
//! 全程事件驱动：无广播时消息泵阻塞休眠，零轮询、零周期任务。

use std::sync::atomic::{AtomicI8, Ordering};
use std::sync::OnceLock;

use tauri::{AppHandle, Emitter, Manager};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, TranslateMessage, MSG, WINDOW_EX_STYLE, WM_DESTROY, WM_SETTINGCHANGE,
    WNDCLASSW, WS_OVERLAPPED,
};

const PERSONALIZE_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const APPS_USE_LIGHT: &str = "AppsUseLightTheme";

/// 监听窗口的 AppHandle（wndproc 仅在本模块线程接收消息）。
static APP: OnceLock<AppHandle> = OnceLock::new();
/// 最近一次已知的浅色标记：-1 未知（启动读取失败），0 深色，1 浅色。
static LAST_LIGHT: AtomicI8 = AtomicI8::new(-1);

/// 读取"应用使用浅色主题"开关；None = 读取失败。
fn apps_use_light_theme() -> Option<bool> {
    let mut value: u32 = 0;
    let mut size = u32::try_from(std::mem::size_of::<u32>()).ok()?;
    let key = windows::core::HSTRING::from(PERSONALIZE_KEY);
    let name = windows::core::HSTRING::from(APPS_USE_LIGHT);
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            &key,
            &name,
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as _),
            Some(&mut size),
        )
    };
    result.is_ok().then_some(value == 1)
}

fn theme_name(light: bool) -> &'static str {
    if light { "light" } else { "dark" }
}

/// 应用主题：设置窗口原生主题并通知前端（托盘常驻无窗口时仅通知，
/// 重建窗口后前端挂载时经 `system_theme` 命令拉取最新值）。
fn apply_theme(app: &AppHandle, light: bool) {
    let theme = if light {
        tauri::Theme::Light
    } else {
        tauri::Theme::Dark
    };
    for window in app.webview_windows().values() {
        let _ = window.set_theme(Some(theme));
    }
    let _ = app.emit("system-theme", theme_name(light));
}

/// 广播到来：核对注册表实际值，变化才应用（任何设置广播都触发，
/// 不区分 lParam——核对本身是廉价单次读，比字符串过滤更稳）。
unsafe extern "system" fn theme_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_SETTINGCHANGE {
        if let (Some(app), Some(light)) = (APP.get(), apps_use_light_theme()) {
            let current = light as i8;
            if LAST_LIGHT.swap(current, Ordering::Relaxed) != current {
                apply_theme(app, light);
            }
        }
    } else if msg == WM_DESTROY {
        unsafe { PostQuitMessage(0) };
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// 启动常驻监听：启动即广播一次（前端可能晚于事件注册，另有
/// `system_theme` 命令兜底查询），随后经隐藏顶层窗口接收
/// WM_SETTINGCHANGE 广播（HWND_MESSAGE 的 message-only 窗口收不到
/// 广播，必须是顶层）。窗口与消息泵存续至进程退出，与旧轮询线程同。
pub fn start_theme_watcher(app: AppHandle) {
    let _ = APP.set(app);
    tauri::async_runtime::spawn_blocking(|| unsafe {
        let Some(app) = APP.get() else { return };
        if let Some(light) = apps_use_light_theme() {
            LAST_LIGHT.store(light as i8, Ordering::Relaxed);
            let _ = app.emit("system-theme", theme_name(light));
        }

        let class = WNDCLASSW {
            lpfnWndProc: Some(theme_wndproc),
            hInstance: GetModuleHandleW(None).unwrap_or_default().into(),
            lpszClassName: w!("PowerPlanThemeWatcher"),
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return; // 注册失败仅失去主题跟随（与旧版轮询读取失败同级），不影响其余功能
        }
        let Ok(hwnd) = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class.lpszClassName,
            PCWSTR::null(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(class.hInstance),
            None,
        ) else {
            return;
        };
        if hwnd.is_invalid() {
            return;
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

/// 当前系统主题（"light"/"dark"，未知时回退 "light"）。
pub fn current_system_theme() -> &'static str {
    theme_name(apps_use_light_theme().unwrap_or(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_name_maps_light_flag() {
        assert_eq!(theme_name(true), "light");
        assert_eq!(theme_name(false), "dark");
    }

    #[test]
    fn current_theme_is_light_or_dark() {
        assert!(matches!(current_system_theme(), "light" | "dark"));
    }
}
