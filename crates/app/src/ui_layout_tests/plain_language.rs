//! The plain-language Customize page, as drawn (RFC 034, job J23). The texts themselves are pinned
//! in `crates/i18n/tests/plain_language.rs`; here is what needs the application:
//!
//! * "recommended" is on the default resize algorithm and the default trace preset, and only there
//!   (so a changed default fails the build until the text follows);
//! * each blurb is on the page, in both languages, directly under its heading;
//! * no placeholder is a literal (a literal is English in the Japanese interface, as `e.g. 256` was).

use iced::Size;
use logolig::{ExportPlan, MessageKey, ResizeAlgorithm, VtracerPreset};
use logolig_i18n::{Locale, Translator};

use super::support::{Case, Combo, Kind, probe_of};
use super::type_scale::interface_files;
use crate::ui::advanced_drawer::choices::{algorithm_message_key, preset_message_key};

/// The marker of the default in each language.
const RECOMMENDED: [(Locale, &str); 2] = [(Locale::En, "recommended"), (Locale::Ja, "推奨")];

#[test]
fn recommended_is_on_the_default_algorithm_and_the_default_preset_only() {
    for (locale, marker) in RECOMMENDED {
        let t = Translator::for_locale(locale);
        for algorithm in ResizeAlgorithm::all() {
            let label = t.t(algorithm_message_key(algorithm));
            assert_eq!(
                label.contains(marker),
                algorithm == ResizeAlgorithm::default(),
                "[{}] {algorithm:?}: {label}",
                locale.as_bcp47()
            );
        }
        for preset in VtracerPreset::all() {
            let label = t.t(preset_message_key(preset));
            assert_eq!(
                label.contains(marker),
                preset == VtracerPreset::default(),
                "[{}] {preset:?}: {label}",
                locale.as_bcp47()
            );
        }
    }
}

/// The default is what the page says it is, and the plan starts there.
#[test]
fn the_plan_starts_at_the_recommended_choices() {
    let plan = ExportPlan::default();
    assert_eq!(plan.algorithm, ResizeAlgorithm::Lanczos3);
    assert_eq!(plan.vtracer_preset, VtracerPreset::Default);
}

/// Each blurb is drawn directly after its heading, in both languages: nothing between them.
#[test]
fn each_blurb_is_on_the_page_directly_under_its_heading() {
    let pairs = [
        (
            MessageKey::SectionPngOutputSizes,
            MessageKey::SectionPngOutputSizesBlurb,
        ),
        (
            MessageKey::SectionSvgConversion,
            MessageKey::SectionSvgConversionBlurb,
        ),
        (MessageKey::SectionMisc, MessageKey::KeepTransparencyBlurb),
    ];
    for locale in Locale::all() {
        let t = Translator::for_locale(locale);
        let combo = Combo {
            case: Case::CustomizeAdvanced,
            size: Size::new(1000.0, 4000.0),
            locale,
            dark: false,
        };
        let state = combo.state();
        let mut p = probe_of(&state, combo.size);
        let texts: Vec<String> = p
            .walk()
            .into_iter()
            .filter(|s| s.kind == Kind::Text)
            .map(|s| s.text)
            .collect();
        for (title, blurb) in pairs {
            let at = texts
                .iter()
                .position(|s| *s == t.t(title))
                .unwrap_or_else(|| {
                    panic!(
                        "[{}] the heading {title:?} is not on the page",
                        locale.as_bcp47()
                    )
                });
            assert_eq!(
                texts.get(at + 1),
                Some(&t.t(blurb)),
                "[{}] {blurb:?} is not directly under {title:?}",
                locale.as_bcp47()
            );
        }
    }
}

/// The placeholder of the custom-size field is translated, in Japanese too.
#[test]
fn the_custom_size_placeholder_is_translated() {
    let en = Translator::for_locale(Locale::En);
    let ja = Translator::for_locale(Locale::Ja);
    assert_ne!(
        en.t(MessageKey::CustomSizePlaceholder),
        ja.t(MessageKey::CustomSizePlaceholder)
    );
}

/// The lines of `source` where a text field is given a literal placeholder: `text_input("…"`, also
/// when the formatter has put the argument on the next line. Comments are skipped.
fn literal_placeholders(source: &str) -> Vec<usize> {
    let code: Vec<&str> = source
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect();
    let mut found = Vec::new();
    for (n, line) in code.iter().enumerate() {
        if let Some(i) = line.find("text_input(") {
            let rest = line[i + "text_input(".len()..].trim();
            let next = if rest.is_empty() {
                code.get(n + 1).map_or("", |l| l.trim())
            } else {
                rest
            };
            if next.starts_with('"') {
                found.push(n + 1);
            }
        }
    }
    found
}

#[test]
fn no_text_field_in_the_interface_has_a_literal_placeholder() {
    let mut found = Vec::new();
    for file in interface_files() {
        let text = std::fs::read_to_string(&file).expect("a source file");
        let body = text.split("#[cfg(test)]").next().unwrap_or(&text);
        for n in literal_placeholders(body) {
            found.push(format!("{}:{n}", file.display()));
        }
    }
    assert!(
        found.is_empty(),
        "a literal placeholder is untranslated:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_literal_placeholder_check_can_fail() {
    let one_line = r#"        text_input("e.g. 256", &state.png_size_input)"#;
    let wrapped = "        text_input(\n            \"e.g. 256\",\n            &state.png_size_input\n        )";
    assert_eq!(literal_placeholders(one_line), [1]);
    assert_eq!(literal_placeholders(wrapped), [1]);
    assert!(
        literal_placeholders("        text_input(&t.t(MessageKey::CustomSizePlaceholder), &x)")
            .is_empty()
    );
    assert!(literal_placeholders("        text_input(\n            &t.t(MessageKey::CustomSizePlaceholder),\n            &x)").is_empty());
    assert!(literal_placeholders(r#"        // text_input("e.g. 256")"#).is_empty());
}
