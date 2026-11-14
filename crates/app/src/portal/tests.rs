//! Tests of the dialog verdict (RFC 031): the pure rule, over every combination of the facts.

use super::{Facts, Verdict, dialog_is_missing, verdict};

use Verdict::{DialogAvailable as Open, NoDialog as Notice};

/// Every one of the 2^4 combinations, written out by hand: (portal, backends found, zenity,
/// sandboxed) and the verdict. The rule behind it: the notice only with positive evidence that no
/// dialog can appear: no `zenity`, and either no portal or (outside a sandbox) no backend.
const TABLE: [(bool, bool, bool, bool, Verdict); 16] = [
    // zenity present: `rfd` falls back to it, a dialog can always appear.
    (true, true, true, false, Open),
    (true, true, true, true, Open),
    (true, false, true, false, Open),
    (true, false, true, true, Open),
    (false, true, true, false, Open),
    (false, true, true, true, Open),
    (false, false, true, false, Open),
    (false, false, true, true, Open),
    // no zenity, no portal: nothing can answer, sandbox or not.
    (false, true, false, false, Notice),
    (false, true, false, true, Notice),
    (false, false, false, false, Notice),
    (false, false, false, true, Notice),
    // no zenity, a portal: it needs a backend, but a sandbox hides the backends.
    (true, true, false, false, Open),
    (true, true, false, true, Open),
    (true, false, false, false, Notice),
    (true, false, false, true, Open),
];

fn facts(portal: bool, has_backends: bool, zenity: bool, sandboxed: bool) -> Facts {
    Facts {
        portal,
        backends: if has_backends { 2 } else { 0 },
        zenity,
        sandboxed,
    }
}

#[test]
fn the_verdict_is_right_for_every_combination_of_the_facts() {
    for (portal, has_backends, zenity, sandboxed, expected) in TABLE {
        assert_eq!(
            verdict(&facts(portal, has_backends, zenity, sandboxed)),
            expected,
            "portal {portal}, backends {has_backends}, zenity {zenity}, sandboxed {sandboxed}"
        );
    }
    // The table holds every combination exactly once.
    let mut seen: Vec<_> = TABLE.iter().map(|r| (r.0, r.1, r.2, r.3)).collect();
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), 16);
}

/// The number of backends does not matter beyond zero or not.
#[test]
fn one_backend_is_as_good_as_many() {
    let one = Facts {
        backends: 1,
        ..facts(true, true, false, false)
    };
    let many = Facts { backends: 9, ..one };
    assert_eq!(verdict(&one), Open);
    assert_eq!(verdict(&many), Open);
}

/// A probe that failed or timed out ("unknown") never counts as "missing": the dialog request is
/// left unchanged. Only a known system with positive evidence replaces the dialog.
#[test]
fn an_unknown_answer_never_replaces_the_dialog() {
    assert!(!dialog_is_missing(None));
    assert!(!dialog_is_missing(Some(facts(true, true, false, false))));
    assert!(dialog_is_missing(Some(facts(false, false, false, false))));
    assert!(dialog_is_missing(Some(facts(true, false, false, false))));
    assert!(!dialog_is_missing(Some(facts(false, false, true, false))));
}

/// `zenity` on `PATH`: an executable file in one of the directories counts; a file without the
/// executable bit, a directory of that name, a missing file and an empty `PATH` do not.
#[cfg(unix)]
#[test]
fn zenity_is_found_only_as_an_executable_file_on_the_path() {
    use super::imp::on_path;
    use std::os::unix::fs::PermissionsExt;

    let root = std::env::temp_dir().join(format!("logolig-portal-test-{}", std::process::id()));
    let (with, without, dir_named) = (root.join("with"), root.join("without"), root.join("dir"));
    for d in [&with, &without, &dir_named] {
        std::fs::create_dir_all(d).unwrap();
    }
    let exe = with.join("zenity");
    std::fs::write(&exe, "#!/bin/sh\n").unwrap();
    std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755)).unwrap();
    let plain = without.join("zenity");
    std::fs::write(&plain, "not executable").unwrap();
    std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::create_dir_all(dir_named.join("zenity")).unwrap();

    let path = |dirs: &[&std::path::Path]| std::env::join_paths(dirs).unwrap();
    assert!(
        on_path("zenity", &path(&[&without, &with])),
        "found in the second directory"
    );
    assert!(!on_path("zenity", &path(&[&without])), "no executable bit");
    assert!(
        !on_path("zenity", &path(&[&dir_named])),
        "a directory is not a program"
    );
    assert!(
        !on_path("zenity", &path(&[&root.join("absent")])),
        "no such directory"
    );
    assert!(
        !on_path("zenity", std::ffi::OsStr::new("")),
        "an empty PATH"
    );

    let _ = std::fs::remove_dir_all(&root);
}
