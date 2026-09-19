import { afterEach, describe, expect, it } from "vitest";
import { applySystemTheme } from "@/lib/theme";

afterEach(() => {
  document.documentElement.className = "";
  document.documentElement.removeAttribute("style");
});

describe("applySystemTheme", () => {
  it("toggles dark class and inline color-scheme", () => {
    applySystemTheme("dark");
    expect(document.documentElement.classList.contains("dark")).toBe(true);
    expect(document.documentElement.style.colorScheme).toBe("dark");

    applySystemTheme("light");
    expect(document.documentElement.classList.contains("dark")).toBe(false);
    expect(document.documentElement.style.colorScheme).toBe("light");
  });

  it("removes dark class when switching away from dark", () => {
    document.documentElement.classList.add("dark");
    applySystemTheme("light");
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });
});
