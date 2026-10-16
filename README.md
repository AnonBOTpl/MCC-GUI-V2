# MCC GUI V2

A local Windows 11 desktop app for running **several Minecraft accounts at once, each on one
or more servers**.

One window instead of a dozen console windows: every account gets its own tab with a live,
colourful log, you send chat or commands to one connection or to all of them at once, and
Microsoft sign-in is done by copying a code shown on screen — no password is ever typed into
the app or stored by it.

The actual Minecraft connection is handled by
[Minecraft Console Client](https://mccteam.github.io/guide/usage.html) (MCC), the same
program you would otherwise run by hand. The app starts one MCC process per connection and
shows what it prints.

🇵🇱 Wersja polska: [README-PL.md](README-PL.md)

## What you need

- **Windows 11.** The app is written for Windows only.
- **Minecraft Console Client.** A separate program you download yourself from
  [mccteam.github.io](https://mccteam.github.io/guide/usage.html) and keep in a permanent
  folder. The app never bundles or downloads it — it only starts the
  `MinecraftClient.exe` you point it at.
- Nothing else. The WebView2 component the app uses is already part of Windows 11.

## Installing the app

1. Download `MCC.GUI.V2_2.0.0_x64-setup.exe` from the Releases page of this repository (the
   release shows the build's file name with dots, because GitHub does not accept spaces in
   uploaded file names).
2. Run it. Windows will show a **SmartScreen** warning, because the installer is not
   code-signed — choose **More info** → **Run anyway**.
3. The installer needs **no administrator rights**; it installs into your own user profile.

The installer is unsigned, so a signature cannot confirm that a download is the file this
project published. Every release therefore lists the **SHA256** of its installer in the
release description. Compare it before running the file:

```powershell
Get-FileHash -Algorithm SHA256 ".\MCC.GUI.V2_2.0.0_x64-setup.exe"
```

On a system where that command is unavailable (or in an older shell), `certutil` — which
ships with Windows — returns the same number:

```powershell
certutil -hashfile ".\MCC.GUI.V2_2.0.0_x64-setup.exe" SHA256
```

If the hash differs from the one in the release description, do not run the installer.

## First run

On the first launch the app opens its Settings, because it does not know yet where MCC is:

1. Click **Browse** and pick your `MinecraftClient.exe`.
2. Click **Save**.

That path is remembered and reused later.

Then press **Add account** in the header of the accounts panel on the left, choose the
account type, fill in the login and confirm:

- **Microsoft** — type the e-mail address. No password: MCC shows a code to enter at
  `microsoft.com/link`, and the app shows a banner with the code and a button that opens the
  page for you.
- **Offline** — just a nickname, no sign-in at all.

The account appears in the panel and **nothing connects yet**. Add one or more servers to it
there (an address, and a forced Minecraft version if that server needs one), then press the
green play button next to a server (**Start** in its tooltip) to connect.

## Using the app

**Accounts panel (left).** Every saved account with its list of servers and the status of
each connection. From here you add accounts and servers, remove them, and start or stop a
connection. A server is connected only when you press its green button (its tooltip is
**Start**) — nothing connects on its own, and saved tabs come back stopped.

**Tabs (top).** One tab per connection (account + server), each with its own log and its own
status dot. The close button on a tab closes it **and stops that connection**; the server
stays saved on the account and can be started again.

**Connection header.** The account label, the server address and the current status, the
**Auto-scroll** checkbox, and either **Start** (when the connection is stopped or after a
crash) or **Disconnect** with **Reconnect**.

**Log.** The output of that connection, exactly as MCC prints it, with Minecraft colour codes
rendered. Long lines wrap to the window. While auto-scroll is on, the view follows new lines;
scroll up and it stops following, scroll back to the bottom (or tick **Auto-scroll**) and it
resumes. You can select and copy lines: click a line to select it, `Ctrl`+click to add or
remove one, `Shift`+click for a range, `Ctrl`+`A` for everything, `Ctrl`+`C` to copy and `Esc`
to clear. A right click gives **Copy** and **Copy all**. Copied text is clean — colour codes
are stripped.

**Input.** Type chat text and press `Enter`, or type an MCC command starting with `/`. Tick
**Send to all** to send the same line to every running connection, with a small delay between
accounts so servers do not see it as spam. Server-side commands have to go through MCC's own
`/send`, for example `/send /home`.

**Quick actions (bottom).** Your own shortcut buttons for the commands you type most. The bar
sits collapsed as a thin strip at the bottom and unfolds when you click it. Clicking an action
sends it exactly as if you had typed it, with the same scope as the input bar.
**Edit** lets you add, rename, recolour, reorder and switch actions on or off; an action that
is off disappears from the bar but stays in the editor.

**Settings (gear icon in the title bar).** The MCC path, restoring tabs on start, asking
before closing with live connections, the **Delay between accounts** for "send to all", the
**Log lines kept per account** and the chat log settings. Next to the app title there is also
a small **EN / PL** switch for the language, and that choice is remembered.

**Closing.** Closing the window stops every MCC process the app started, so no
`MinecraftClient.exe` is left running in the background. If connections are still up, the app
asks first — you can turn that question off in the same dialog or in Settings. In the
installed build, starting the app a second time does not open another window: it brings the
one that is already running to the front.

## Chat logs

Chat logs are **on by default** and can be switched off in Settings. Every connection writes
what it sees and says to:

```
<logs>\<server>\<account label>\<YYYY-MM-DD>.log
```

`<logs>` is `%APPDATA%\com.mccgui.v2\logs` unless you pick another folder in Settings
(**Change**, **Open folder**). All sessions of the same day append to one file, each one
delimited by `=== session start ... ===` and `=== session end ... ===` lines. After local
midnight the writer moves to the next day's file, even in a running session.

Lines are UTF-8, one per MCC output line: `[21:03:07] <Steve> hello`, with Minecraft `§`
colour codes and ANSI escapes removed. Anything MCC writes to stderr gets a `[stderr] `
prefix.

**Privacy:** these files hold the server chat of that account — including private messages
sent by other players, which the server delivers to the client. They never leave your
computer: nothing is uploaded, and the folder is a plain directory you can back up, search or
delete. Login prompts (device code and link) are filtered out before anything is written, so
no code ever reaches a file.

**Retention:** **Keep chat logs for** (default 30 days, `0` = keep everything). Cleanup runs
at startup and once a day, and deletes only files named `YYYY-MM-DD.log` at
`<server>\<account>\` depth — anything else you keep in the logs folder is left alone.

Writing happens in the background, so a slow disk can never stall the app; if too much output
arrives at once, the file gets a single `N lines omitted (queue full)` line. A write error
(for example a full disk) shows one error message and turns writing off for that connection,
instead of repeating it.

## Where your data lives

Everything is stored under `%APPDATA%\com.mccgui.v2\`:

| Path | Contents |
|------|----------|
| `settings.json` | App settings: MCC path, language, "send to all" delay, log buffer size, restore-tabs switch, chat log settings, server history |
| `accounts.json` | Saved accounts, each with its list of servers |
| `presets.json` | Quick action buttons: label, the exact line sent to MCC, optional colour and an on/off switch per action |
| `session.json` | The tabs that were open when you closed the app, restored on the next start |
| `logs\` | Chat logs (see above); the folder can be moved in Settings |
| `accounts\<id>\` | Per-account MCC working directory: one generated `.ini` per connection and MCC's own session cache |

An installed build and a development build share this directory, so your accounts, settings
and open tabs are the same in both.

**No passwords or tokens are ever stored by the app.** Microsoft accounts use a device-code
login and offline accounts have no password at all. MCC session files stay inside the account
folder and the app never reads, copies or exports them.

## Good to know

- **Microsoft login uses the device code flow.** Which Microsoft account gets used is decided
  by your browser session, not by the e-mail typed in the app — for another account, open the
  link in a private window or a different browser profile. MCC also opens the browser itself.
- **MCC remembers each account** after the first successful login, so the code has to be
  copied only once per account, and every server of that account reuses the same login.
- **Offline accounts** need no Microsoft login at all.
- **The app makes no network requests of its own.** Everything works offline except what MCC
  itself needs.
- **Nothing else is collected.** No telemetry, no analytics, no crash reporting.

## Known limitations

- The installer is **not code-signed**, so Windows shows a SmartScreen warning the first time
  you run it.
- Only **Windows 11** is supported.
- You need your own copy of Minecraft Console Client — the app does not include it.
- Log search, log export outside the logs folder and notifications are not part of this
  version; chat logs go to files instead.

## License and third-party notices

MCC GUI V2 is released under the **MIT License** — see [LICENSE](LICENSE).
Copyright (c) 2026 AnonBOTpl.

The third-party components the app uses, each with its license, are listed in
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

The project is **not affiliated with, endorsed by or sponsored by** Mojang Studios, Microsoft,
or the Minecraft Console Client team. “Minecraft” and “Mojang” are trademarks of Mojang
Studios. The app never speaks the Minecraft protocol itself — it goes through MCC.

**Minecraft Console Client is not bundled with this app and is not redistributed by it.** You
download MCC yourself from [mccteam.github.io](https://mccteam.github.io/guide/usage.html), and
the app only starts the `MinecraftClient.exe` you point it at, as a separate process.

MCC itself is licensed under the **CDDL 1.0**
([LICENSE.md in the MCC repository](https://github.com/MCCTeam/Minecraft-Console-Client/blob/master/LICENSE.md)).
That licence is copyleft per file, but its obligations apply to MCC's own source files, and
this app copies, modifies, links or bundles none of them — it talks to MCC over standard input
and output, which the licence explicitly allows as a “Larger Work”. So nothing here is covered
by CDDL and this project can stay on MIT.

## For developers

Everything below is about working on the app, not about using it.

### Status

Work in progress.

| Stage | What it covers | Status |
|-------|----------------|--------|
| 0 | Scaffold, tooling, MCC research | done |
| 1 | Process manager (Rust) | done |
| 2 | Core UI: tabs, log, input | done — 200 lines/s keeps the UI at ~59 fps |
| 3 | Microsoft login banner and status handling | done |
| 4 | Persistence: accounts, settings, per-account servers | done |
| 5 | Log scroll polish | done (log filters are out of scope) |
| 5b | UI robustness: the log no longer freezes with a full buffer | done |
| 6 | Quick actions | done |
| 7 | Language switch, final Polish translation, packaging | todo — see the note below |

**Stage 7** has its language switch, app icon and installer done; what is left is a
read-through of the Polish translation, which is why the row above is still `todo`.
Everything from stages 0-6 works, including the `5b` robustness fix.

### Toolchain

- **Node.js 24** and npm (the version this project is built and tested with).
- **Rust 1.90+** with the MSVC toolchain (`rustup default stable-msvc`); the crate declares
  edition 2024.

### Development commands

```bash
npm install
npm run tauri dev
```

For development you can skip the settings dialog and provide the executable through the
`MCC_EXECUTABLE` environment variable — a saved path always wins over it:

```bash
MCC_EXECUTABLE="C:/path/to/MinecraftClient.exe" npm run tauri dev
```

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

### Architecture

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
- **Chat logs** are written by a separate task through a bounded queue, so reading MCC output
  never blocks; dropped lines are counted and reported in the file.
- **Auto-scroll is per tab:** scrolling up in one connection pauses only that tab, and each
  tab keeps its own setting. Starting or restarting a connection follows its output again,
  because its log starts from scratch.
- **The only bridge** between Rust and the UI is the Tauri command/event contract
  described in `src/api/contract.ts`.
- **The app never spawns processes or touches the filesystem from the frontend** — all of
  that lives in Rust.

### Releases

```bash
npm run tauri build
```

The executable and bundles land in `src-tauri/target/release/` and
`src-tauri/target/release/bundle/`.

```bash
npm run tauri -- build --bundles nsis
```

This produces a per-user NSIS installer,
`src-tauri/target/release/bundle/nsis/MCC GUI V2_2.0.0_x64-setup.exe`. It needs no
administrator rights and installs into the user's own profile.

The version has **one source: `src-tauri/Cargo.toml`** (`version = "2.0.0"`). The Tauri
config has no `version` field on purpose, so Tauri takes the number from `Cargo.toml` — the
installer name, the executable metadata and the bundle metadata all follow it. `package.json`
carries the same number only because npm requires a `version` field there; the build does not
read it. On a release, bump `Cargo.toml` and set the same number in `package.json`.

### Languages

The interface is **English by default** and starts in English on first launch regardless
of the system locale. Polish is available from the language switch in the title bar, and the
choice is remembered.

Every user-visible string goes through `vue-i18n`. `en` is the source of truth and the
fallback; `pl` must contain every key, which `npm run check:i18n` enforces. Rust never sends
human-readable text to the UI — it sends stable error codes with parameters, and the
frontend translates them. Raw MCC and server output is shown exactly as received and is
never translated.

### Project layout

```
src/                 Vue 3 frontend (TypeScript, Pinia, Tailwind, vue-i18n)
  api/               Tauri contract types and the invoke/listen wrappers
  components/        UI components (account sidebar, title bar, tabs, log view, login banner, ...)
  log/               Colour parsing, log buffer, line wrapping, selection, status helpers (unit-tested)
  presets/           Quick action model, ordering and the colour palette (unit-tested)
  session/           Which tabs were open at shutdown and how they are restored (unit-tested)
  stores/            Pinia stores (accounts, logs, settings)
  i18n/              Locale files, error-code translation, check script target
src-tauri/           Rust backend
  src/chatlog.rs           Chat logs: paths, sanitising, retention, writer queue
  src/connection.rs        Connection id and per-connection settings file name
  src/process_manager.rs   One MCC process per connection, Job Object cleanup
  src/device_code.rs       Microsoft device-code prompt detection
  src/settings.rs          App settings model and validation
  src/store.rs             Atomic JSON persistence
  src/bin/fake_mcc.rs      Test double used by the automated tests
  tests/                   Integration tests against fake_mcc
scripts/             Repo scripts (i18n check, UI inspection helpers)
```

### Reference material

`reference/` holds the read-only MCC source code and a `MinecraftClient.exe` for local
testing. It is **gitignored**, excluded from TypeScript, ESLint, Vite and Vitest, and is not
needed to build or run the app. Behaviour that the app depends on is verified against the
real binary.
