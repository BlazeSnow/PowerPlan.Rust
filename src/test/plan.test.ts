import { describe, expect, it } from "vitest";
import type { PlanInfo } from "@/lib/api";
import {
  buildCopyPlanName,
  resolveUltimateState,
  ULTIMATE_TEMPLATE_GUID,
} from "@/lib/plan";

const plan = (guid: string, name = "", isActive = false): PlanInfo => ({
  guid,
  name,
  isActive,
});

describe("resolveUltimateState", () => {
  it("exists when template guid is in plans", () => {
    const plans = [plan(ULTIMATE_TEMPLATE_GUID, "卓越性能", true)];
    expect(resolveUltimateState(plans, null)).toBe("exists");
  });

  it("exists when saved guid is in plans", () => {
    const saved = "99999999-8888-7777-6666-555555555555";
    const plans = [plan("381b4222-f694-41f0-9685-ff5bb260df2e", "平衡"), plan(saved, "卓越性能")];
    expect(resolveUltimateState(plans, saved)).toBe("exists");
  });

  it("hidden when saved guid is absent from plans", () => {
    const plans = [plan("381b4222-f694-41f0-9685-ff5bb260df2e", "平衡")];
    expect(
      resolveUltimateState(plans, "99999999-8888-7777-6666-555555555555"),
    ).toBe("hidden");
  });

  it("missing when no saved guid", () => {
    expect(resolveUltimateState([], null)).toBe("missing");
  });

  it("treats blank saved guid as missing", () => {
    expect(resolveUltimateState([], "   ")).toBe("missing");
  });
});

describe("buildCopyPlanName", () => {
  it("trims source name and appends suffix", () => {
    expect(buildCopyPlanName(" 平衡 ", "电源计划", "副本")).toBe("平衡 - 副本");
  });

  it("falls back to default name when blank", () => {
    expect(buildCopyPlanName("  ", "电源计划", "副本")).toBe("电源计划 - 副本");
    expect(buildCopyPlanName(null, "电源计划", "副本")).toBe("电源计划 - 副本");
  });
});
