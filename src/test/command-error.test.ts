import { beforeEach, describe, expect, it, vi } from "vitest";
import type { TFunction } from "i18next";
import { toast } from "sonner";
import { showCommandError } from "@/lib/command-error";

vi.mock("sonner", () => ({
  toast: { error: vi.fn(), success: vi.fn() },
}));

// 确定性 t：带非空参数时渲染为「key(参数JSON)」，便于断言两级嵌套结构
const tMock = vi.fn(
  (key: string, args?: Record<string, unknown>) =>
    args && Object.keys(args).length > 0
      ? `${key}(${JSON.stringify(args)})`
      : key,
);
// vi.fn 的签名不满足 TFunction 的品牌类型，测试中按实际行为断言
const t = tMock as unknown as TFunction;

beforeEach(() => {
  vi.clearAllMocks();
});

describe("showCommandError", () => {
  it("renders Win32 wrapper error with localized label and code", () => {
    showCommandError(
      t,
      {
        key: "PowerPlan.Error.Win32",
        args: { label: "PowerPlan.Error.SetActiveFailed", code: "5" },
      },
      "Main.Status.SwitchFailed",
    );

    // 内层先取具体错误键文本，外层套 Win32 模板（{0}：{1}）
    expect(tMock).toHaveBeenNthCalledWith(1, "PowerPlan.Error.SetActiveFailed");
    expect(tMock).toHaveBeenNthCalledWith(2, "PowerPlan.Error.Win32", {
      0: "PowerPlan.Error.SetActiveFailed",
      1: "5",
    });
    expect(toast.error).toHaveBeenCalledWith(
      'PowerPlan.Error.Win32({"0":"PowerPlan.Error.SetActiveFailed","1":"5"})',
    );
  });

  it("wraps plain command error with fallback status key", () => {
    showCommandError(
      t,
      { key: "PowerPlan.Error.EmptyName", args: {} },
      "Main.Status.CopyFailed",
    );

    expect(tMock).toHaveBeenNthCalledWith(2, "Main.Status.CopyFailed", {
      0: "PowerPlan.Error.EmptyName",
    });
    expect(toast.error).toHaveBeenCalledWith(
      'Main.Status.CopyFailed({"0":"PowerPlan.Error.EmptyName"})',
    );
  });

  it("falls back to raw string for unknown errors", () => {
    showCommandError(t, "boom", "Main.Status.RefreshFailed");

    expect(toast.error).toHaveBeenCalledWith(
      'Main.Status.RefreshFailed({"0":"boom"})',
    );
  });

  it("falls back to unknown marker for empty errors", () => {
    showCommandError(t, undefined, "Main.Status.RefreshFailed");

    expect(toast.error).toHaveBeenCalledWith(
      'Main.Status.RefreshFailed({"0":"unknown"})',
    );
  });
});
