//! How the "saved" message names the file (Flathub job F2).
//!
//! The application writes the file at the exact path the file chooser returned. Inside a Flatpak
//! the chooser is the document portal, and the path it returns is
//! `<XDG_RUNTIME_DIR>/doc/<document id>/<file name>` (or `.../doc/by-app/<app id>/<id>/<name>`): a
//! path that exists only inside the sandbox and that the user cannot find. The user has just chosen
//! the folder in the chooser, so for such a path the file name alone is true and enough; any other
//! path is shown in full, as before. The real host folder is not known to the sandboxed process
//! without asking the Documents portal again, which is parked.

use std::path::{Component, Path, PathBuf};

/// The text that tells the user where a file was saved: the file name when `path` lies under
/// `<runtime_dir>/doc/` (a document-portal path), the full path otherwise.
///
/// `runtime_dir` is `XDG_RUNTIME_DIR`, read by the caller: this function reads no environment. When it
/// is `None` or empty, the literal form `/run/user/<digits>/doc/` is used. A path without a file name
/// gives the full path. Linux only: on Windows and macOS the full path is returned unchanged.
pub(super) fn path_for_message(path: &Path, runtime_dir: Option<&Path>) -> String {
    let full = || path.display().to_string();
    if !cfg!(target_os = "linux") {
        return full();
    }
    let Some(name) = path.file_name() else {
        return full();
    };
    let in_portal = match runtime_dir.filter(|dir| !dir.as_os_str().is_empty()) {
        Some(dir) => path.starts_with(dir.join("doc")),
        None => has_literal_portal_prefix(path),
    };
    if in_portal {
        name.to_string_lossy().into_owned()
    } else {
        full()
    }
}

/// `/run/user/<digits>/doc/` as the start of `path`, component by component (so `docx` is not `doc`).
fn has_literal_portal_prefix(path: &Path) -> bool {
    let mut parts = path.components();
    fn normal(c: Option<Component<'_>>) -> Option<&str> {
        match c {
            Some(Component::Normal(part)) => part.to_str(),
            _ => None,
        }
    }
    parts.next() == Some(Component::RootDir)
        && normal(parts.next()) == Some("run")
        && normal(parts.next()) == Some("user")
        && normal(parts.next())
            .is_some_and(|uid| !uid.is_empty() && uid.bytes().all(|b| b.is_ascii_digit()))
        && normal(parts.next()) == Some("doc")
}

/// `XDG_RUNTIME_DIR`, if set and not empty. The one place the environment is read for the message.
pub(super) fn runtime_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
}

/// The text for a saved file's path, with the environment as it is now.
pub(super) fn saved_path_text(path: &Path) -> String {
    path_for_message(path, runtime_dir().as_deref())
}
