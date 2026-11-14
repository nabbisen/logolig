//! Can a file dialog appear at all? (RFC 031)
//!
//! `rfd` cannot tell a cancelled dialog from one that could not be shown: both come back as
//! `None`. On a system with no portal, or a portal without a backend, and no `zenity` program for
//! `rfd` to fall back to, "Choose file…" and "Save" therefore do nothing and say nothing. This
//! module asks, before a dialog is opened, whether there is **positive evidence that none can
//! appear**; the application then shows one notice instead of opening the dialog.
//!
//! The rule is conservative: **when in doubt, do nothing new.** A probe that fails or takes more
//! than two seconds returns `None` ("unknown"), and `None` never counts as "missing".

/// What the probe found out about the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Facts {
    /// `org.freedesktop.portal.Desktop` has an owner on the session bus, or the bus can start it.
    pub(crate) portal: bool,
    /// How many `org.freedesktop.impl.portal.desktop.*` backends are owned or activatable.
    pub(crate) backends: usize,
    /// A `zenity` executable is on `PATH` (`rfd` falls back to it when the portal call fails).
    pub(crate) zenity: bool,
    /// The application runs in a Flatpak or Snap sandbox: the bus then shows the portal but not
    /// its backends, so the backend count says nothing.
    pub(crate) sandboxed: bool,
}

/// What to do about the facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Open the dialog as always.
    DialogAvailable,
    /// Show the notice and do not open the dialog.
    NoDialog,
}

/// `NoDialog` only with positive evidence: no `zenity`, and either no portal or (outside a
/// sandbox) a portal with no backend.
pub(crate) fn verdict(facts: &Facts) -> Verdict {
    let no_portal = !facts.portal;
    let no_backend = !facts.sandboxed && facts.backends == 0;
    if !facts.zenity && (no_portal || no_backend) {
        Verdict::NoDialog
    } else {
        Verdict::DialogAvailable
    }
}

/// Should the dialog be replaced by the notice? An unknown (`None`) answer never says yes.
pub(crate) fn dialog_is_missing(facts: Option<Facts>) -> bool {
    facts.is_some_and(|facts| verdict(&facts) == Verdict::NoDialog)
}

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod imp {
    use super::Facts;
    use std::path::Path;

    /// Is an executable called `name` in one of the directories of the `path` variable?
    pub(super) fn on_path(name: &str, path: &std::ffi::OsStr) -> bool {
        use std::os::unix::fs::PermissionsExt;
        std::env::split_paths(path).any(|dir| {
            std::fs::metadata(Path::new(&dir).join(name))
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
    }

    /// Flatpak (`/.flatpak-info`) or Snap (`SNAP` set).
    fn is_sandboxed() -> bool {
        Path::new("/.flatpak-info").exists() || std::env::var_os("SNAP").is_some()
    }

    /// The facts of this system, or `None` when they cannot be found out (no session bus, an error,
    /// no answer within two seconds). Async: it runs inside the dialog's task, never on the UI thread.
    pub(super) async fn probe() -> Option<Facts> {
        use ashpd::zbus::{self, fdo::DBusProxy};
        use std::time::Duration;

        const PORTAL: &str = "org.freedesktop.portal.Desktop";
        const BACKEND_PREFIX: &str = "org.freedesktop.impl.portal.desktop.";

        let ask = async {
            let connection = zbus::Connection::session().await.ok()?;
            let bus = DBusProxy::new(&connection).await.ok()?;
            let owned = bus.list_names().await.ok()?;
            let activatable = bus.list_activatable_names().await.ok()?;
            let names: std::collections::BTreeSet<&str> = owned
                .iter()
                .chain(activatable.iter())
                .map(|n| n.as_str())
                .collect();
            Some((
                names.contains(PORTAL),
                names
                    .iter()
                    .filter(|n| n.starts_with(BACKEND_PREFIX))
                    .count(),
            ))
        };
        let (portal, backends) = tokio::time::timeout(Duration::from_secs(2), ask)
            .await
            .ok()??;
        Some(Facts {
            portal,
            backends,
            zenity: std::env::var_os("PATH").is_some_and(|path| on_path("zenity", &path)),
            sandboxed: is_sandboxed(),
        })
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use imp::probe;

/// Other platforms use no portal: the answer is always "unknown".
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
async fn probe() -> Option<Facts> {
    None
}

/// For the dialog tasks: is there positive evidence that no dialog can appear?
pub(crate) async fn no_dialog_can_appear() -> bool {
    dialog_is_missing(probe().await)
}

#[cfg(test)]
mod tests;
