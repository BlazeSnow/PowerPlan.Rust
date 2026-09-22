//! 电源计划：通过 powrprof.dll 原生 API 读取与切换（references/power-plans.md）。
//! 行为对齐旧版 PowerPlan.Core 的 PowerPlanService（枚举缓冲、空名回退、缓存与失效时机）。
//! 全程普通用户权限；PowerRestoreDefaultPowerSchemes 需要管理员，权限不足时返回 Win32 错误。

mod cache;
mod guid;

use guid::{guid_from_bytes, guid_to_windows};

pub use cache::{invalidate_plans_cache, list_plans_cached};

use serde::Serialize;
use uuid::Uuid;
use windows::core::GUID;
use windows::Win32::Foundation::{
    LocalFree, ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, ERROR_NO_UNICODE_TRANSLATION, ERROR_SUCCESS,
    WIN32_ERROR,
};
use windows::Win32::System::Power::{
    PowerDuplicateScheme, PowerEnumerate, PowerGetActiveScheme, PowerReadFriendlyName,
    PowerRestoreDefaultPowerSchemes, PowerSetActiveScheme, PowerWriteFriendlyName, ACCESS_SCHEME,
};

/// 系统卓越性能模板 GUID（references/power-plans.md「创建卓越性能计划」）。
pub const ULTIMATE_PERFORMANCE_TEMPLATE: Uuid =
    uuid::uuid!("e9a42b02-d5df-448d-aa00-03f14749eb61");

/// 电源计划信息（序列化给前端）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanInfo {
    pub guid: String,
    pub name: String,
    pub is_active: bool,
}

/// Win32 错误码。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Win32Error(pub u32);

impl std::fmt::Display for Win32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Win32 error {}", self.0)
    }
}

impl std::error::Error for Win32Error {}

/// 业务核心错误：Win32 错误码，或复制成功但系统未返回新 GUID。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanError {
    Win32(Win32Error),
    MissingDuplicateGuid,
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::Win32(e) => write!(f, "{e}"),
            PlanError::MissingDuplicateGuid => write!(f, "duplicate returned no guid"),
        }
    }
}

impl std::error::Error for PlanError {}

impl From<Win32Error> for PlanError {
    fn from(value: Win32Error) -> Self {
        PlanError::Win32(value)
    }
}

fn check(err: WIN32_ERROR) -> Result<(), Win32Error> {
    if err == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(Win32Error(err.0))
    }
}

/// 枚举用户可见电源计划的 GUID。
///
/// 对齐旧版实现：GUID 定长 16 字节，预分配缓冲单次调用即可取回，
/// 无需"空缓冲探大小"的两段式调用。
fn enumerate_scheme_guids() -> Result<Vec<Uuid>, Win32Error> {
    let mut guids = Vec::new();
    let mut index = 0u32;
    loop {
        let mut buffer = [0u8; 16];
        let mut size = buffer.len() as u32;
        let err = unsafe {
            PowerEnumerate(
                None,
                None,
                None,
                ACCESS_SCHEME,
                index,
                Some(buffer.as_mut_ptr()),
                &mut size,
            )
        };
        if err == ERROR_NO_MORE_ITEMS {
            break;
        }
        check(err)?;
        guids.push(guid_from_bytes(buffer));
        index += 1;
    }
    Ok(guids)
}

/// 读取计划友好名称（UTF-16 宽字符，不依赖系统 ANSI 代码页）。
pub fn friendly_name(guid: Uuid) -> Result<String, Win32Error> {
    let scheme = guid_to_windows(guid);
    let mut size = 0u32;
    let err =
        unsafe { PowerReadFriendlyName(None, Some(&scheme), None, None, None, &mut size) };
    // 实测首调（空缓冲）可能返回 SUCCESS（size=所需字节数）或 MORE_DATA，两者都要处理
    if err != ERROR_SUCCESS && err != ERROR_MORE_DATA {
        return Err(Win32Error(err.0));
    }
    if size == 0 {
        return Ok(String::new());
    }
    let mut buffer = vec![0u8; size as usize];
    let err = unsafe {
        PowerReadFriendlyName(
            None,
            Some(&scheme),
            None,
            None,
            Some(buffer.as_mut_ptr()),
            &mut size,
        )
    };
    check(err)?;
    let wide: Vec<u16> = buffer
        .as_chunks::<2>().0.iter()
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|unit| *unit != 0)
        .collect();
    String::from_utf16(&wide).map_err(|_| Win32Error(ERROR_NO_UNICODE_TRANSLATION.0))
}

/// 写入计划友好名称（复制计划后命名；UTF-16 + 结尾 NUL）。
pub fn write_friendly_name(guid: Uuid, name: &str) -> Result<(), Win32Error> {
    let scheme = guid_to_windows(guid);
    let mut wide: Vec<u16> = name.encode_utf16().collect();
    wide.push(0);
    let buffer: Vec<u8> = wide.iter().flat_map(|unit| unit.to_le_bytes()).collect();
    check(unsafe { PowerWriteFriendlyName(None, &scheme, None, None, &buffer) })
}

/// 读取当前激活计划 GUID。
pub fn active_scheme() -> Result<Uuid, Win32Error> {
    let mut pointer: *mut GUID = std::ptr::null_mut();
    let err = unsafe { PowerGetActiveScheme(None, &mut pointer) };
    check(err)?;
    let guid = unsafe { *pointer };
    unsafe {
        let _ = LocalFree(Some(windows::Win32::Foundation::HLOCAL(pointer.cast())));
    }
    Ok(Uuid::from_u128(guid.to_u128()))
}

/// 切换当前计划。
pub fn set_active_scheme(guid: Uuid) -> Result<(), Win32Error> {
    let scheme = guid_to_windows(guid);
    check(unsafe { PowerSetActiveScheme(None, Some(&scheme)) })
}

/// 复制电源计划，返回新计划 GUID（对齐旧版：成功但未返回 GUID 视为独立错误）。
pub fn duplicate_scheme(source: Uuid) -> Result<Uuid, PlanError> {
    let source = guid_to_windows(source);
    let mut dest: *mut GUID = std::ptr::null_mut();
    let err = unsafe { PowerDuplicateScheme(None, &source, &mut dest) };
    check(err).map_err(PlanError::Win32)?;
    if dest.is_null() {
        return Err(PlanError::MissingDuplicateGuid);
    }
    let guid = unsafe { *dest };
    unsafe {
        let _ = LocalFree(Some(windows::Win32::Foundation::HLOCAL(dest.cast())));
    }
    Ok(Uuid::from_u128(guid.to_u128()))
}

/// 复制系统卓越性能模板，返回新计划 GUID。
pub fn duplicate_ultimate_performance() -> Result<Uuid, PlanError> {
    duplicate_scheme(ULTIMATE_PERFORMANCE_TEMPLATE)
}

/// 复制计划并重命名。名称校验在调用方完成后再进入本函数，
/// 避免创建副本后命名失败留下无名副本（旧版 changelog 修复项）。
pub fn copy_plan(source: Uuid, new_name: &str) -> Result<Uuid, PlanError> {
    let guid = duplicate_scheme(source)?;
    write_friendly_name(guid, new_name).map_err(PlanError::Win32)?;
    Ok(guid)
}

/// 恢复默认电源计划（需要本地 Administrators 组成员）。
pub fn restore_default_schemes() -> Result<(), Win32Error> {
    check(unsafe { PowerRestoreDefaultPowerSchemes() })
}

/// 列出用户可见电源计划（名称 + 是否激活）。
///
/// 对齐旧版：名称读取失败使整个列表操作失败；名称为空的计划回退显示 GUID 文本。
pub fn list_plans() -> Result<Vec<PlanInfo>, Win32Error> {
    let active = active_scheme()?;
    let mut plans = Vec::new();
    for guid in enumerate_scheme_guids()? {
        let name = friendly_name(guid)?;
        let name = if name.trim().is_empty() {
            guid.to_string()
        } else {
            name
        };
        plans.push(PlanInfo {
            guid: guid.to_string(),
            name,
            is_active: guid == active,
        });
    }
    Ok(plans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_error_display_and_conversion() {
        // 错误展示与 Win32Error → PlanError 自动转换
        let win32 = Win32Error(5);
        let plan_error: PlanError = win32.into();
        assert_eq!(plan_error, PlanError::Win32(Win32Error(5)));
        assert_eq!(
            PlanError::MissingDuplicateGuid.to_string(),
            "duplicate returned no guid"
        );
        assert_eq!(PlanError::Win32(win32).to_string(), "Win32 error 5");
    }

    #[test]
    fn list_plans_returns_active_plan_with_names() {
        let plans = list_plans().expect("enumerate power plans");
        assert!(!plans.is_empty(), "系统至少存在一个电源计划");
        assert!(plans.iter().any(|plan| plan.is_active), "存在激活计划");
        assert!(
            plans.iter().all(|plan| !plan.name.is_empty()),
            "计划名称均非空（空名回退 GUID 文本）"
        );
    }
}
