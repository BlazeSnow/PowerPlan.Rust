//! GUID 与 uuid 互转：Windows GUID 内存布局为 data1/data2/data3 小端 + data4 字节序。

use uuid::Uuid;
use windows::core::GUID;


/// GUID 内存布局为 data1/data2/data3 小端 + data4 字节序，转 UUID 须按字段重组。
pub(super) fn guid_from_bytes(bytes: [u8; 16]) -> Uuid {
    Uuid::from_fields(
        u32::from_le_bytes(bytes[0..4].try_into().expect("16 bytes guid")),
        u16::from_le_bytes(bytes[4..6].try_into().expect("16 bytes guid")),
        u16::from_le_bytes(bytes[6..8].try_into().expect("16 bytes guid")),
        &bytes[8..16].try_into().expect("16 bytes guid"),
    )
}

pub(super) fn guid_to_windows(uuid: Uuid) -> GUID {
    GUID::from_u128(uuid.as_u128())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guid_bytes_conversion_matches_memory_layout() {
        // 模板 GUID e9a42b02-d5df-448d-aa00-03f14749eb61 的内存字节序：
        // data1/data2/data3 小端 + data4 字节序（Windows API 缓冲区布局）
        let bytes: [u8; 16] = [
            0x02, 0x2b, 0xa4, 0xe9, 0xdf, 0xd5, 0x8d, 0x44, 0xaa, 0x00, 0x03, 0xf1, 0x47, 0x49,
            0xeb, 0x61,
        ];
        let uuid = guid_from_bytes(bytes);
        assert_eq!(uuid, crate::core::power::ULTIMATE_PERFORMANCE_TEMPLATE);
        // 反向 uuid → windows GUID → u128 往返一致
        let windows_guid = guid_to_windows(uuid);
        assert_eq!(Uuid::from_u128(windows_guid.to_u128()), uuid);
    }
}
