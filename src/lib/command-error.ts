import type { TFunction } from "i18next";
import { toast } from "sonner";
import type { CommandError } from "@/lib/api";

/** 渲染后端命令错误：有键名走 i18next，否则回退到指定文案拼接原始信息 */
export function showCommandError(
  t: TFunction,
  error: unknown,
  fallbackKey: string,
): void {
  const err = error as CommandError | string | undefined;
  if (err && typeof err === "object" && typeof err.key === "string") {
    toast.error(t(err.key, err.args ?? {}));
    return;
  }
  toast.error(t(fallbackKey, { 0: String(err ?? "unknown") }));
}
