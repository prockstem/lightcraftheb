//! Presentation-only localization. Command ids, user names and catalog data remain unchanged.
use std::{cell::Cell, collections::BTreeMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/formats.rs"));

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    En,
    #[serde(rename = "ja")]
    Ja,
    #[serde(rename = "he")]
    He,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::En, Self::Ja, Self::He];
    pub fn name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Ja => "日本語",
            Self::He => "עברית",
        }
    }
    pub fn parse(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Self::En),
            "ja" => Some(Self::Ja),
            "he" => Some(Self::He),
            _ => None,
        }
    }
    /// Is text in this language written right to left?
    pub fn is_rtl(self) -> bool {
        self == Self::He
    }
    pub fn tr(self, source: &str) -> &str {
        match catalog(self) {
            Some(messages) => messages.get(source).map(String::as_str).unwrap_or(source),
            None => source,
        }
    }
}

thread_local! {
    static LANGUAGE: Cell<Language> = const { Cell::new(Language::En) };
}

pub fn default_language() -> Language {
    std::env::var("LIGHTCRAFT_LANGUAGE").ok().and_then(|code| Language::parse(&code)).unwrap_or_default()
}

pub fn set_language(language: Language) {
    LANGUAGE.with(|value| value.set(language));
}

/// The language the UI is drawn in on this thread.
pub fn current() -> Language {
    LANGUAGE.with(Cell::get)
}

pub fn is_japanese() -> bool {
    current() == Language::Ja
}

fn catalog(language: Language) -> Option<&'static BTreeMap<String, String>> {
    match language {
        Language::En => None,
        Language::Ja => Some(japanese()),
        Language::He => Some(hebrew()),
    }
}

fn japanese() -> &'static BTreeMap<String, String> {
    static MESSAGES: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    MESSAGES.get_or_init(|| load("Japanese", include_str!("../locales/ja.json")))
}

fn hebrew() -> &'static BTreeMap<String, String> {
    static MESSAGES: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    MESSAGES.get_or_init(|| load("Hebrew", include_str!("../locales/he.json")))
}

fn load(name: &str, json: &str) -> BTreeMap<String, String> {
    serde_json::from_str(json).unwrap_or_else(|error| {
        log::error!("Invalid {name} message catalog: {error}");
        BTreeMap::new()
    })
}

/// Translate a built-in display label, preserving unknown labels verbatim.
/// Never call this on editable user text, filenames or command identifiers.
pub fn tr(source: &str) -> &str {
    current().tr(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_valid_and_contains_core_workflows() {
        let messages: BTreeMap<String, String> = serde_json::from_str(include_str!("../locales/ja.json")).unwrap();
        for key in ["Import Photos…", "Export…", "Exposure", "White Balance", "Settings", "Language"] {
            assert!(messages.get(key).is_some_and(|value| !value.is_empty() && value != key), "{key}");
        }
    }

    #[test]
    fn hebrew_catalog_covers_every_japanese_string() {
        let ja: BTreeMap<String, String> = serde_json::from_str(include_str!("../locales/ja.json")).unwrap();
        let he: BTreeMap<String, String> = serde_json::from_str(include_str!("../locales/he.json")).unwrap();
        let missing: Vec<_> = ja.keys().filter(|k| he.get(*k).is_none_or(|v| v.is_empty())).collect();
        assert!(missing.is_empty(), "untranslated: {missing:?}");
        let jf: BTreeMap<String, String> = serde_json::from_str(include_str!("../locales/ja-formats.json")).unwrap();
        let hf: BTreeMap<String, String> = serde_json::from_str(include_str!("../locales/he-formats.json")).unwrap();
        assert_eq!(jf.keys().collect::<Vec<_>>(), hf.keys().collect::<Vec<_>>());
    }

    #[test]
    fn hebrew_translates_labels_and_formats() {
        set_language(Language::He);
        assert!(current().is_rtl());
        assert_eq!(tr("Exposure"), "חשיפה");
        assert_eq!(tr("my-photo.jpg"), "my-photo.jpg");
        assert_eq!(tr_format!("{n} photo{}", "s", n = 12), "תמונות: 12");
        assert_eq!(tr_format!("Exported {ok} of {total} photo{}", "s", ok = 4, total = 12), "יוצאו 4 מתוך 12 תמונות");
        assert_eq!(tr_format!("Added {n} photo{} to “{}”", "s", "Trip", n = 3), "נוספו אל “Trip”: 3");
        assert_eq!(Language::parse("he"), Some(Language::He));
        assert_eq!(serde_json::to_string(&Language::He).unwrap(), "\"he\"");
        set_language(Language::En);
        assert_eq!(tr("Exposure"), "Exposure");
    }

    #[test]
    fn both_font_weights_cover_the_hebrew_catalog() {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        ctx.fonts_mut(|fonts| {
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Name(crate::theme::FONT_SEMIBOLD.into())] {
                let font = egui::FontId::new(13.0, family);
                // Characters the translation adds (symbols copied from the English source are the
                // English UI's concern).
                for (source, message) in hebrew() {
                    for ch in message.chars().filter(|ch| !ch.is_whitespace() && !source.contains(*ch)) {
                        assert!(fonts.has_glyph(&font, ch), "Missing glyph {ch} in {message}");
                    }
                }
            }
        });
    }

    #[test]
    fn language_switches_and_unknown_text_survives() {
        set_language(Language::Ja);
        assert_eq!(tr("Exposure"), "露出");
        assert_eq!(tr("my-photo.jpg"), "my-photo.jpg");
        assert_eq!(tr("develop.set"), "develop.set");
        assert_eq!(crate::menubar::display_item_label("album.addPhotos", &serde_json::json!({"id": 1}), "Color"), "Color");
        assert_eq!(crate::menubar::display_item_label("app.export", &serde_json::json!({"preset": "Color"}), "Color"), "Color");
        assert_eq!(crate::menubar::display_item_label("view.photoGrid", &serde_json::Value::Null, "Color"), "カラー");
        set_language(Language::En);
        assert_eq!(tr("Exposure"), "Exposure");
    }

    #[test]
    fn translated_formats_preserve_counts_and_remove_english_plural_suffixes() {
        set_language(Language::Ja);
        assert_eq!(tr_format!("{n} photo{}", "s", n = 12), "12枚");
        assert_eq!(tr_format!("Exported {ok} of {total} photo{}", "s", ok = 4, total = 12), "12枚中4枚を書き出しました");
        set_language(Language::En);
        assert_eq!(tr_format!("{n} photo{}", "s", n = 12), "12 photos");
    }

    #[test]
    fn preferences_round_trip_and_old_settings_remain_readable() {
        let old: crate::state::UiState = serde_json::from_str("{}").unwrap();
        assert_eq!(old.language, Language::En);
        let settings = crate::state::UiState { language: Language::Ja, ..old };
        let saved = serde_json::to_string(&settings).unwrap();
        let restored: crate::state::UiState = serde_json::from_str(&saved).unwrap();
        assert_eq!(restored.language, Language::Ja);
    }

    #[test]
    fn japanese_is_painted_and_both_font_weights_cover_the_catalog() {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut app = crate::LightcraftApp::new(lightcraft_engine::Session::with_demo(), Default::default());
        app.ui.language = Language::Ja;
        app.ui.left_panel = true;
        let mut text = String::new();
        fn collect(shape: &egui::epaint::Shape, text: &mut String) {
            match shape {
                egui::epaint::Shape::Text(shape) => {
                    text.push_str(&shape.galley.job.text);
                    text.push('\n');
                }
                egui::epaint::Shape::Vec(shapes) => shapes.iter().for_each(|shape| collect(shape, text)),
                _ => {}
            }
        }
        for frame in 0..4 {
            let input = crate::headless::HeadlessView::raw_input(egui::vec2(1600.0, 1000.0), 1.0, frame as f64 / 60.0, vec![]);
            let mut out = ctx.run_ui(input, |ui| {
                app.logic(ui.ctx());
                app.ui(ui);
            });
            // This assertion inspects shapes without a renderer; discard texture uploads explicitly.
            out.textures_delta.clear();
            text.clear();
            for shape in out.shapes {
                collect(&shape.shape, &mut text);
            }
        }
        assert!(text.contains("マイフォト"), "{text}");
        assert!(text.contains("すべての写真"), "{text}");
        ctx.fonts_mut(|fonts| {
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Name(crate::theme::FONT_SEMIBOLD.into())] {
                let font = egui::FontId::new(13.0, family);
                for message in japanese().values() {
                    for ch in message.chars().filter(|ch| !ch.is_whitespace()) {
                        assert!(fonts.has_glyph(&font, ch), "Missing glyph {ch} in {message}");
                    }
                }
            }
        });
        // Locale affects presentation only: command ids remain the same.
        let ids = |app: &crate::LightcraftApp| crate::menus::menu_entries(app).into_iter().map(|entry| entry.id).collect::<Vec<_>>();
        let japanese_ids = ids(&app);
        set_language(Language::En);
        assert_eq!(japanese_ids, ids(&app));
    }
}
