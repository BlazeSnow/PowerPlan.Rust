//! 托盘菜单深浅色兼容层（对齐旧版 TrayMenuTheme）：Windows 原生菜单默认
//! 不随应用深色模式，需通过 uxtheme.dll 未公开序号函数开启应用级深色策略；
//! 动态重建菜单后必须刷新主题缓存。禁止 ForceDark/ForceLight/硬编码菜单颜色。

use std::sync::{LazyLock, Mutex};

use windows::core::PCSTR;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

// uxtheme 序号函数：1903(18362) 起序号 135 为 SetPreferredAppMode，
// 1809(17763)–1902 为 AllowDarkModeForApp；104/136 为状态刷新与菜单缓存刷新
const ALLOW_DARK_MODE_FOR_APP_ORDINAL: u16 = 135;
const REFRESH_IMMERSIVE_COLOR_POLICY_STATE_ORDINAL: u16 = 104;
const FLUSH_MENU_THEMES_ORDINAL: u16 = 136;
const WIN10_BUILD_1809: u32 = 17763;
const WIN10_BUILD_1903: u32 = 18362;

// PreferredAppMode：Default=0（跟随系统默认策略）、AllowDark=1（允许深色，仍由系统决定）
const PREFERRED_APP_MODE_DEFAULT: u32 = 0;
const PREFERRED_APP_MODE_ALLOW_DARK: u32 = 1;

type SetPreferredAppModeFn = unsafe extern "system" fn(u32) -> u32;
type AllowDarkModeForAppFn = unsafe extern "system" fn(bool) -> bool;
type VoidFn = unsafe extern "system" fn();

#[link(name = "ntdll")]
unsafe extern "system" {
    fn RtlGetNtVersionNumbers(major: *mut u32, minor: *mut u32, build: *mut u32);
}

struct TrayTheme {
    set_preferred_app_mode: Option<SetPreferredAppModeFn>,
    refresh_immersive: Option<VoidFn>,
    flush_menu_themes: Option<VoidFn>,
    supported: bool,
}

static TRAY_THEME: LazyLock<Mutex<TrayTheme>> = LazyLock::new(|| {
    Mutex::new(TrayTheme {
        set_preferred_app_mode: None,
        refresh_immersive: None,
        flush_menu_themes: None,
        supported: false,
    })
});

fn windows_build_number() -> u32 {
    unsafe {
        let (mut major, mut minor, mut build) = (0u32, 0u32, 0u32);
        RtlGetNtVersionNumbers(&mut major, &mut minor, &mut build);
        if major == 10 {
            build & 0x0FFF_FFFF
        } else {
            0
        }
    }
}

fn get_proc<T>(module: HMODULE, ordinal: u16) -> Option<T> {
    unsafe {
        GetProcAddress(module, PCSTR(ordinal as *const u8))
            .map(|address| std::mem::transmute_copy(&address))
    }
}

fn refresh_locked(theme: &TrayTheme) {
    unsafe {
        if let Some(refresh) = theme.refresh_immersive {
            refresh();
        }
        if let Some(flush) = theme.flush_menu_themes {
            flush();
        }
    }
}

/// 应用启动时调用（早于首个托盘菜单创建）：开启应用级深色策略。
/// 不支持的系统或 API 解析失败时保持默认策略回退。
pub fn init() {
    let mut theme = TRAY_THEME.lock().unwrap();
    if theme.supported {
        return;
    }
    let build = windows_build_number();
    if build < WIN10_BUILD_1809 {
        return;
    }
    let module = unsafe { LoadLibraryW(windows::core::w!("uxtheme.dll")) };
    let Ok(module) = module else {
        return;
    };

    if build < WIN10_BUILD_1903 {
        theme.set_preferred_app_mode = None;
        let allow_dark: Option<AllowDarkModeForAppFn> =
            get_proc(module, ALLOW_DARK_MODE_FOR_APP_ORDINAL);
        match allow_dark {
            Some(allow_dark) => unsafe {
                allow_dark(true);
            },
            None => return,
        }
    } else {
        let set_preferred: Option<SetPreferredAppModeFn> =
            get_proc(module, ALLOW_DARK_MODE_FOR_APP_ORDINAL);
        match set_preferred {
            Some(set_preferred) => unsafe {
                set_preferred(PREFERRED_APP_MODE_ALLOW_DARK);
            },
            None => return,
        }
        theme.set_preferred_app_mode = Some(set_preferred.expect("checked above"));
    }

    theme.refresh_immersive = get_proc(module, REFRESH_IMMERSIVE_COLOR_POLICY_STATE_ORDINAL);
    theme.flush_menu_themes = get_proc(module, FLUSH_MENU_THEMES_ORDINAL);
    if theme.refresh_immersive.is_none() || theme.flush_menu_themes.is_none() {
        // 回退默认策略
        if let Some(set_preferred) = theme.set_preferred_app_mode {
            unsafe {
                set_preferred(PREFERRED_APP_MODE_DEFAULT);
            }
        }
        theme.set_preferred_app_mode = None;
        theme.refresh_immersive = None;
        theme.flush_menu_themes = None;
        return;
    }

    theme.supported = true;
    refresh_locked(&theme);
}

/// 动态重建托盘菜单后调用：刷新沉浸式颜色策略与菜单主题缓存。
pub fn refresh() {
    let theme = TRAY_THEME.lock().unwrap();
    if !theme.supported {
        return;
    }
    refresh_locked(&theme);
}
