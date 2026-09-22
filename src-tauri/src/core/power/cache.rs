//! 计划列表缓存：托盘常驻场景避免重复枚举；所有写操作后必须失效。

use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use super::{PlanInfo, Win32Error};
use super::list_plans;

/// 计划列表缓存：托盘常驻场景避免重复枚举；所有写操作后必须失效。
/// 旧版另有并发去重（single-flight），本版命令为同步快速调用，无并发抓取路径，从简。
const PLANS_CACHE_TTL: Duration = Duration::from_secs(5 * 60);

struct PlansCacheEntry {
    at: Instant,
    plans: Vec<PlanInfo>,
}

static PLANS_CACHE: LazyLock<Mutex<Option<PlansCacheEntry>>> =
    LazyLock::new(|| Mutex::new(None));

/// 带缓存的计划列表；`force` 跳过缓存强制刷新。
pub fn list_plans_cached(force: bool) -> Result<Vec<PlanInfo>, Win32Error> {
    let mut guard = PLANS_CACHE.lock().unwrap();
    if !force
        && let Some(entry) = guard.as_ref()
            && entry.at.elapsed() < PLANS_CACHE_TTL {
                return Ok(entry.plans.clone());
            }
    let plans = list_plans()?;
    *guard = Some(PlansCacheEntry {
        at: Instant::now(),
        plans: plans.clone(),
    });
    Ok(plans)
}

/// 使计划缓存失效；切换、复制、创建、恢复默认后调用。
pub fn invalidate_plans_cache() {
    *PLANS_CACHE.lock().unwrap() = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_list_matches_direct_list() {
        invalidate_plans_cache();
        let direct = super::super::list_plans().expect("direct list");
        let cached = list_plans_cached(false).expect("cached list");
        assert_eq!(direct, cached);
        let forced = list_plans_cached(true).expect("forced list");
        assert_eq!(direct, forced);
        invalidate_plans_cache();
    }
}
