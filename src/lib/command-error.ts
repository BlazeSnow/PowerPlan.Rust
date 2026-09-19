import type { TFunction } from "i18next";
import { toast } from "sonner";
import type { CommandError } from "@/lib/api";

/**
 * 渲染后端命令错误，对齐旧版 LocalizedPowerPlanErrorFormatter 的两级格式：
 * - Win32 包装错误（key=PowerPlan.Error.Win32，args 携带具体错误键 label 与错误码 code）
 *   渲染为「{具体错误文本}：{Win32 代码}」
 * - 其余错误以外层状态文案（fallbackKey）包裹具体错误键文本
 */
export function showCommandError(
  t: TFunction,
  error: unknown,
  fallbackKey: string,
): void {
  const err = error as CommandError | string | undefined;
  if (err && typeof err === "object" && typeof err.key === "string") {
    if (err.key === "PowerPlan.Error.Win32") {
      toast.error(
        t("PowerPlan.Error.Win32", {
          0: t(err.args?.label ?? ""),
          1: err.args?.code ?? "",
        }),
      );
      return;
    }
    toast.error(t(fallbackKey, { 0: t(err.key, err.args ?? {}) }));
    return;
  }
  toast.error(t(fallbackKey, { 0: String(err ?? "unknown") }));
}
