# LightCraft in Hebrew (עברית)

LightCraft's interface is available in English, Japanese and Hebrew. The language is stored in
`ui.json` (`language`: `en`, `ja` or `he`).

- Switch with **Edit → Language → עברית** or **Settings → General → Language**; the choice is kept
  for the next launch. `LIGHTCRAFT_LANGUAGE=he` sets the default for a fresh profile.
- Everything the Japanese catalog covers is translated: menus, the Edit panels, masking, crop,
  settings, import, export and progress messages. Untranslated technical errors and release notes
  stay in English, as in Japanese.
- Hebrew uses the bundled Noto Sans Hebrew (SIL OFL), Regular and SemiBold.
- egui draws text left to right and applies no bidirectional algorithm. `crates/ui-egui/src/rtl_text.rs`
  is an egui plugin that puts every drawn row containing right-to-left text into visual order
  (Unicode UAX #9, after wrapping) and right-aligns right-to-left paragraphs, so Hebrew mixed with
  numbers and Latin text reads correctly. Catalogs keep text in logical (reading) order.
- Not done yet: the panel layout is not mirrored (menus, the toolbar and side panels keep their
  left-to-right positions), and a caret inside Hebrew text being edited can be drawn away from its
  letter. Watermark text in exports is not shaped right to left.
- Translation is presentation-only: command ids, file names and metadata you type never change.

## Maintaining the translation

Static labels are in `crates/ui-egui/locales/he.json`; labels with values are in
`crates/ui-egui/locales/he-formats.json`, keyed by the English text like the Japanese files. The
build checks every language's format placeholders with Rust's `format!`. Where English adds a plural
`s`, the Hebrew form consumes that argument with `{:.0}`; counts that can be 1 are written as
"תמונות: {n}" so they read correctly for any number.

`LIGHTCRAFT_LANGUAGE=he lightcraft-cli snapshot --demo -o he.png` renders the Hebrew UI headlessly.
`cargo test -p lightcraft-ui-egui i18n:: rtl_text::` checks catalog coverage, formats, fonts and the
right-to-left drawing.
