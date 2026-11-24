//! The type scale (RFC 034, job J22): five named steps, none below 13 px, and no literal size anywhere
//! under `ui/` or in `shell.rs` (the preview's mock-ups depict other applications and are exempt).
//! Also the navigation labels: at the new size none wraps.

use std::path::{Path, PathBuf};

use iced::Size;
use logolig_i18n::Locale;

use super::support::{Case, Combo, probe_of};
use crate::ui::type_scale::{BODY, CAPTION, HEADING, ICON_LARGE, ICON_SMALL, SECONDARY, TITLE};

#[test]
fn the_steps_ascend_and_none_is_below_13() {
    let steps = [CAPTION, SECONDARY, BODY, TITLE, HEADING];
    assert!(
        steps.windows(2).all(|w| w[0] < w[1]),
        "ascending: {steps:?}"
    );
    assert!(
        steps.iter().all(|s| *s >= 13.0),
        "no text below 13 px: {steps:?}"
    );
    let icons = [ICON_SMALL, ICON_LARGE];
    assert!(icons.windows(2).all(|w| w[0] < w[1]));
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("a directory") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The files of the interface: everything under `ui/` and `shell.rs`.
fn interface_files() -> Vec<PathBuf> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src.join("ui"), &mut files);
    files.push(src.join("shell.rs"));
    files.sort();
    files
}

/// Where a size may be written as a number: nowhere, except the mock-ups of other applications.
const EXEMPT: &[(&str, &str)] = &[(
    "preview_panel/mockups.rs",
    "depicts a browser tab and a phone's home screen: the sizes are those of the thing depicted",
)];

/// Is `line` a literal size? `.size(` / `.text_size(` / `Pixels(` / `icon_element_sized::<..>(.., N)`
/// followed by a number.
fn has_literal_size(line: &str) -> bool {
    let code = line.split("//").next().unwrap_or("");
    let digit_after = |marker: &str| {
        code.match_indices(marker).any(|(i, _)| {
            code[i + marker.len()..]
                .trim_start()
                .starts_with(|c: char| c.is_ascii_digit())
        })
    };
    digit_after(".size(")
        || digit_after(".text_size(")
        || digit_after("Pixels(")
        || (code.contains("icon_element_sized") && {
            // the last argument is a number: `, 16.0)` or `, 22.0),`
            let t = code.trim_end().trim_end_matches([',', ')']);
            t.rsplit([',', ' '])
                .next()
                .is_some_and(|w| w.parse::<f32>().is_ok() && w.chars().any(|c| c.is_ascii_digit()))
        })
}

#[test]
fn no_literal_size_is_written_in_the_interface() {
    let mut found = Vec::new();
    for file in interface_files() {
        let text = std::fs::read_to_string(&file).expect("a source file");
        let name = file.to_string_lossy().replace('\\', "/");
        if EXEMPT.iter().any(|(suffix, _)| name.ends_with(suffix)) {
            continue;
        }
        // The test modules of the file are not the interface.
        let body = text.split("#[cfg(test)]").next().unwrap_or(&text);
        for (n, line) in body.lines().enumerate() {
            if has_literal_size(line) {
                found.push(format!(
                    "{}:{}: {}",
                    name.rsplit("src/").next().unwrap_or(&name),
                    n + 1,
                    line.trim()
                ));
            }
        }
    }
    assert!(
        found.is_empty(),
        "{} literal size(s); use a step of ui/type_scale.rs:\n{}",
        found.len(),
        found.join("\n")
    );
}

#[test]
fn the_check_can_fail() {
    assert!(has_literal_size("    .size(13)"));
    assert!(has_literal_size("text(x).size( 11 ).color(c)"));
    assert!(has_literal_size(
        "icon_element_sized::<Message>(&Icon::Lucide(lucide::Check), 16.0),"
    ));
    assert!(has_literal_size("Pixels(16.0)"));
    assert!(!has_literal_size("    .size(type_scale::BODY)"));
    assert!(!has_literal_size("text(m).size(size)"));
    assert!(!has_literal_size(
        "icon_element_sized::<Message>(icon, type_scale::ICON_LARGE);"
    ));
    assert!(!has_literal_size("// .size(13) in a comment"));
}

/// CAPTION is for text that nothing depends on: it may be named in these files only.
#[test]
fn caption_is_used_only_where_nothing_depends_on_the_text() {
    const ALLOWED: &[&str] = &["shell.rs", "ui/type_scale.rs"];
    let mut used = Vec::new();
    for file in interface_files() {
        let name = file.to_string_lossy().replace('\\', "/");
        let text = std::fs::read_to_string(&file).expect("a source file");
        let body = text.split("#[cfg(test)]").next().unwrap_or(&text);
        if body.contains("type_scale::CAPTION") && !ALLOWED.iter().any(|a| name.ends_with(a)) {
            used.push(name);
        }
    }
    assert!(
        used.is_empty(),
        "CAPTION used where an action may depend on the text: {used:?}"
    );
}

/// The navigation labels, in both languages, are one line at the new size, desktop and phone.
#[test]
fn no_navigation_label_wraps() {
    for (size, bottom) in [
        (Size::new(1280.0, 720.0), false),
        (Size::new(360.0, 640.0), true),
    ] {
        for locale in [Locale::En, Locale::Ja] {
            let state = Combo {
                case: Case::Home,
                size,
                locale,
                dark: false,
            }
            .state();
            let mut p = probe_of(&state, size);
            let walk = p.walk();
            let labels = state.translator.clone();
            for key in [
                logolig::MessageKey::NavHome,
                logolig::MessageKey::NavCustomize,
                logolig::MessageKey::NavSettings,
            ] {
                let want = labels.t(key);
                let piece = walk
                    .iter()
                    .find(|s| s.text == want)
                    .unwrap_or_else(|| panic!("no nav label {want:?}"));
                assert!(
                    piece.bounds.height < SECONDARY * 1.6,
                    "{locale:?} {} {want:?}: {:.1} px high: it wraps",
                    if bottom { "phone" } else { "desktop" },
                    piece.bounds.height
                );
                assert!(piece.bounds.width <= if bottom { size.width / 3.0 } else { 112.0 });
            }
        }
    }
}

/// The details of a result card (its size, "56.8 KB") are one line at every matrix size: on a phone the
/// card is narrow and the size moves to a line of its own instead of breaking in two (J22).
#[test]
fn a_result_cards_size_is_never_broken_over_two_lines() {
    let mut checked = 0;
    for case in [Case::ResultDefault, Case::ResultAllOptions] {
        for (w, h) in super::support::SIZES {
            let size = Size::new(w, h);
            for locale in Locale::all() {
                let state = Combo {
                    case,
                    size,
                    locale,
                    dark: false,
                }
                .state();
                let mut p = probe_of(&state, size);
                for piece in p.walk() {
                    let is_size = piece.text.ends_with(" KB")
                        || piece.text.ends_with(" B")
                        || piece.text.ends_with(" MB");
                    if is_size
                        && piece
                            .text
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_digit())
                    {
                        checked += 1;
                        assert!(
                            piece.bounds.height < SECONDARY * 1.6,
                            "{case:?} {w}x{h} {locale:?}: {:?} is {:.1} px high: it is broken over two lines",
                            piece.text,
                            piece.bounds.height
                        );
                    }
                }
            }
        }
    }
    assert!(checked > 200, "only {checked} sizes measured");
}
