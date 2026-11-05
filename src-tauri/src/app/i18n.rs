//! The few strings Rust shows itself (tray menu), read from the interface's locale files.

use serde_json::Value;

include!(concat!(env!("OUT_DIR"), "/locales.rs"));

const FALLBACK: &str = "en";

pub struct Translator {
    messages: Value,
    fallback: Value,
}

impl Translator {
    pub fn new(preferred: Option<&str>) -> Self {
        let code = resolve(
            preferred
                .map(str::to_owned)
                .or_else(system_locale)
                .as_deref(),
        );
        Self {
            messages: catalog(code),
            fallback: catalog(FALLBACK),
        }
    }

    /// Looks up a dotted key such as `tray.quit`; missing keys fall back to English, then to the key.
    pub fn t(&self, key: &str) -> String {
        lookup(&self.messages, key)
            .or_else(|| lookup(&self.fallback, key))
            .unwrap_or(key)
            .to_owned()
    }
}

pub fn system_locale() -> Option<String> {
    sys_locale::get_locale()
}

/// Maps a BCP 47 tag such as `fr-CA` to a bundled language, or English.
fn resolve(tag: Option<&str>) -> &'static str {
    let Some(tag) = tag else { return FALLBACK };
    let wanted = tag.replace('_', "-").to_ascii_lowercase();
    let base = wanted.split('-').next().unwrap_or_default();
    LOCALES
        .iter()
        .map(|(code, _)| *code)
        .find(|code| code.eq_ignore_ascii_case(&wanted))
        .or_else(|| {
            LOCALES
                .iter()
                .map(|(code, _)| *code)
                .find(|code| code.eq_ignore_ascii_case(base))
        })
        .unwrap_or(FALLBACK)
}

fn catalog(code: &str) -> Value {
    LOCALES
        .iter()
        .find(|(name, _)| *name == code)
        .and_then(|(_, json)| serde_json::from_str(json).ok())
        .unwrap_or(Value::Null)
}

fn lookup<'a>(messages: &'a Value, key: &str) -> Option<&'a str> {
    key.split('.')
        .try_fold(messages, |node, part| node.get(part))?
        .as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_regional_and_unknown_tags() {
        assert_eq!(resolve(Some("fr-CA")), "fr");
        assert_eq!(resolve(Some("fr_FR")), "fr");
        assert_eq!(resolve(Some("EN")), "en");
        assert_eq!(resolve(Some("xx")), "en");
        assert_eq!(resolve(None), "en");
    }

    #[test]
    fn every_locale_has_the_tray_strings() {
        for (code, _) in LOCALES {
            let messages = catalog(code);
            for key in ["open", "settings", "autostart", "quit"] {
                assert!(
                    lookup(&messages, &format!("tray.{key}")).is_some(),
                    "{code} lacks tray.{key}"
                );
            }
        }
    }

    #[test]
    fn falls_back_to_english_then_to_the_key() {
        let translator = Translator::new(Some("fr"));
        assert_eq!(translator.t("tray.quit"), "Quitter Arthex");
        assert_eq!(translator.t("does.not.exist"), "does.not.exist");
    }
}
