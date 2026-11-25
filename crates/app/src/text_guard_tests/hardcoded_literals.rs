//! No hardcoded user-visible text (RFC 016, job K4).
//!
//! Every alphabetic string literal in the application's non-test sources must be on `EXEMPT`
//! (by file, by literal, with a reason for each group) or be one of the few call forms that never
//! reach the screen (`CALLS`). Anything else is text a user may see in one language only: make it
//! a `MessageKey`, in `en.toml` and `ja.toml`. A stale entry (its literal is gone) fails too, so
//! the list cannot grow into a place to hide things.

use std::path::{Path, PathBuf};

use super::lexer::{Literal, scan};

/// Calls whose literal argument is never shown: a panic message of an invariant, a debug name, a
/// widget id, an environment variable, a file name of the settings file, a `cfg` value.
const CALLS: &[&str] = &[
    ".expect(",
    ".field(",
    "debug_struct(",
    "Id::new(",
    "var_os(",
    ".with_filename(",
    "target_os =",
    "feature =",
];

const PLACEHOLDER: &str =
    "the name of a `{placeholder}` of a translated sentence: an identifier, not prose";
const DETAIL: &str = "technical detail of an `AppError` (what failed inside, for the log and the detail line of a toast): English like the operating system's own error text; the sentence around it is translated";
const FORMAT_NAME: &str = "a file-format or unit name, the same in every language";
const FILE_NAME: &str =
    "a file name or an extension: part of what the product writes, not a sentence";

/// (file under `src/`, the literals, why they may stay).
const EXEMPT: &[(&str, &[&str], &str)] = &[
    ("app/persist.rs", &["error"], PLACEHOLDER),
    ("app/state.rs", &["error"], PLACEHOLDER),
    ("app/toast.rs", &["detected", "named"], PLACEHOLDER),
    (
        "app/update/settings.rs",
        &["size", "input", "min", "max"],
        PLACEHOLDER,
    ),
    ("ui/drop_zone.rs", &["count"], PLACEHOLDER),
    ("ui/settings_page.rs", &["theme"], PLACEHOLDER),
    (
        "native_store.rs",
        &[
            "settings I/O error: {e}",
            "settings serialize error: {e}",
            "settings deserialize error: {e}",
        ],
        "`Debug` and `Display` of the settings store's error: shown only inside the detail of a translated toast",
    ),
    (
        "portal.rs",
        &[
            "zenity",
            "/.flatpak-info",
            "org.freedesktop.portal.Desktop",
            "org.freedesktop.impl.portal.desktop.",
        ],
        "operating-system, environment and D-Bus names used to detect a file dialog (RFC 031): never shown",
    ),
    (
        "result.rs",
        &["{} B", "{:.1} KB", "{} KB", "{:.1} MB"],
        "a size with its unit (B, KB, MB): units are the same in every language",
    ),
    (
        "result.rs",
        &["PNG", "ICO", "SVG", "HTML", "JSON"],
        FORMAT_NAME,
    ),
    (
        "shortcuts.rs",
        &["Cmd", "Ctrl"],
        "the names of the modifier keys as the keyboard prints them",
    ),
    (
        "ui/settings_page.rs",
        &["{cmd}+O", "{cmd}+S", "Esc", "{cmd}+1 / 2 / 3", "F1"],
        "key chords as the keyboard prints them; the sentence beside each is a `MessageKey`",
    ),
    (
        "task_queue.rs",
        &["png", "svg", "webp", "jpg", "jpeg", "zip"],
        "file extensions of the dialog's filters",
    ),
    ("task_queue.rs", &["ZIP"], FORMAT_NAME),
    (
        "task_queue.rs",
        &["favicon-bundle.zip"],
        "the default file name of the bundle: a name the product gives its output, as `favicon.ico` is",
    ),
    (
        "task_queue.rs",
        &[
            ".ico",
            ".svg",
            ".png",
            ".html",
            ".webmanifest",
            ".json",
            "mono",
        ],
        FILE_NAME,
    ),
    (
        "task_queue.rs",
        &["preview", "conversion", "{label} task failed: {e}"],
        "the label of a background task in the detail of an error (`AppError`): technical",
    ),
    (
        "task_queue.rs",
        &[
            "artifact path is not valid UTF-8: {}",
            "artifact path is not a plain relative path: {}",
            "artifact path is empty",
            "failed to write {name}: {e}",
            "zip extra field: {}",
            "zip start_file: {}",
            "zip write_all: {}",
            "zip finish: {}",
            "failed to write zip file: {}",
            "<unnamed>",
        ],
        DETAIL,
    ),
    (
        "task_queue.rs",
        &["(unknown)"],
        "the file name of a result item whose path has none: the exporter always names its files, so this is a fallback that cannot occur",
    ),
    (
        "ui/advanced_drawer/manifest.rs",
        &["#RRGGBB"],
        "the notation of a colour value (a pattern to type), not a word; it is the same in every language",
    ),
    (
        "ui/preview_panel/mockups.rs",
        &[
            "logolig.example.com",
            "https://logolig.example.com/",
            "Logolig",
        ],
        "the mock-ups depict another application (a browser tab, a phone's home screen): the address and the app name drawn in them are part of the picture; the captions are `MessageKey`s",
    ),
    (
        "focus.rs",
        &["disclosure-advanced-header", "disclosure-preview-toggle"],
        "widget ids of the two disclosure controls: identifiers, never drawn",
    ),
    (
        "ui/theme.rs",
        &["Logolig Light", "Logolig Dark"],
        "names of the two iced themes: identifiers, never drawn",
    ),
    (
        "window.rs",
        &["io.github.nabbisen.Logolig"],
        "an operating-system name in a `cfg` and the application id: identifiers",
    ),
];

/// Source files of the application that are not tests.
fn sources() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("read the source folder") {
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
            {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut out);
    out.sort();
    out
}

/// The part of a file that is not an inline test module (`#[cfg(test)]` followed by `mod x {`).
pub(super) fn without_inline_tests(src: &str) -> &str {
    let lines: Vec<&str> = src.lines().collect();
    let mut offset = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "#[cfg(test)]"
            && lines
                .get(i + 1)
                .is_some_and(|n| n.trim_start().starts_with("mod ") && n.trim_end().ends_with('{'))
        {
            return &src[..offset];
        }
        offset += line.len() + 1;
    }
    src
}

/// Does the literal say anything in words? Escapes (`\n`, `\"`, `\u{..}`) are not words.
fn has_letters(text: &str) -> bool {
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            chars.next();
        } else if c.is_alphabetic() {
            return true;
        }
    }
    false
}

/// The literals of `src` that are neither a call form of `CALLS` nor free of letters.
fn candidates(src: &str) -> Vec<Literal> {
    scan(without_inline_tests(src))
        .literals
        .into_iter()
        .filter(|l| has_letters(&l.text))
        .filter(|l| {
            let before = l.before.trim_end();
            !CALLS.iter().any(|c| before.ends_with(c))
        })
        .collect()
}

fn is_exempt(file: &str, literal: &str) -> bool {
    EXEMPT
        .iter()
        .any(|(f, lits, _)| *f == file && lits.contains(&literal))
}

/// Which literals of `src` (the file `file`) are neither exempt nor a call form: `line: text`.
fn offenders(file: &str, src: &str) -> Vec<String> {
    candidates(src)
        .into_iter()
        .filter(|l| !is_exempt(file, &l.text))
        .map(|l| format!("{file}:{}: {:?}", l.line, l.text))
        .collect()
}

fn relative(path: &Path) -> String {
    path.strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
        .expect("under src")
        .to_string_lossy()
        .replace('\\', "/")
}

/// The real scan: no alphabetic literal reaches the interface outside the named list.
#[test]
fn no_hardcoded_user_visible_text() {
    let mut found = Vec::new();
    for path in sources() {
        let src = std::fs::read_to_string(&path).expect("a source file");
        found.extend(offenders(&relative(&path), &src));
    }
    assert!(
        found.is_empty(),
        "text that may reach the screen is written in the code (make it a MessageKey in en.toml and ja.toml, \
         or, if it is not text a user reads, list it in EXEMPT with the reason):\n{}",
        found.join("\n")
    );
}

/// The list stays honest: every literal it names is still in the file it names.
#[test]
fn every_exemption_is_still_needed() {
    let mut stale = Vec::new();
    for (file, literals, reason) in EXEMPT {
        assert!(!reason.is_empty(), "{file}: an exemption needs a reason");
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file);
        let src = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{file} exists"));
        let present: Vec<String> = candidates(&src).into_iter().map(|l| l.text).collect();
        for literal in *literals {
            if !present.iter().any(|p| p == literal) {
                stale.push(format!("{file}: {literal:?}"));
            }
        }
    }
    assert!(
        stale.is_empty(),
        "exemptions whose literal is gone (delete them):\n{}",
        stale.join("\n")
    );
}

/// The guard can fail: a planted literal is reported; a key, an exempt literal, a call form, a
/// comment and a string without letters are not.
#[test]
fn the_scan_reports_a_planted_literal_and_only_that() {
    let planted = r#"
fn view() {
    let a = text("Hello");                      // a sentence in the code
    let b = text(t.t(MessageKey::AppTitle));    // a key
    let c = text("{:.1} KB");                   // listed for result.rs
    let d = foo.expect("an invariant");         // never shown
    let e = text("12 / 34");                    // no letters
    // text("in a comment")
}
#[cfg(test)]
mod tests {
    fn t() { text("in a test"); }
}
"#;
    assert_eq!(
        offenders("x.rs", planted),
        vec![r#"x.rs:3: "Hello""#, r#"x.rs:5: "{:.1} KB""#],
        "a sentence and a unit that this file does not list"
    );
    assert_eq!(
        offenders("result.rs", planted),
        vec![r#"result.rs:3: "Hello""#],
        "the unit is listed for result.rs, the sentence is not"
    );
    // A literal written back into the toast code is found (the K4 mutation, in miniature).
    let toast = r#"Toast::new(id, ToastIntent::Error, "Operation failed", body, msg)"#;
    assert_eq!(offenders("app/toast.rs", toast).len(), 1);
}
