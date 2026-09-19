import { afterEach, describe, expect, it } from "vitest";
import { resolveLanguage } from "@/i18n";

function stubNavigatorLanguage(value: string) {
  Object.defineProperty(window.navigator, "language", {
    value,
    configurable: true,
  });
}

afterEach(() => {
  // 删除实例属性，恢复 jsdom 原型上的只读 getter
  Reflect.deleteProperty(window.navigator, "language");
});

describe("resolveLanguage", () => {
  it.each([
    ["zh-CN", "zh-Hans"],
    ["zh-TW", "zh-Hant"],
    ["zh-HK", "zh-Hant"],
    ["fr-FR", "fr"],
    ["de-DE", "de"],
    ["it-IT", "it"],
    ["es-ES", "es"],
    ["en-US", "en"],
    ["ja-JP", "en"], // 未支持语言回退英语
  ])("maps navigator language %s to %s", (navigatorLanguage, expected) => {
    stubNavigatorLanguage(navigatorLanguage);
    expect(resolveLanguage(undefined)).toBe(expected);
  });

  it("returns explicitly supported language as-is", () => {
    stubNavigatorLanguage("en-US");
    expect(resolveLanguage("zh-Hant")).toBe("zh-Hant");
  });

  it("treats auto and unknown values as system language", () => {
    stubNavigatorLanguage("fr-FR");
    expect(resolveLanguage("auto")).toBe("fr");
    expect(resolveLanguage("xx-YY")).toBe("fr");
  });
});
