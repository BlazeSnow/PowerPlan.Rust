import type { PlanInfo } from "@/lib/api";

/** 系统卓越性能模板 GUID，识别规则见 references/power-plans.md */
export const ULTIMATE_TEMPLATE_GUID = "e9a42b02-d5df-448d-aa00-03f14749eb61";

/** 卓越性能卡片三态（对齐旧版 ApplyPlansToView） */
export type UltimateState = "exists" | "hidden" | "missing";

/**
 * 解析卓越性能卡片状态：
 * - 模板 GUID 或储存 UUID 出现在计划列表中 → exists（隐藏卡片）
 * - 储存 UUID 存在但计划不在列表（被隐藏/已删除前状态）→ hidden（提供激活）
 * - 其余 → missing（提供创建）
 */
export function resolveUltimateState(
  plans: PlanInfo[],
  savedGuid: string | null,
): UltimateState {
  const saved = savedGuid?.trim() || null;
  const exists =
    plans.some((plan) => plan.guid === ULTIMATE_TEMPLATE_GUID) ||
    (saved !== null && plans.some((plan) => plan.guid === saved));
  if (exists) {
    return "exists";
  }
  return saved !== null ? "hidden" : "missing";
}

/** 复制对话框预填名：空名回退默认名称，统一「名称 - 副本」后缀（对齐旧版 BuildCopyPlanName） */
export function buildCopyPlanName(
  name: string | null | undefined,
  defaultName: string,
  suffix: string,
): string {
  return `${name?.trim() || defaultName} - ${suffix}`;
}
