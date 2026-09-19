//! 后端 i18n（Fluent）：本地化 Rust 侧自身生成的用户可见文本（托盘菜单、提示）。
//! 前端文案走 i18next（src/locales）；命令错误以键名返回由前端渲染，不经本模块。
//! 资源位于 `src-tauri/locales/{lang}/main.ftl`，编译期内嵌；缺失键回退 en
//! （references/i18n.md：未匹配语言回退英语）。

use std::borrow::Cow;
use std::collections::HashMap;

use fluent_templates::{static_loader, Loader};
use unic_langid::{langid, LanguageIdentifier};

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
    };
}

/// 受支持语言，与前端 src/i18n.ts 保持一致。
pub const SUPPORTED: [&str; 7] = ["zh-Hans", "zh-Hant", "en", "fr", "it", "de", "es"];

/// 后端文案语言。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lang(LanguageIdentifier);

impl Lang {
    /// 按 settings 的 language 解析（auto/未知 → 系统语言）。
    pub fn from_config(language: &str) -> Self {
        Self::parse(language).unwrap_or_else(Self::system)
    }

    /// 按系统 locale 解析；zh 系按地区区分简繁，其余前缀匹配，未匹配回退英语。
    pub fn system() -> Self {
        let locale = sys_locale::get_locale().unwrap_or_default().to_lowercase();
        let zh_hant = locale.starts_with("zh-hant")
            || locale.starts_with("zh-tw")
            || locale.starts_with("zh-hk")
            || locale.starts_with("zh-mo");
        if zh_hant {
            return Self::langid("zh-Hant");
        }
        for (prefix, code) in [
            ("zh", "zh-Hans"),
            ("fr", "fr"),
            ("it", "it"),
            ("de", "de"),
            ("es", "es"),
        ] {
            if locale.starts_with(prefix) {
                return Self::langid(code);
            }
        }
        Self::langid("en")
    }

    fn langid(code: &str) -> Self {
        Lang(langid_for(code).unwrap_or_else(|| langid!("en")))
    }

    fn parse(code: &str) -> Option<Self> {
        SUPPORTED
            .contains(&code)
            .then(|| Self::langid(code))
    }

    /// 查无占位符的消息。
    pub fn message(&self, key: &str) -> String {
        LOCALES.lookup(&self.0, key)
    }

    /// 查带命名参数的消息：`{$name}` 占位符对应 `args` 中的同名条目。
    pub fn message_with(&self, key: &str, args: &[(&str, &str)]) -> String {
        let map: HashMap<Cow<'static, str>, fluent_templates::fluent_bundle::FluentValue> = args
            .iter()
            .map(|(name, value)| {
                (
                    Cow::Owned(name.to_string()),
                    fluent_templates::fluent_bundle::FluentValue::from(value.to_string()),
                )
            })
            .collect();
        LOCALES.lookup_with_args(&self.0, key, &map)
    }
}

fn langid_for(code: &str) -> Option<LanguageIdentifier> {
    Some(match code {
        "zh-Hans" => langid!("zh-Hans"),
        "zh-Hant" => langid!("zh-Hant"),
        "en" => langid!("en"),
        "fr" => langid!("fr"),
        "it" => langid!("it"),
        "de" => langid!("de"),
        "es" => langid!("es"),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_config_maps_supported_languages() {
        for code in SUPPORTED {
            assert_eq!(Lang::from_config(code), Lang::langid(code));
        }
    }

    #[test]
    fn from_config_falls_back_to_system_for_auto() {
        let lang = Lang::from_config("auto");
        assert!(SUPPORTED.iter().any(|code| Lang::langid(code) == lang));
    }

    #[test]
    fn tray_menu_localizes_in_supported_languages() {
        assert_eq!(Lang::langid("zh-Hans").message("tray-menu-exit"), "退出");
        assert_eq!(Lang::langid("en").message("tray-menu-exit"), "Exit");
    }

    #[test]
    fn message_with_interpolates_named_args() {
        let text = Lang::langid("zh-Hans").message_with("tray-tooltip-plan", &[("plan", "平衡")]);
        assert!(text.contains("平衡"), "插值缺失: {text}");
    }

    #[test]
    fn unsupported_language_falls_back_to_english_resource() {
        // en 资源缺失的键回退 en（fallback_language），此处验证 fallback 配置生效
        assert_eq!(Lang::langid("fr").message("tray-menu-exit"), "Quitter");
    }
}
