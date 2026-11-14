# JOURNAL — windows-app

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. `/lcd:resume` reads MAP.md + this block + DECISIONS headers and nothing else.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** The release workflow builds the command as a `.zip` and the app as an installer for Windows x64, and the window is right on Windows, so the owner can try a dry-run build on a real PC (step 5 of the release roadmap, D-072).
- **Next action:** the owner runs "The owner's test" below on a real PC and reports. Then S7: fix what the report finds, fold AC-214..AC-216 into `SPEC.md`, tick step 5 in `docs/release-roadmap.md`, archive this work-item.
- **Branch:** feat/release-windows  ·  **Updated:** 2026-11-14 13:58

## STEPS
- [x] S1 — recon: Tauri Windows bundle targets, WebView2, what the macOS title bar keys do on Windows
- [x] S2 — temporary probe workflow: build both Windows legs, install the app, screenshot it
- [x] S3 — the window on Windows: tests first (AC-214..AC-216), then the fixes the screenshots called for
- [x] S4 — `release.yml`: the `windows-x86_64` command leg and the `app-windows` job, both in `release`
- [x] S5 — PR, dry run green, the `release` artifact read: names and checksums
- [x] S6 — docs: CONTRIBUTING, the workflow's header; the probe workflow deleted
- [ ] S7 — after the owner's report from a real PC: closeout, step 5 ticked in `docs/release-roadmap.md`  ← next

## DECISIONS (this work-item)
- NSIS, not MSI: it installs for the current user with no administrator prompt, takes any version string (an `.msi` refuses a pre-release like `-rc.1`), and needs nothing on the runner (WiX needs the VBSCRIPT feature Microsoft is retiring).
- The command links the C runtime statically (`-C target-feature=+crt-static`, in the workflow's Build step): a clean Windows has no `vcruntime140.dll`, and a single `.exe` in a `.zip` has no installer to bring it. The job fails if the `.exe` names that DLL.
- The app needs no flag: Tauri links the runtime statically by default since tauri-cli 2.12 (`build.windows.staticVCRuntime`). The job checks that too.
- WebView2 stays on Tauri's default (the installer downloads the bootstrapper if the runtime is missing): Windows 11 and Windows 10 from 1803 ship it, and embedding it adds 127 MB.
- The installed program is `stidcart-gui.exe` (the Cargo package name), under the product name "stidcart" in the Start menu. Left as is: renaming it means `mainBinaryName`, which also renames the binary inside the macOS bundle.
- Platform differences live in `gui/src/lib/platform.ts` (the page) and behind one `WINDOWS` constant in `gui/src-tauri/src/menu.rs` (the native menu). Only a Windows webview is treated as Windows; Linux keeps the macOS drawing it had, which is its own question (see the report).
- Menu shortcuts on Windows are forwarded by the page (`forwardMenuKeys` → the `menu_key` command → the same path as a click). Seen on the runner: with the webview holding the keyboard, no accelerator fired; after the change each fires once.

## OPEN QUESTIONS
- none blocking. For the owner: whether Linux should get the same treatment (inset, ⌘ glyphs, app menu), and whether Redo should also answer to Ctrl+Y on Windows.

## EDIT BOUNDARY (paths this work may touch)
- `.github/workflows/release.yml`
- `.github/workflows/windows-probe.yml`
- `gui/src/App.svelte`
- `gui/src/main.ts`
- `gui/src/lib/platform.ts`
- `gui/src/lib/platform.test.ts`
- `gui/src/lib/host.ts`
- `gui/src/lib/EmptyState.svelte`
- `gui/src/lib/Toolbar.svelte`
- `gui/src/lib/Listing.svelte`
- `gui/src-tauri/src/menu.rs`
- `gui/src-tauri/src/lib.rs`
- `docs/lcd/work/windows-app/JOURNAL.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/SPEC.md`
- `docs/release-roadmap.md`
- `.claude/rules/lcd-conventions.md`
- `CONTRIBUTING.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-214** (surfaces: RENDER): On Windows a shortcut is written with Ctrl (`Ctrl+O`, `Ctrl+Shift+N`) wherever the window names one: the empty state's two buttons, the New Folder button's tooltip, the list's context menu. On a Mac it stays `⌘O`, `⇧⌘N`.

**AC-215** (surfaces: RENDER): On Windows the page's title bar keeps no room for traffic lights and is not a drag region; the system's title bar above it moves the window. On a Mac both stay.

**AC-216** (surfaces: RENDER): On Windows a menu shortcut typed while the page has the keyboard does what a click on its menu item does, once. On a Mac the page forwards nothing.

## The owner's test

1. **Get the files.** On the PR's checks, open the `Release` run, then its `release` artifact
   (or `gh run download <run id> -n release` on the PC). It unzips to the installer
   `stidcart-<version>-windows-x86_64-setup.exe`, the command's
   `stidcart-cli-<version>-windows-x86_64.zip`, the other platforms' files and `SHA256SUMS`.
   `Get-FileHash <file>` in PowerShell should match the line in `SHA256SUMS`.
2. **Install.** Run the installer. SmartScreen says "Windows protected your PC" because it is not
   signed: "More info", then "Run anyway". It should not ask for an administrator password. Note
   whether it downloads anything (WebView2, only on a PC that lacks it).
3. **Look at the window.** Start stidcart from the Start menu. One title bar, the system's. Under it
   File, Edit, View, Window, Help. The mark and "stidcart" sit at the left edge with no gap. The two
   buttons say `Ctrl+O` and `Ctrl+N`. Help ▸ About stidcart shows the version. Try light and dark
   mode in Windows settings and say if anything is unreadable.
4. **Open and edit.** `Ctrl+O`, pick an EasyFlash `.crt`. `Ctrl+Shift+N`: exactly one new folder,
   its name ready to type. `Ctrl+Z` takes it away, `Ctrl+Shift+Z` brings it back. Rename an entry
   and, while typing, `Ctrl+Z` undoes the typing, not the cart.
5. **Add files two ways.** "Add…" and pick a `.prg`; then drag another `.prg` from Explorer onto
   the window. Drag a row onto a folder. With no cart open, drag a `.crt` from Explorer to open it.
6. **Save.** `Ctrl+S`, then `Ctrl+Shift+S` to a new name in a folder with a space or an accent in
   its path. With unsaved changes, close the window with the X, with Alt+F4 and with `Ctrl+Q`:
   each asks Save, Don't Save or Cancel, and Cancel keeps the window.
7. **The hex window.** Select an entry, `Ctrl+Shift+H`. Click entries in the main window: the hex
   window follows. Close the main window: the hex window goes too.
8. **The command.** Unzip the `.zip`, and in PowerShell: `.\stidcart.exe build -o t.crt some.prg`,
   `.\stidcart.exe list t.crt`, then open `t.crt` in the app and in VICE or on the cartridge if one
   is at hand. A cart saved by the app should `list` the same.
9. **Uninstall.** Settings ▸ Apps ▸ Installed apps ▸ stidcart ▸ Uninstall. The Start menu entry and
   `%LOCALAPPDATA%\stidcart` should be gone.

Worth a line in the report even if fine: how the text looks at 125% or 150% scaling, and whether
anything flashed white at start-up.

## LOG (append-only; not read on resume)

- 2026-11-14 12:55 — triage: Standard. Recon dispatched (Tauri 2 Windows bundling, WebView2, title bar keys, menu on Windows, static CRT).
- 2026-11-14 13:00 — probe workflow pushed: `windows-probe.yml`, on this branch only, so the macOS jobs of `release.yml` do not run for every look at the window.
- 2026-11-14 13:05 — recon digest: `titleBarStyle`, `hiddenTitle`, `trafficLightPosition` are read only under `cfg(target_os = "macos")`, so Windows gets a native title bar and the menu bar inside the window. muda on Windows: `hide` hides the window with no way back, `hide_others` does nothing, `about` does nothing without metadata. `data-tauri-drag-region` drags and, on a double-click, maximizes, title bar or not. NSIS and WiX are both downloaded by the bundler; NSIS installs per user by default. Tauri links the VC runtime statically by default.
- 2026-11-14 13:25 — probe run 43641418627, the window before any change: a native title bar, the macOS app menu ("stidcart": About, Hide, Hide Others, Quit) first in the bar, 92 px of empty title bar before the mark, `⌘O` and `⌘N` on the buttons. Screenshots work on the runner (`CopyFromScreen`), at 1280x1024.
- 2026-11-14 13:25 — the built app binary is `stidcart-gui.exe`, not `stidcart.exe`; the installer is 2.2 MB and installs to `%LOCALAPPDATA%\stidcart`.
- 2026-11-14 13:35 — run 43642939835, after the first fixes: inset gone, `Ctrl+O`/`Ctrl+N`, File/Edit/View/Window/Help. But no key did anything. `SendKeys` needs the window focused by a real click (`AppActivate` is refused); with that, typing reached the page and the shortcuts still did nothing.
- 2026-11-14 13:47 — run 43644205818: keyboard focus is always `Chrome_WidgetWin_1`, the webview, even after a click on the title bar. Every menu item works by click (New Folder, About with the version, Hex Viewer, Exit with the unsaved-changes question). None works by its shortcut, by `SendKeys` or by `keybd_event`. Tauri translates accelerators only for messages its own loop sees (`app.rs`, `msg_hook`), and the webview's keys never get there.
- 2026-11-14 13:56 — fix: the page forwards the shortcuts on Windows (AC-216). Run 43644894291: Ctrl+O opens the dialog and a fixture cart; Ctrl+Shift+N adds one folder; Ctrl+Z, Ctrl+Shift+Z, Ctrl+Shift+S, Ctrl+Shift+H work; Ctrl+Q from the hex window asks about unsaved changes.
- 2026-11-14 13:38 — merged `origin/main` (the Linux app leg, #86): both jobs kept, `release` needs both.
- 2026-11-14 13:58 — not seen by anyone: drag and drop from Explorer, dark mode, display scaling, SmartScreen, a PC without WebView2, the save dialog's result on disk. These are the owner's test.
