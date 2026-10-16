# Changelog

All notable changes to MCC GUI V2. Dates are the days the work happened, newest first.

Version **2.0.0 has not been released yet**. The installer carrying that number on disk was
built during development, so it has to be built again before a release — otherwise it will
not contain everything listed below.

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
