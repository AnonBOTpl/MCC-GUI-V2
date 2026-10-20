# Changelog

All notable changes to MCC GUI V2. Dates are the days the work happened, newest first.

Version **2.0.0** is published. Its installer was rebuilt on 20 October 2026 from the current
tree and the release asset was replaced with that build (new SHA256 in the release notes).
The 19 October build was installed on Windows 11 to check it; the 20 October build has not
been installed by hand yet. Everything listed below is in the current installer.

## 20 October 2026

**Fixed**

- **One Enter sends one message.** The input field is cleared only after the backend answers,
  so a second Enter (or a double click on Send) while the first send was still in flight sent
  the same text again — and with “Send to all” that meant two overlapping rounds with the
  per-account delay. Submits are now ignored until the send finishes; text typed meanwhile
  stays in the field.
- **`Esc` closes only the top dialog.** Settings and the close-confirmation each listened for
  `Esc` on the window, so one press closed both — and the question about quitting took the
  unsaved settings with it. Only the topmost modal handles the keyboard now; `Tab` is trapped
  by that one dialog too instead of by all of them at once.
- **The chat-log switch and folder apply to running connections.** Saving is stored per
  connection, and a session was created only when that connection started: turning saving on
  mid-session wrote nothing, and changing the folder left the old writer pointing at the old
  place (or stopped writing until the connection restarted). Both now take effect on the next
  line, without restarting anything.
- **Checking a log folder no longer touches your files.** The write test used a fixed name
  (`.mcc-gui-write-test`) and wrote an empty file before deleting it, so a file you happened to
  keep under that name in the chosen folder was wiped. The probe now gets a unique name
  (process id and a counter), is created with `create_new` and only the probe's own file is
  removed.
- **An account that could not be saved is no longer reported as added.** When `accounts.json`
  cannot be written (for example because an unreadable copy was quarantined at the start), the
  dialog used to close as on success while the account lived only in memory and disappeared on
  the next launch. The save result now reaches the form: the dialog stays open with your input,
  the reason is shown, and the half-added account is rolled back. Adding a server reports the
  same way and does not start a connection for a server that was not stored.
- **The folder picker speaks your language.** Its window title was a hard-coded English string
  in Rust; it now comes from the interface translation like every other visible text.
- **A moved or deleted `MinecraftClient.exe` no longer resets your settings.** One invalid field
  rejected the whole settings file at startup, and the next save wrote the defaults over the
  rest. Only the broken field falls back now (the app also says which one), so the delay,
  buffer size, retention, log folder and switches survive; the Settings dialog opens to ask
  for the exe again.

## 19 October 2026

**Fixed**

- **Log colours stay readable on the dark background.** Six of the sixteen Minecraft colours
  were below the WCAG AA contrast for text on the log background (`§0` 1.04:1, `§1` 1.52:1,
  `§4` 2.60:1, `§5` 3.16:1, `§8` 2.71:1, `§9` 3.97:1). They are now lightened inside their own
  colour family (`§0` → `#7b7b7b`, `§1` → `#7373d9`, `§4` → `#d94a4a`, `§5` → `#b95cb9`,
  `§8` → `#8f8f8f`, `§9` → `#7d7dff`), so dark still reads darker than light in every pair.
  The other ten colours are untouched, and a unit test now checks all sixteen against the log
  background. ANSI colours take their values from the same table, so they get the fix too.
  Details in `docs/MCC_NOTES.md` §9.
- **A long session no longer keeps row heights of lines that left the buffer.** The measured
  heights are pruned together with the log buffer, so memory follows the buffer size instead of
  the time the app has been open.

**Accessibility and keyboard**

- **Screen readers get proper tab roles:** each tab carries a name and points at the panel it
  controls, and that panel is labelled by its tab.
- **The focus outline and the tab arrow-key shortcuts are gone again.** Both slipped into this
  batch without being asked for: a 2 px outline boxed in every focused element, and the arrow
  keys (plus `Home`, `End` and `Delete`) took over the tab bar while `Tab` stopped reaching a
  tab's close button. Neither is in this build — focus looks the way it did before, and `Tab`
  walks through every tab and its close button again. The screen-reader roles listed above stay.
- **The log menu opens with `Shift`+`F10` or the Menu key**, appears next to the first selected
  line, starts on the first item that can be used, moves with the arrow keys and returns focus
  to the log on `Esc`.
- **Icon buttons are 24×24** (remove account, remove server, cancel adding a server, close a
  message, the help icon in the login banner), and the server's remove button now appears when
  it has keyboard focus, not only on hover.
- **A long server address no longer pushes the header buttons off screen** — the address and
  the account name shorten, and the buttons keep their place.
- **Text is easier to read:** helper text moved from `slate-500`/`slate-600` to `slate-400`
  (7.9:1 on the log background, 6.9:1 on panels), placeholders to `slate-500`, and button hover
  states now keep at least 4.5:1 with white text (`blue-700`, `sky-700`, `red-700`).
- **The empty screen says what to do next:** with no accounts it offers “Add account”; with
  accounts but no tab it points at the green play button next to a server.
- **Reduced motion is respected:** with that Windows setting on, tab, modal and toast
  transitions stop animating.

**Changed**

- **Removing an account asks first.** The trash button opens a question naming the account and
  says that a running connection is closed first; nothing is removed until you confirm.
- **Closing a tab with a live connection asks first.** When the client is starting, waiting for
  login or running, the tab asks before it disconnects; stopped and crashed tabs close right
  away. Cancelling leaves the tab and the process untouched.
- **The app now sends a Content Security Policy.** Only the app's own scripts run: an injected
  inline script is blocked in the built app, while inline styles stay allowed because the
  virtualised log positions its rows with them. Connections are limited to the app itself and
  the Tauri IPC channel.

**Docs**

- The README now says that quick actions and their editor only exist while a connection tab is
  open, so their absence with no tabs is expected behaviour, not a bug.

## 18 October 2026

**Changed**

- **Each account's chat logs get their own folder, with a short hash in the name.** The folder
  used to be named after the account label alone, so two labels that Windows treats as the
  same folder (`Work` and `work`, or a label with a trailing space) wrote into one directory and
  their logs mixed. The folder is now `<account label>-<four hex characters>`, for example
  `Work-3f9a`. The hash comes from the label, so it stays the same across restarts; renaming an
  account starts a new folder. Logs written so far stay where they are and are not moved.

## 17 October 2026

A review of the whole app (input handling, files on disk, dialogs, process lifetime) turned up
several defects. All of them are fixed below.

**Fixed**

- **A damaged `accounts.json` no longer wipes every account.** The file is read entry by
  entry, so one broken account is skipped and the rest load normally. The app says how many
  entries it skipped and keeps a copy of the damaged file (name ending in `.bad-…`) next to it,
  instead of turning the whole list into an empty one that the next save would then write down.
- **Files that cannot be read are no longer overwritten.** If `accounts.json`, `settings.json`,
  `presets.json` or `session.json` cannot be read when the app starts, a message explains what
  happened, a copy ending in `.bad-…` is kept next to the file, and saving that file is paused
  until it reads correctly again (for presets that is: fix the file and press Reload). Before,
  the first save silently replaced the file with empty defaults.
- **Closing a tab now waits for the process to stop.** Closing a tab right after starting a
  connection could leave `MinecraftClient.exe` running with no tab left to control it. The tab
  stays visible while the client shuts down, and if stopping fails the tab stays with a message
  instead of disappearing.
- **Starting the same connection twice no longer opens two clients.** A second click while the
  first start was still in flight could leave two `MinecraftClient.exe` processes for one tab
  (and a false “crashed” status when one of them exited).
- **A client that stops reading its input can no longer freeze the app or block stopping it.**
  Sending input and stopping no longer wait on a global lock, and a write that the client never
  reads gives up after a timeout.
- **Double-clicking a Start button no longer connects and immediately disconnects.** The second
  click of a double click is ignored: the button changes under the cursor after the first click,
  so it used to hit a different action (disconnect in the panel, reconnect in the tab).
- **The dialogs are proper dialogs for the keyboard.** Add account, Settings, the closing
  question and the quick-action editor now keep `Tab` inside themselves, close with `Esc`,
  keep the focus in the right field when they open, and give focus back to the button that
  opened them. Before, `Tab` walked into the window behind the dialog.
- **The collapsed quick-action bar is out of the keyboard order** — hidden presets can no
  longer be reached (and sent) with `Tab` and `Enter`. Before, pressing `Tab` a few times while
typing could send an action that was not visible on screen.
- **Chat-log cleanup no longer follows links.** A junction (link) inside the log folder can no
  longer make the app delete old `YYYY-MM-DD.log` files outside that folder.
- **A save interrupted by a crash or a power cut no longer leaves an empty settings file.** Data
  is flushed to disk before the file is swapped in, and two saves of the same file can no longer
  trip over each other's temporary copy.
- **The colour menu in the quick-action editor fits its text.** In Polish the longest name was cut
  short; the menu is now as wide as the longest entry, so the whole word is readable.
- **A closed tab no longer leaves its line selection behind.**
- **A message you typed is no longer lost when nobody could receive it.** With no open
  connection, or with “Send to all” on and nothing running, the text stays in the field and a
  line above it says why it was not sent.
- **Error cards no longer pile up.** The same error shown twice gives one card, and at most five
  are kept at a time — before, a burst of errors filled the whole bottom corner.
- **Error messages are visible on top of dialogs.** The message used to be drawn behind the open
  dialog, so a save that failed in Settings looked like nothing had happened at all.
- **A failed save in Settings keeps what you entered.** Before, the dialog threw the changes away
  and went back to the values stored on disk, so retrying meant typing everything again.
- **Two quick setting changes no longer cancel each other.** Changes are written one after
  another, so the second one no longer overwrites the first.
- **The quick-action editor stops at the limit.** The 61st action could be added and the whole
  list was rejected on Save; the editor now blocks it and shows `n / 60`.
- **The button that jumps back to the end of the log says what it does.** It read “Following”
  while following was off; it now says “Follow”.
- **The window tells other programs which language it is in.** After switching to Polish, screen
  readers and the spell checker in text fields know the app is Polish instead of assuming English.
- **The Settings dialog scrolls** when the window is short, so the Save button stays reachable
  (it used to be pushed off screen).

**Changed**

- **Account data is validated before it is used.** An account id with a path separator, a login
  with a control character or a server address that would break the generated client settings
  are rejected — whether they come from the panel or from a hand-edited file.
- **Only Microsoft-hosted login links can be opened.** A line in chat that merely looks like a
  login prompt can no longer turn into a clickable link to some other site.
- **`MCC_EXECUTABLE` is a development-only fallback.** Installed builds take the path from
  Settings only, so an environment variable can no longer point a released build at some other
  program.

## 16 October 2026

**Added**

- **Chat logs saved to disk** — on by default. Every server and every account has its own
  folder with one file per day. Each line carries the time it was received, error output is
  marked, and the start and end of a session are clearly separated. A session that runs
  through midnight continues in the next day's file. Settings can switch logging off, change
  the folder, open it with one button and choose how many days to keep (or keep everything).
  **Login codes are never written** — not to these files and not anywhere else.
- **Selecting and copying lines** in the log: click selects a line, `Ctrl`+click adds or
  removes one, `Shift`+click selects a range, `Ctrl`+`A` selects everything, `Ctrl`+`C`
  copies, `Esc` clears. A right click gives **Copy** and **Copy all**. In text fields
  `Ctrl`+`C` and `Ctrl`+`A` keep working normally.
- **One copy of the app at a time** — starting it again does not open another window, it
  brings the running one to the front (installed build).
- **A question before closing** while connections are live — “Close and disconnect?”, with the
  number of connections, a “Don't ask again” option, and the same switch in Settings.
- **The language switch (EN/PL) moved to the title bar**, so it no longer takes a whole row.
- **A third-party license list** — every component the app is built on, with its version and
  license, is listed in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

**Fixed**

- **“Disconnect” did nothing** if you had typed `/quit` earlier — the button looked dead while
  the connection kept running. It disconnects reliably now.
- **Tab bar**: the active tab scrolls itself into view, and the mouse wheel scrolls the bar
  horizontally.
- **The hint on an empty screen** points at the right button for adding an account.

**Changed**

- The project got its licence (MIT) and a reworked README: it explains that the app is not
  affiliated with Mojang, Microsoft or the Minecraft Console Client team, that MCC is not
  bundled and where to get it, and that the installer is unsigned and how to check its
  SHA256 fingerprint.
- One single version number across the project, so the installer and the app can no longer
  show different numbers.

## 15 October 2026

**Fixed**

- **Saved accounts disappearing**: with tab restoring switched off, adding a new account wiped
  the previously saved accounts from the list.
- **Auto-scroll works per tab** — scrolling one log up no longer stops the others from
  following their output.

**Changed**

- The names Windows shows (publisher in “Apps & features”, the app's data folder) were made
  consistent and readable.
- README: a section on the unsigned installer and on verifying the downloaded file.

## 13 October 2026

**Added**

- **Adding an account in its own dialog** instead of a form permanently in the panel, with the
  account type shown as two cards.
- **Tabs come back after a restart** — stopped, so nothing connects on its own.
- **A stop button next to each server** in the accounts panel, beside the green start button.
- **The first installer** (unsigned, for testing).

**Fixed**

- **The log stopped refreshing once its buffer was full** (long sessions). Confirmed by the
  user after almost 10 hours on real accounts without a single freeze.
- **The log did not refresh after returning to the window** when it had been minimised or
  covered by other programs.

**Changed**

- Accounts are added with a button in the accounts panel header — the separate top bar for
  adding accounts is gone.

## 12 October 2026

**Fixed**

- The log showed no new messages after returning to a minimised or covered window — you had to
  scroll with the mouse wheel for the content to appear.

## 11 October 2026

**Fixed**

- **The switch next to a quick action did something other than it appeared to** — it now simply
  switches that action on or off. A switched-off action disappears from the bar but stays in
  the editor, so it can be brought back.
- **The scope of a quick action** comes from the “Send to all” checkbox, exactly like a message
  typed by hand.
- **Reordering actions by dragging** actually works (before, nothing happened even though the
  interface invited it).

## 10 October 2026

**Added**

- **Quick actions**: your own buttons for the commands you use most, with an editor (label,
  colour, on/off, order) and a **Reload** button that re-reads the file.
- **Log scrolling**: scrolling up stops the view from following new lines, and scrolling back
  to the bottom (or ticking **Auto-scroll**) makes it follow again.

**Fixed**

- The log could “freeze” after a few hours while the app was running in the background.

## 4 October 2026

**Added and changed**

- **One account, many servers**: an account can hold a list of servers, and every connection is
  its own tab. All servers of an account share the same login.
- **A forced Minecraft version per server** — for servers that do not answer the version
  question (the client used to stop and wait on those).
- Measured performance: 200 lines per second for a minute with no slowdown in the interface.

**Fixed**

- The form for adding a server could not be closed — it now has a Cancel button and reacts to
  `Esc`.
- Clicking a server name connected immediately — the name is now just a label and a green
  button connects.
- Long messages created a horizontal scrollbar in the log — they now wrap to the window width.

## 3 October 2026

**Added — the first working version**

- An accounts panel with each account's server list and the status of every connection.
- Tabs: one connection per tab, each with its own log.
- A coloured log (Minecraft colours) with scrolling and a bounded number of lines.
- Sending messages and commands to the selected account or to all of them, with a delay
  between accounts so servers do not see it as spam.
- **Microsoft sign-in without a password**: a code to enter in the browser, a button to copy
  the code and a button to open the link.
- Connection statuses: starting, waiting for login, connected, stopped, crashed.
- Settings: client path, language, send delay, log size, restoring tabs and a history of the
  server addresses you type. The interface works in English and Polish.
- Accounts and settings are saved between launches — **without any passwords or tokens**.

## Deliberately not included

- **Log filters** (all / chat / system / errors) — dropped: the important things are visible
  outside the log (status dots, the connection header, the login banner, error messages), and
  filters would only crowd the interface.
- **Timestamps in the app window** — the time is only in the saved chat files.
- **Uploading the log anywhere, searching it, notifications** — saving the chat to files on
  your computer replaced that.
- **Keyboard shortcuts** beyond selecting and copying lines.
- **Editing an account** — only its servers can be changed; the account itself has to be
  removed and added again.
- **Automatic reconnecting** — postponed.

## Known limitations

- The installer is unsigned, so Windows shows a SmartScreen warning the first time you run it.
- The app runs on Windows 11 only.
- You need your own copy of Minecraft Console Client — the app does not include it.
