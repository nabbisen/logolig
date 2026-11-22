# Packaging

What is here and how it is used. Nothing in this repository, and no CI job, publishes anything: publication is an owner action.

| Path | Purpose |
|---|---|
| `linux/io.github.nabbisen.Logolig.desktop` | Desktop entry; the file name is the application ID the window reports |
| `linux/icons/hicolor/` | Application icons for the hicolor icon theme (PNG 32, 48, 64, 128, 256 and 512) |
| `icons/logolig.png` | Master artwork; every icon is generated from it (see `icons/README.md`) |
| `windows/` | Microsoft Store manifest and package assets (see "Windows (Microsoft Store)") |
| `../scripts/appx-version.sh` | Checks (and, on request, updates) the Store manifest version against the workspace version |

## Linux and Flathub

Linux distribution is through **Flathub** (RFC 023). The Flatpak manifest and the submission are the owner's and are not in this repository. What lives here are the upstream files a Flatpak build reads: the desktop entry and the icons under `linux/`. There is no other Linux package in this repository (the AUR package that was prepared earlier was removed: the project does not publish there; the history keeps it).

## Windows (Microsoft Store)

**The version rule.** The `Version` of the `<Identity>` element in `windows/AppxManifest.xml` is the workspace version **plus a fourth part `.0`**: workspace `1.28.0` is Store `1.28.0.0`. The fourth part belongs to the Store and stays `0`. The one source is `version` under `[workspace.package]` in the root `Cargo.toml`. `scripts/appx-version.sh check` fails (and names the fix) when the manifest differs; `scripts/appx-version.sh update` rewrites that one attribute and nothing else. CI runs the check as the step "Store manifest version" of the `fmt` job, so a version bump that forgets the manifest fails on that commit. Change the version in the manifest only through the script; the other attributes (`Name`, `Publisher`, the tile paths, `MinVersion`, `MaxVersionTested`) are not derived and are never touched by it.

**What the package contains:** the executable (`logolig.exe`, from `cargo build --release`), the four tile images in `windows/Assets/` (`StoreLogo.png`, `Square44x44Logo.png`, `Square150x150Logo.png`, `Wide310x150Logo.png`, made from the master artwork, see `icons/README.md`) and the manifest. **The Store listing images are not part of the package:** the images shown on the product page (the app tile icon, the box art, the screenshots) are separate uploads in Partner Center and have to be replaced there by hand whenever the logo changes.

**Building and submitting are still manual** (build the executable, pack it, upload it in Partner Center). A Windows build and an unsigned package in CI are RFC 037 phase B, and submission from CI is phase C (the owner's decision, with the owner's credentials); neither exists yet, and nothing in this repository or CI reads a Store credential or submits anything.

## What the application needs on Linux

The binary links only `glibc` and `libgcc`; the windowing, keyboard and graphics libraries are loaded at run time, so a package or a runtime has to provide them; text needs a system font; the file chooser needs `xdg-desktop-portal` **and** a portal backend (without a portal backend neither "Choose file…" nor "Save" opens a window, and dragging a file onto the window still works; since RFC 031 the application shows one message when it can tell that no dialog can appear, and stays silent when it cannot be sure). **Known limit:** a portal that accepts a dialog and never answers leaves the application waiting (no timeout, by design: a user may leave a dialog open for minutes); if a dialog never appears and never answers, restart the application.
