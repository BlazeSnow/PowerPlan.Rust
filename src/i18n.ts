import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import zhHans from "./locales/zh-Hans.json";
import zhHant from "./locales/zh-Hant.json";
import en from "./locales/en.json";
import fr from "./locales/fr.json";
import it from "./locales/it.json";
import de from "./locales/de.json";
import es from "./locales/es.json";

// 键名沿用旧版 resw（含 "."，禁用 i18next 分隔符）；占位符统一 {{0}} 位置参数
// 初始语言：Rust 在页面脚本前注入持久化偏好（window.__POWERPLAN_LANG__）；未注入时按系统语言推断
export const SUPPORTED_LANGUAGES = [
  "zh-Hans",
  "zh-Hant",
  "en",
  "fr",
  "it",
  "de",
  "es",
] as const;

const bootLang = (window as { __POWERPLAN_LANG__?: string })
  .__POWERPLAN_LANG__;

export function resolveLanguage(value?: string): string {
  const trimmed = value?.trim();
  if (
    trimmed &&
    (SUPPORTED_LANGUAGES as readonly string[]).includes(trimmed)
  ) {
    return trimmed;
  }
  const nav = navigator.language.toLowerCase();
  if (
    nav.startsWith("zh-hant") ||
    nav.startsWith("zh-tw") ||
    nav.startsWith("zh-hk") ||
    nav.startsWith("zh-mo")
  ) {
    return "zh-Hant";
  }
  if (nav.startsWith("zh")) return "zh-Hans";
  if (nav.startsWith("fr")) return "fr";
  if (nav.startsWith("it")) return "it";
  if (nav.startsWith("de")) return "de";
  if (nav.startsWith("es")) return "es";
  return "en";
}

void i18n.use(initReactI18next).init({
  resources: {
    "zh-Hans": { translation: zhHans },
    "zh-Hant": { translation: zhHant },
    en: { translation: en },
    fr: { translation: fr },
    it: { translation: it },
    de: { translation: de },
    es: { translation: es },
  },
  lng: resolveLanguage(bootLang),
  fallbackLng: "en",
  keySeparator: false,
  nsSeparator: false,
  interpolation: { escapeValue: false },
});

export default i18n;
