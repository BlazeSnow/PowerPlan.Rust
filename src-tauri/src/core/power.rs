//! 电源计划：通过 powrprof.dll 原生 API 读取与切换（references/power-plans.md）。
//! 全程普通用户权限；PowerRestoreDefaultPowerSchemes 需要管理员，权限不足时返回 Win32 错误。

use serde::Serialize;
use uuid::Uuid;
use windows::core::GUID;
use windows::Win32::Foundation::{
    LocalFree, ERROR_BAD_LENGTH, ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, ERROR_NO_UNICODE_TRANSLATION,
    ERROR_SUCCESS, WIN32_ERROR,
};
use windows::Win32::System::Power::{
    PowerDuplicateScheme, PowerEnumerate, PowerGetActiveScheme, PowerReadFriendlyName,
    PowerRestoreDefaultPowerSchemes, PowerSetActiveScheme, ACCESS_SCHEME,
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

fn check(err: WIN32_ERROR) -> Result<(), Win32Error> {
    if err == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(Win32Error(err.0))
    }
}

/// GUID 内存布局为 data1/data2/data3 小端 + data4 字节序，转 UUID 须按字段重组。
fn guid_from_bytes(bytes: [u8; 16]) -> Uuid {
    Uuid::from_fields(
        u32::from_le_bytes(bytes[0..4].try_into().expect("16 bytes guid")),
        u16::from_le_bytes(bytes[4..6].try_into().expect("16 bytes guid")),
        u16::from_le_bytes(bytes[6..8].try_into().expect("16 bytes guid")),
        &bytes[8..16].try_into().expect("16 bytes guid"),
    )
}

fn guid_to_windows(uuid: Uuid) -> GUID {
    GUID::from_u128(uuid.as_u128())
}

/// 枚举用户可见电源计划的 GUID。
fn enumerate_scheme_guids() -> Result<Vec<Uuid>, Win32Error> {
    let mut guids = Vec::new();
    let mut index = 0u32;
    loop {
        let mut size = 0u32;
        let err =
            unsafe { PowerEnumerate(None, None, None, ACCESS_SCHEME, index, None, &mut size) };
        if err == ERROR_NO_MORE_ITEMS {
            break;
        }
        if err != ERROR_MORE_DATA {
            return Err(Win32Error(err.0));
        }
        let mut buffer = vec![0u8; size as usize];
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
        check(err)?;
        let bytes: [u8; 16] = buffer[..16].try_into().map_err(|_| Win32Error(ERROR_BAD_LENGTH.0))?;
        guids.push(guid_from_bytes(bytes));
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
    // 实测首调（NULL 缓冲）可能返回 SUCCESS（size=所需字节数）或 MORE_DATA；size=0 表示无可读名称
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
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|unit| *unit != 0)
        .collect();
    String::from_utf16(&wide).map_err(|_| Win32Error(ERROR_NO_UNICODE_TRANSLATION.0))
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

/// 复制系统卓越性能模板，返回新计划 GUID。
pub fn duplicate_ultimate_performance() -> Result<Uuid, Win32Error> {
    let source = guid_to_windows(ULTIMATE_PERFORMANCE_TEMPLATE);
    let mut dest: *mut GUID = std::ptr::null_mut();
    let err = unsafe { PowerDuplicateScheme(None, &source, &mut dest) };
    check(err)?;
    let guid = unsafe { *dest };
    unsafe {
        let _ = LocalFree(Some(windows::Win32::Foundation::HLOCAL(dest.cast())));
    }
    Ok(Uuid::from_u128(guid.to_u128()))
}

/// 恢复默认电源计划（需要本地 Administrators 组成员）。
pub fn restore_default_schemes() -> Result<(), Win32Error> {
    check(unsafe { PowerRestoreDefaultPowerSchemes() })
}

/// 列出用户可见电源计划（名称 + 是否激活）。
pub fn list_plans() -> Result<Vec<PlanInfo>, Win32Error> {
    let active = active_scheme()?;
    let mut plans = Vec::new();
    for guid in enumerate_scheme_guids()? {
        let name = friendly_name(guid).unwrap_or_default();
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
    fn list_plans_returns_active_plan_with_names() {
        let plans = list_plans().expect("enumerate power plans");
        assert!(!plans.is_empty(), "系统至少存在一个电源计划");
        assert!(plans.iter().any(|plan| plan.is_active), "存在激活计划");
        assert!(
            plans.iter().any(|plan| !plan.name.is_empty()),
            "至少一个计划名称可读"
        );
    }
}

