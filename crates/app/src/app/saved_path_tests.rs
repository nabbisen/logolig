//! `update` tests: the "saved" message names a place the user can find (Flathub job F2).
//!
//! Inside a Flatpak the file chooser is the portal, and the path it returns is a document-portal
//! path (`<runtime dir>/doc/<id>/<name>`) that means nothing to the user. The message then shows the
//! file name only; elsewhere it shows the full path as it always did.

use super::*;

use std::path::{Path, PathBuf};

/// The runtime directory the handlers will read (`XDG_RUNTIME_DIR`), or the usual one when it is not
/// set: a portal path built under it is a portal path in either branch of the check.
fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|d| !d.is_empty())
        .map_or_else(|| PathBuf::from("/run/user/1000"), PathBuf::from)
}

fn body_after(message: Message) -> String {
    let mut state = AppState::default();
    let _ = update(&mut state, message);
    assert_eq!(state.toasts.len(), 1, "one success toast");
    state.toasts[0].message.clone()
}

#[cfg(target_os = "linux")]
#[test]
fn a_portal_path_shows_the_file_name_after_one_download() {
    let path = runtime_dir().join("doc/ab12cd/favicon-32.png");
    assert_eq!(
        body_after(Message::DownloadOneCompleted(Ok(path))),
        "favicon-32.png"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn a_portal_path_shows_the_file_name_after_download_all() {
    let path = runtime_dir().join("doc/ab12cd/favicons.zip");
    assert_eq!(
        body_after(Message::DownloadAllCompleted(Ok(path))),
        "favicons.zip"
    );
}

#[test]
fn an_ordinary_path_is_shown_in_full_as_before() {
    let path = Path::new("/home/user/Downloads/favicons.zip").to_path_buf();
    assert_eq!(
        body_after(Message::DownloadAllCompleted(Ok(path.clone()))),
        path.display().to_string()
    );
    assert_eq!(
        body_after(Message::DownloadOneCompleted(Ok(path.clone()))),
        path.display().to_string()
    );
}

// ---- `path_for_message` itself (pure: the runtime directory is an argument) ----

mod pure {
    use super::super::saved_path::path_for_message;
    use std::path::Path;

    fn text(path: &str, runtime_dir: Option<&str>) -> String {
        path_for_message(Path::new(path), runtime_dir.map(Path::new))
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_portal_path_under_the_given_runtime_directory_gives_the_name() {
        assert_eq!(
            text(
                "/run/user/1000/doc/ab12cd/favicons.zip",
                Some("/run/user/1000")
            ),
            "favicons.zip"
        );
        assert_eq!(text("/tmp/rt/doc/x/f.png", Some("/tmp/rt")), "f.png");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn the_by_app_form_gives_the_name() {
        let path = "/run/user/1000/doc/by-app/io.github.nabbisen.Logolig/ab12/f.png";
        assert_eq!(text(path, Some("/run/user/1000")), "f.png");
        assert_eq!(text(path, None), "f.png");
    }

    /// With no runtime directory (unset, or empty) the literal `/run/user/<digits>/doc/` is the test.
    #[cfg(target_os = "linux")]
    #[test]
    fn without_a_runtime_directory_the_literal_prefix_is_used() {
        for dir in [None, Some("")] {
            assert_eq!(text("/run/user/1000/doc/ab12cd/f.png", dir), "f.png");
            assert_eq!(text("/run/user/0/doc/ab/f.png", dir), "f.png");
            // Not digits, not `doc`, or not that tree: the full path stays.
            for other in [
                "/run/user/abc/doc/ab/f.png",
                "/run/user//doc/ab/f.png",
                "/run/user/1000/docx/ab/f.png",
                "/home/u/doc/x/f.png",
                "/run/user/1000/f.png",
                "/tmp/rt/doc/x/f.png",
            ] {
                assert_eq!(text(other, dir), other, "{other}");
            }
        }
    }

    /// The prefix is matched by whole path components: `docx` and `doc-x` are not `doc`, and another
    /// runtime directory is not this one.
    #[test]
    fn near_misses_keep_the_full_path() {
        for (path, dir) in [
            ("/run/user/1000/docx/ab/f.png", Some("/run/user/1000")),
            ("/run/user/1000/doc-x/ab/f.png", Some("/run/user/1000")),
            ("/home/u/doc/x/f.png", Some("/run/user/1000")),
            ("/run/user/1001/doc/ab/f.png", Some("/run/user/1000")),
            ("/home/u/Downloads/favicons.zip", Some("/run/user/1000")),
        ] {
            assert_eq!(text(path, dir), path, "{path}");
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn names_with_spaces_and_non_ascii_characters_survive() {
        assert_eq!(
            text(
                "/run/user/1000/doc/ab12cd/ロゴ 最終版 (1).zip",
                Some("/run/user/1000")
            ),
            "ロゴ 最終版 (1).zip"
        );
        assert_eq!(
            text("/run/user/1000/doc/ab12cd/my favicon.ico", None),
            "my favicon.ico"
        );
    }

    /// A path with no file name (it cannot come from the chooser) is shown whole.
    #[test]
    fn a_path_without_a_file_name_is_shown_whole() {
        assert_eq!(text("/", Some("/run/user/1000")), "/");
        assert_eq!(
            text("/run/user/1000/doc/ab/..", Some("/run/user/1000")),
            "/run/user/1000/doc/ab/.."
        );
    }

    /// Outside Linux the full path is returned unchanged, whatever it looks like.
    #[cfg(not(target_os = "linux"))]
    #[test]
    fn off_linux_the_full_path_is_returned_unchanged() {
        assert_eq!(
            text("/run/user/1000/doc/ab12cd/f.png", Some("/run/user/1000")),
            "/run/user/1000/doc/ab12cd/f.png"
        );
    }
}
