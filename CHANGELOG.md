# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/) and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added

- `static` feature to statically link `tdjson` (use with `download-tdlib` or `local-tdlib`); no runtime `tdjson` dependency needed. Powered by **tdlib-rs v1.4.0**.
- CI: `static` feature combinations (`local-tdlib,static`, `download-tdlib,static`) in Linux, macOS, and Windows workflows.
- CI: Android cross-compilation workflow (`aarch64-linux-android`, `x86_64-linux-android`) with static linking.
- Command guide popup (`Alt+F1`) listing keybindings from `keymap.toml`.
- Theme switcher with dynamic theme discovery; theme choice can persist (see docs for release vs debug behaviour).
- Server-side chat message search (TDLib), search overlay, jump-to-message, and `Alt+C` to restore default message order; search errors surface in the status bar.
- Photo viewer for images with keyboard navigation, lazy-loaded history, layout improvements, and keybinding hints loaded from config.
- Mouse support: click-to-focus chat list, chat, and prompt; scroll in list and chat; click chat list items to open a chat (fixed row height for hit testing).
- Multi-line messages in the chat (send and display) and related editing fixes.
- Voice and audio message playback (optional build feature; see README).
- Voice/video call placeholders in the chat list so those chats do not show as empty.
- Pin indicator (📌) for pinned chats in the chat list.
- Pinned messages for the open chat: pinned strip with preview and shortcut hint, `Shift+Tab` / `BackTab` to open a pinned-messages popup (from chat or chat list), browse pins, jump to a pin with Enter, open photo/voice from the popup; `Alt+S` save-as flow for photos/documents with a Ratatui file explorer; `Alt+U` file upload explorer with Ratatui theming.
- Config and data paths aligned with XDG Base Directory (with migration/fixes for prior layout).
- GitHub Actions: Docker workflow; Windows ARM CI; TDLib **1.3.0** with updated **tdlib-rs**, Dockerfile and CI adjustments; Linux image/chafa-related workflow fixes.

### Changed

- Upgraded **tdlib-rs** to v1.4.0 (new `static` feature, `ureq` replaces `reqwest`, Android support, `#[link]` attribute removed from tdjson FFI).
- UI render-on-demand to reduce idle CPU while keeping Telegram-driven updates responsive.
- Focus tracking uses atomics instead of a mutex for lock-free reads/writes.
- Hardcoded keys moved into `keymap.toml`; keymaps merge defaults with user config and load dynamically (including photo viewer hints).
- Chat message list uses bottom-anchored layout to avoid large gaps; reply target highlighting; reply/edit rules (e.g. no editing others’ messages).
- Dependency and toolchain updates (e.g. `rodio` 0.22, `ratatui-image`, `toml`, `clap`, Docker actions, and other crates).

### Fixed

- Stack overflow in the prompt when wrapping at the window edge (`insert_newline` no longer recurses through `insert('\n')`).
- Occasional startup hangs and chat list refresh hangs.
- Chat search reliability and clearer failure feedback via the status bar.
- `q` incorrectly triggering quit while typing in the prompt (context-aware key handling).
- Chat list: scrolling, unread badge refresh when opening chats, refresh on launch, wrong unread count when a message arrives in the open chat, updates after server-side deletes, and `DeleteMessages` from cache vs real deletes.
- Duplicate last message when scrolling; chat history performance regressions; config creation/loading issues; popup and list quirks.
- Reply flow, reply styling, and copying messages with history restore.
- Photo viewer: focus return to chat on close; dynamic keymap text in the viewer.
- Docker build with newer `rodio` API; path/config bugs on the XDG rework; theme readability (`item_message_content` and post-processing).
- Tests and CI (incl. Windows `ratatui-image` picker, timeouts, `Alt+C` test on Windows).

## [1.0.0] - 2024-08-09

### Added

#### Telegram API
- project configuration for APIs
- Authentication
- Receive messages
- Send messages
- Handle updates from server
- Handle login
- Handle logout
- Handle view message
- Change user status to online and offline
- Display messages with time of arrival
- Display if messages has been readed
- Edit message
- Delete message
- Copy message
- Reply message
- Handle message edited

#### Config
- Logger config
- Keybindings config
- App config
- Theme config
- Telegram config

#### CI/CD
- CI for Linux
- CI for MacOS Intel
- CI for MacOS arm64
- CI for Windows
- CD for release

### Changed

### Fixed
