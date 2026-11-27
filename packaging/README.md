# Packaging

What is here and how it is used. Nothing in this repository, and no CI job, publishes anything: publication is an owner action.

| Path | Purpose |
|---|---|
| `linux/io.github.nabbisen.Logolig.desktop` | Desktop entry; the file name is the application ID the window reports |
| `linux/icons/hicolor/` | Application icons for the hicolor icon theme (PNG 32, 48, 64, 128, 256 and 512) |
| `icons/logolig.png` | Master artwork; every icon is generated from it (see `icons/README.md`) |
| `windows/` | Microsoft Store manifest and package assets (see "Windows (Microsoft Store)") |
| `linux/io.github.nabbisen.Logolig.metainfo.xml` | AppStream MetaInfo (Flathub) |
| `../scripts/appx-version.sh` | Checks (and, on request, updates) the Store manifest version against the workspace version |
| `../scripts/metainfo-version.sh` | Checks that the newest MetaInfo release is the workspace version |

## Linux and Flathub

Linux distribution is through **Flathub** (RFC 023). The Flatpak manifest and the submission are the owner's and are not in this repository. What lives here are the upstream files a Flatpak build reads, all under `linux/`:

- `io.github.nabbisen.Logolig.desktop`, the desktop entry (Keywords in English and Japanese; `StartupWMClass` equal to the window's application id; no `MimeType` and no `%f`, because the program takes no command-line arguments);
- `io.github.nabbisen.Logolig.metainfo.xml`, the AppStream MetaInfo Flathub lists (description, licences, URLs, one screenshot, the content rating and the release entries);
- `icons/hicolor/`, the icons (PNG 32 to 512, named by the application id; the logo's terms are in `icons/README.md`).

**Validation.** The workflow `.github/workflows/flatpak-upstream.yml` runs `appstreamcli validate --strict` on the MetaInfo (errors and warnings fail it, as Flathub's) and `desktop-file-validate` on the desktop entry, for every change under `packaging/linux/`. It builds no Flatpak, reads no secret and fetches nothing but the validators. It does not check that the screenshot URL answers (`--no-net`): when the screenshot changes, check the URL by hand. The screenshot link is pinned to a **tag**, as Flathub asks for a tag or a commit and not a branch; a changed screenshot gets a link to a tag or a commit that contains it.

**The MetaInfo version rule.** Every release adds a `<release version="X.Y.Z" date="YYYY-MM-DD">` entry at the top of `<releases>`, and the newest entry is the workspace version (`version` under `[workspace.package]` in the root `Cargo.toml`). `scripts/metainfo-version.sh check` fails otherwise, and says which entry to add; CI runs it as the step "MetaInfo version" of the `fmt` job, so a version bump that forgets the MetaInfo fails on that commit. There is no `update`: an entry needs a date and a description that only a person can write. The release notes are the source of the entry's text.

There is no other Linux package in this repository (an AUR package that was prepared earlier was removed: the project does not publish there; the history keeps it).

## Windows (Microsoft Store)

**The version rule.** The `Version` of the `<Identity>` element in `windows/AppxManifest.xml` is the workspace version **plus a fourth part `.0`**: workspace `1.30.0` is Store `1.30.0.0`. The fourth part belongs to the Store and stays `0`. The one source is `version` under `[workspace.package]` in the root `Cargo.toml`. `scripts/appx-version.sh check` fails (and names the fix) when the manifest differs; `scripts/appx-version.sh update` rewrites that one attribute and nothing else. CI runs the check as the step "Store manifest version" of the `fmt` job, so a version bump that forgets the manifest fails on that commit. Change the version in the manifest only through the script; the other attributes (`Name`, `Publisher`, the tile paths, `MinVersion`, `MaxVersionTested`) are not derived and are never touched by it.

**What the package contains:** the executable (`logolig.exe`, from `cargo build --release`), the four tile images in `windows/Assets/` (`StoreLogo.png`, `Square44x44Logo.png`, `Square150x150Logo.png`, `Wide310x150Logo.png`, made from the master artwork, see `icons/README.md`) and the manifest. **The Store listing images are not part of the package:** the images shown on the product page (the app tile icon, the box art, the screenshots) are separate uploads in Partner Center and have to be replaced there by hand whenever the logo changes.

**The Windows package workflow** (`.github/workflows/windows-package.yml`, RFC 037 phase B). On a Windows runner it prints what the runner has, checks the manifest version rule, builds `logolig.exe` in release mode, **runs the whole test suite** (the step is strict: a failing test fails the job, and the repository checks out with LF line endings on every platform, by `.gitattributes`), finds `makeappx.exe` in the Windows SDK of the runner, stages `logolig.exe`, `Assets\` and `AppxManifest.xml` of `packaging/windows/`, and packs **`logolig-<version>.0-unsigned.msix`** (with `makeappx`'s validation on), which it uploads as a workflow artifact kept for 14 days. It is **not part of `ci.yml`** and nothing requires it: it runs by hand (`workflow_dispatch`), when a version tag (three numbers) is pushed, and when a push to `main` changes only the workflow file itself. It reads no secret and no certificate and submits nothing.

**To get the package:** open the run in the repository's **Actions** tab (workflow "Windows package") and download the artifact `logolig-<version>.0-unsigned.msix` at the bottom of the run page (GitHub wraps it in a ZIP). **It is unsigned on purpose**: Partner Center signs a package on ingestion, so it cannot be installed locally and cannot be tested by installing it; the package's validation by `makeappx` and the Store's own certification (done by Partner Center) are the only checks. The Windows build and the first Windows tests are evidence of what holds on Windows, **not** a claim that Windows is supported: the project's checks in a real window were made on Linux.

**Submitting is still manual** (download the artifact, upload it in Partner Center). Submission from CI is RFC 037 phase C and is not built; it would be the owner's decision, with the owner's credentials, and nothing in this repository or CI reads a Store credential or submits anything.

## What the application needs on Linux

The binary links only `glibc` and `libgcc`; the windowing, keyboard and graphics libraries are loaded at run time, so a package or a runtime has to provide them; text needs a system font; the file chooser needs `xdg-desktop-portal` **and** a portal backend (without a portal backend neither "Choose file…" nor "Save" opens a window, and dragging a file onto the window still works; since RFC 031 the application shows one message when it can tell that no dialog can appear, and stays silent when it cannot be sure). **Known limit:** a portal that accepts a dialog and never answers leaves the application waiting (no timeout, by design: a user may leave a dialog open for minutes); if a dialog never appears and never answers, restart the application.
