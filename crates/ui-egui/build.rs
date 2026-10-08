//! Compile translated format strings so Rust checks every language's placeholders.
use std::{collections::BTreeMap, env, fs, path::PathBuf};

/// Translated format catalogs: (Language variant, file). A missing entry falls back to English.
const FORMATS: [(&str, &str); 2] = [("Ja", "locales/ja-formats.json"), ("He", "locales/he-formats.json")];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut catalogs = Vec::new();
    for (variant, path) in FORMATS {
        println!("cargo:rerun-if-changed={path}");
        let messages: BTreeMap<String, String> = serde_json::from_str(&fs::read_to_string(path)?)?;
        catalogs.push((variant, messages));
    }
    let mut english: Vec<&String> = catalogs.iter().flat_map(|(_, m)| m.keys()).collect();
    english.sort();
    english.dedup();
    let mut source = String::from("macro_rules! tr_format {\n");
    for key in english {
        let en = serde_json::to_string(key)?;
        let mut arms = String::new();
        for (variant, messages) in &catalogs {
            if let Some(translated) = messages.get(key) {
                let tr = serde_json::to_string(translated)?;
                arms.push_str(&format!("$crate::i18n::Language::{variant} => format!({tr} $(, $($args)*)?), "));
            }
        }
        source
            .push_str(&format!("({en} $(, $($args:tt)*)?) => {{ match $crate::i18n::current() {{ {arms}_ => format!({en} $(, $($args)*)?) }} }};\n"));
    }
    source.push_str("}\npub(crate) use tr_format;\n");
    fs::write(PathBuf::from(env::var("OUT_DIR")?).join("formats.rs"), source)?;
    Ok(())
}
