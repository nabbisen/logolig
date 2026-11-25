//! Every `MessageKey` is used, deprecated, or deliberately kept (RFC 016, job K4).
//!
//! `MessageKey` is a public, exhaustive enum of the published `logolig` crate, so removing a
//! variant is a SemVer break (1.x keeps them; removal is on the 2.0 list). The compiler never
//! reports an unused variant of a public enum, so this test does: a variant must be named in the
//! application's or the core crate's code (comments and strings do not count), or carry
//! `#[deprecated]`, or be on `UNUSED_UNTIL_2_0`. A new key that nothing uses fails until it is
//! used or listed; a listed key that has come into use fails until it is taken off the list.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::hardcoded_literals::without_inline_tests;
use super::lexer::scan;

/// Keys no screen uses today, kept only for SemVer. Each group has its own reason; delete the
/// group with the keys at 2.0.
const UNUSED_UNTIL_2_0: &[(&[&str], &str)] = &[
    (
        &[
            "AdvancedTitle",
            "AdvancedBlurb",
            "SectionSvg",
            "SectionSvgBlurb",
            "SectionFiles",
            "SectionFilesBlurb",
            "SectionPngSizes",
            "SectionPngSizesBlurb",
            "SectionIcoSizes",
            "SectionIcoSizesBlurb",
            "GroupWhatToExport",
            "GroupExtras",
            "GroupRenderingQuality",
            "GroupAppPreferences",
        ],
        "headings and blurbs of earlier Advanced and Customize layouts: no screen names them (the J23 texts have their own keys)",
    ),
    (
        &[
            "SectionLanguage",
            "SectionLanguageBlurb",
            "LanguageEnglish",
            "LanguageJapanese",
            "LanguageSystemDefault",
            "SettingsTitle",
            "SidebarLabelSettings",
            "TooltipLanguage",
            "TooltipTheme",
        ],
        "texts of the earlier language and theme pop-ups and the settings title: the Settings page and the side bar use other keys",
    ),
    (
        &[
            "ChangeFileButton",
            "SizeAddButton",
            "SizeChipRemove",
            "SizeInputPlaceholder",
            "EmptySetLabel",
            "CloseButton",
            "ExportButton",
            "ToggleAdvancedButton",
            "ToggleThemeButton",
            "EditRepickButton",
            "IncludeIcoLabel",
            "TooltipAdvanced",
            "TooltipClose",
        ],
        "labels, tooltips and placeholders of controls that were replaced or removed",
    ),
    (
        &[
            "ToastExportBody",
            "ToastIcoSizeAlreadyInSetBody",
            "ToastIcoSizeOutOfRangeBody",
            "ToastInvalidColorTitle",
            "ToastInvalidColorBody",
        ],
        "notices that no code raises any more (the size and colour checks use other keys)",
    ),
    (
        &[
            "DropZoneInstruction",
            "PreviewNoSource",
            "PreviewCheckerLabel",
        ],
        "texts of earlier screens (the drop zone, the empty preview, the checker caption)",
    ),
];

/// The variants of `enum MessageKey` in `src`, each with whether it is `#[deprecated]`.
fn variants(src: &str) -> Vec<(String, bool)> {
    let start = src.find("pub enum MessageKey").expect("the enum");
    let body = &src[start..];
    let body = &body[..body.find("\n}\n").expect("the end of the enum")];
    let (mut out, mut deprecated) = (Vec::new(), false);
    for line in body.lines().map(str::trim) {
        if line.starts_with("#[deprecated") {
            deprecated = true;
        }
        if let Some(name) = line.strip_suffix(',')
            && !name.is_empty()
            && name.chars().all(|c| c.is_ascii_alphanumeric())
            && name.starts_with(|c: char| c.is_ascii_uppercase())
        {
            out.push((name.to_string(), deprecated));
            deprecated = false;
        }
    }
    out
}

/// The identifiers of the code of `sources` (no comments, no strings, no inline tests).
fn identifiers<'a>(sources: impl IntoIterator<Item = &'a str>) -> HashSet<String> {
    let mut set = HashSet::new();
    for src in sources {
        let code = scan(without_inline_tests(src)).code;
        for word in code.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
            if !word.is_empty() {
                set.insert(word.to_string());
            }
        }
    }
    set
}

/// (keys nothing uses and nothing excuses, listed keys that are used or do not exist).
fn check(
    variants: &[(String, bool)],
    used: &HashSet<String>,
    listed: &HashSet<&str>,
) -> (Vec<String>, Vec<String>) {
    let unexcused = variants
        .iter()
        .filter(|(name, deprecated)| {
            !used.contains(name) && !deprecated && !listed.contains(name.as_str())
        })
        .map(|(name, _)| name.clone())
        .collect();
    let stale = listed
        .iter()
        .filter(|name| {
            let known = variants.iter().find(|(v, _)| v == *name);
            known.is_none() || used.contains(**name)
        })
        .map(|name| name.to_string())
        .collect();
    (unexcused, stale)
}

/// The sources the keys may be used in: the application and the core crate, not their tests, and
/// not `message_key.rs` itself.
fn code_files() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("read a source folder") {
            let path = entry.expect("an entry").path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                if name != "ui_layout_tests" && !name.ends_with("_tests") {
                    walk(&path, out);
                }
            } else if name.ends_with(".rs")
                && !name.ends_with("_tests.rs")
                && name != "test_support.rs"
                && name != "tests.rs"
                && name != "message_key.rs"
            {
                out.push(path);
            }
        }
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    walk(&manifest.join("src"), &mut out);
    walk(&manifest.join("../core/src"), &mut out);
    out
}

fn listed() -> HashSet<&'static str> {
    UNUSED_UNTIL_2_0
        .iter()
        .flat_map(|(keys, _)| keys.iter().copied())
        .collect()
}

#[test]
fn every_message_key_is_used_deprecated_or_listed() {
    let key_src = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../core/src/message_key.rs"),
    )
    .expect("message_key.rs");
    let texts: Vec<String> = code_files()
        .iter()
        .map(|p| std::fs::read_to_string(p).expect("a source file"))
        .collect();
    let used = identifiers(texts.iter().map(String::as_str));
    let all = variants(&key_src);
    assert!(all.len() > 100, "the enum was read: {} variants", all.len());
    let (unexcused, stale) = check(&all, &used, &listed());
    assert!(
        unexcused.is_empty(),
        "MessageKey variants that nothing uses (use them, mark them #[deprecated], or list them in \
         UNUSED_UNTIL_2_0 with a reason):\n{}",
        unexcused.join("\n")
    );
    assert!(
        stale.is_empty(),
        "UNUSED_UNTIL_2_0 lists keys that are used or no longer exist (take them off):\n{}",
        stale.join("\n")
    );
}

#[test]
fn every_group_of_the_list_has_a_reason_and_no_key_twice() {
    let mut seen = HashSet::new();
    for (keys, reason) in UNUSED_UNTIL_2_0 {
        assert!(!keys.is_empty() && !reason.is_empty());
        for key in *keys {
            assert!(seen.insert(*key), "{key} is listed twice");
        }
    }
}

/// The guard can fail, both ways, on a small enum: a new unused key is reported; a listed key that
/// is used (or gone) is reported; a deprecated or listed one is not.
#[test]
fn the_check_fails_for_a_new_unused_key_and_for_a_stale_entry() {
    let enum_src = "pub enum MessageKey {\n    Used,\n    Reserved,\n    #[deprecated(note = \"x\")]\n    Old,\n    Fresh,\n}\n";
    let all = variants(enum_src);
    assert_eq!(
        all,
        vec![
            ("Used".into(), false),
            ("Reserved".into(), false),
            ("Old".into(), true),
            ("Fresh".into(), false)
        ]
    );
    let used = identifiers(["fn f() { let _ = MessageKey::Used; /* Fresh */ let _ = \"Fresh\"; }"]);
    // Fresh is named only in a comment and a string: not used.
    let (unexcused, stale) = check(&all, &used, &HashSet::from(["Reserved"]));
    assert_eq!(unexcused, vec!["Fresh".to_string()]);
    assert!(stale.is_empty());
    // A listed key that is used, and a listed key that does not exist.
    let (_, stale) = check(&all, &used, &HashSet::from(["Reserved", "Used", "Gone"]));
    let mut stale = stale;
    stale.sort();
    assert_eq!(stale, vec!["Gone".to_string(), "Used".to_string()]);
}
