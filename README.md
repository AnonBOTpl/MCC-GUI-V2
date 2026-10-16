# MCC GUI V2

A local Windows 11 desktop app for running **several Minecraft accounts at once, each on one
or more servers**. An account is a login identity with its own MCC session; every
(account, server) pair is a separate
[Minecraft Console Client](https://mccteam.github.io/guide/usage.html) (MCC) process and a
tab. The app gives you one window for all of them: live coloured logs, input to one
connection or to all of them, and a login banner for Microsoft accounts.

MCC stays the engine — the app never implements the Minecraft protocol itself, and it
never bundles, downloads or patches MCC.

🇵🇱 Wersja polska: [README-PL.md](README-PL.md)

## Status

Work in progress.

| Stage | What it covers | Status |
|-------|----------------|--------|
| 0 | Scaffold, tooling, MCC research | done |
| 1 | Process manager (Rust) | done |
| 2 | Core UI: tabs, log, input | done — 200 lines/s keeps the UI at ~59 fps |
| 3 | Microsoft login banner and status handling | done |
| 4 | Persistence: accounts, settings, per-account servers | done |
| 5 | Log scroll polish | done (log filters are out of scope) |
| 6 | Quick actions | done |
| 7 | Language switch, final Polish translation, packaging | todo — see the note below |

**Stage 7** has its language switch, app icon and installer done; what is left is a
read-through of the Polish translation, which is why the row above is still `todo`.
Everything from stages 0-6 works.

**Known limitations right now:** the installer is unsigned, so Windows SmartScreen warns
before you run it.

## Requirements

- **Windows 11** (the only supported target).
- **Node.js 24+** and npm.
- **Rust 1.90+** with the MSVC toolchain (`rustup default stable-msvc`). The crate declares
  edition 2024.
- **WebView2 runtime** — preinstalled on Windows 11.
- **Minecraft Console Client** itself. It is a separate program you download and keep
  somewhere permanent; the app only points at `MinecraftClient.exe`.

## Getting started

```bash
npm install
npm run tauri dev
```

On the first launch the app opens its settings dialog, because it does not know where MCC
is yet:

1. Click **Browse** and pick your `MinecraftClient.exe`.
2. Click **Save**.

The path is stored and reused on later launches. Then press **Add account** in the header of
the accounts panel on the left, choose the type (Microsoft or offline), fill in the login
and confirm — the account appears in the panel and nothing connects yet. Add one or more
servers to it there (an address and, if you need it, a forced Minecraft version), then press
the green **Play** button next to a server to connect.

For development you can also skip the dialog and provide the executable through the
`MCC_EXECUTABLE` environment variable — a saved path always wins over it:

```bash
MCC_EXECUTABLE="C:/path/to/MinecraftClient.exe" npm run tauri dev
```

Close the app with the window's ✕ button. Closing it stops every MCC process it started
(they are tied to the app through a Windows Job Object), so no `MinecraftClient.exe`
should be left behind in Task Manager.

## Building a release

```bash
npm run tauri build
```

The executable and bundles land in `src-tauri/target/release/` and
`src-tauri/target/release/bundle/`.

### Installer

```bash
npm run tauri -- build --bundles nsis
```

This produces a per-user NSIS installer,
`src-tauri/target/release/bundle/nsis/MCC GUI V2_2.0.0_x64-setup.exe` (the version in the
name follows `version` in `src-tauri/tauri.conf.json`). It needs no administrator rights and
installs into the user's own profile.

The installer is **not code-signed**, so Windows shows a SmartScreen warning the first time
you run it — choose **More info** → **Run anyway**. A signing certificate is a paid purchase
and stays out of the MVP.

The installed build uses the same app identifier as a development build, so it reads the
same files under `%APPDATA%` (see below) — your accounts, settings and open tabs carry
over, and you do not sign in again.

## How it works

- **Rust `process_manager`** runs one `MinecraftClient.exe` per connection (account +
  server). An account has one working directory shared by all of its connections (one MCC
  session), and each connection gets its own generated `.ini` file; the process is started
  as `MinecraftClient.exe settings.ini BasicIO` so that output arrives as a clean line
  stream (with `§` colour codes kept).
- **stdout/stderr** are forwarded line by line to the UI, together with the account id,
  the stream and a timestamp. The Rust side never rewords MCC output.
- **The frontend** parses `§` and ANSI colour codes, classifies lines and keeps a bounded
  log buffer per connection, rendered as a virtualised list so the UI stays smooth at a few
  hundred lines per second.
- **Auto-scroll is per tab:** scrolling up in one connection pauses only that tab, and each
  tab keeps its own setting. Starting or restarting a connection follows its output again,
  because its log starts from scratch.
- **The only bridge** between Rust and the UI is the Tauri command/event contract
  described in `src/api/contract.ts`.
- **The app never spawns processes or touches the filesystem from the frontend** — all of
  that lives in Rust.
- **Quick actions** are preset buttons at the bottom of a connection: a thin handle expands a
  panel that shows at most three rows of buttons (anything more scrolls inside the panel, so
  the log keeps its space). A click sends the preset's text exactly as if you had typed it,
  with the same scope: the active connection, or every running one while the "Send to all"
  checkbox under the log is ticked. `Edit` opens the editor (add, rename, recolour, reorder by
  dragging the handle, delete, switch an action on or off — an action that is off disappears
  from the bar but stays in the editor), `Reload` reads `presets.json` again after you edit it
  by hand.

## Where your data lives

Everything is stored under `%APPDATA%\com.mccgui.v2\`:

| Path | Contents |
|------|----------|
| `settings.json` | App settings: MCC path, language, "send to all" delay, log buffer size, restore-tabs switch, server history |
| `accounts.json` | Saved accounts, each with its list of servers |
| `presets.json` | Quick action buttons: label, the exact line sent to MCC, optional colour and an on/off switch per action |
| `accounts\<id>\` | Account MCC working directory: one generated `.ini` per connection and MCC's own session cache |

The installed build and a development build share this directory (same identifier), and the
MCC path comes from `settings.json` — the `MCC_EXECUTABLE` variable is a development-only
convenience.

**No passwords or tokens are ever stored by the app.** The app supports Microsoft accounts
(no password; MCC uses a device-code login) and offline accounts (no password). MCC session
files stay inside the account folder and the app never reads, copies or exports them.

## Development commands

| Command | What it does |
|---------|--------------|
| `npm run tauri dev` | Run the app with hot reload |
| `npm run dev` | Frontend only (Vite dev server on port 1420) |
| `npm run build` | Type-check and build the frontend |
| `npm run tauri build` | Build the release executable and bundles |
| `npm run typecheck` | `vue-tsc --noEmit` |
| `npm run lint` | ESLint |
| `npm run test` | Vitest (unit tests) |
| `npm run test:watch` | Vitest in watch mode |
| `npm run check:i18n` | Fails when `en` and `pl` message files have different keys |

Rust-side checks:

```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

All of the above must pass before a change is considered finished.

## Languages

The interface is **English by default** and starts in English on first launch regardless
of the system locale. Polish is available from the language switch in the UI, and the
choice is remembered.

Every user-visible string goes through `vue-i18n`. `en` is the source of truth and the
fallback; `pl` must contain every key, which `npm run check:i18n` enforces. Rust never sends
human-readable text to the UI — it sends stable error codes with parameters, and the
frontend translates them. Raw MCC and server output is shown exactly as received and is
never translated.

## Project layout

```
src/                 Vue 3 frontend (TypeScript, Pinia, Tailwind, vue-i18n)
  api/               Tauri contract types and the invoke/listen wrappers
  components/        UI components (account sidebar, title bar, tabs, log view, login banner, ...)
  log/               Colour parsing, log buffer, line wrapping, status helpers (unit-tested)
  stores/            Pinia stores (accounts, logs, settings)
  i18n/              Locale files, error-code translation, check script target
src-tauri/           Rust backend
  src/connection.rs        Connection id and per-connection settings file name
  src/process_manager.rs   One MCC process per connection, Job Object cleanup
  src/device_code.rs       Microsoft device-code prompt detection
  src/settings.rs          App settings model and validation
  src/store.rs             Atomic JSON persistence
  src/bin/fake_mcc.rs      Test double used by the automated tests
  tests/                   Integration tests against fake_mcc
scripts/             Repo scripts (i18n check, UI inspection helpers)
```

## Reference material

`reference/` holds the read-only MCC source code and a `MinecraftClient.exe` for local
testing. It is **gitignored**, excluded from TypeScript, ESLint, Vite and Vitest, and is not
needed to build or run the app. Behaviour that the app depends on is verified against the
real binary.

## Notes and caveats

- **Microsoft login uses the device code flow.** MCC prints a code and a link, and the app
  shows both in a login banner. Which Microsoft account gets used is decided by your
  browser session, not by the login typed in the app — for extra accounts, open the link in
  a private window or another browser profile. MCC also opens the browser itself.
- **MCC remembers each account** after the first successful login (its session cache lives
  in the account's folder), so the code only has to be copied once per account, and every
  server of that account reuses the same login.
- Offline accounts work without any Microsoft login.
- The app makes no network requests of its own. Everything works offline except what MCC
  itself needs.
