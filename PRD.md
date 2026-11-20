# Product Requirements Document (PRD): `snip`
**Version:** 1.0  
**Platform:** Windows (Primary), Cross-Platform Compatible  
**Language:** Rust (Edition 2021+)

---

## 1. Product Overview
`snip` is a fast, local-first Terminal User Interface (TUI) and CLI snippet manager written in Rust. It allows developers to store, fuzzy-search, parameterize, and instantly copy or execute frequently used terminal commands without leaving the keyboard.

### Goals
- **Speed:** Instantaneous startup (< 15ms) and real-time filtering across hundreds of snippets.
- **Zero-Friction Storage:** Plain-text TOML storage located in Windows `%APPDATA%` so users can easily back up, sync via Git, or hand-edit their library.
- **Safe Execution:** Cleanly restore the Windows Terminal / PowerShell buffer upon exit or panic.

---

## 2. Target User & Workflow
**Primary User:** Windows developers using Windows Terminal, PowerShell, CMD, or Git Bash who frequently forget complex CLI flags (`docker`, `git`, `ffmpeg`, `netstat`, `cargo`, `ssh`).

**Core User Flow:**
1. User opens any terminal and runs `snip`.
2. User types a partial keyword (e.g., `port`).
3. The list filters live to `"Kill process on Windows port"`.
4. User presses `Enter` (copies command to Windows clipboard) or `Ctrl+X` (runs command directly in PowerShell/CMD).
5. `snip` exits immediately, returning control to the shell.

---

## 3. Functional Requirements

### 3.1 Storage & Configuration (`storage.rs`)
- **FR-1.1:** Locate the config directory via `dirs::config_dir()`, resolving to `C:\Users\<User>\AppData\Roaming\snip\snippets.toml` on Windows.
- **FR-1.2:** On first run, automatically create the `snip` folder and populate `snippets.toml` with 3–5 helpful Windows/Git/Docker starter snippets.
- **FR-1.3:** Gracefully report syntax line numbers if a user manually edits `snippets.toml` and introduces invalid TOML.

### 3.2 Interactive TUI Mode (`ui.rs` & `model.rs`)
- **FR-2.1 (Search Input):** Focus starts automatically in the top search bar. Typing filters snippets case-insensitively across `title`, `command`, and `tags`.
- **FR-2.2 (Split-Pane Layout):**
  - *Left Pane (45% width):* Scrollable list of matching snippet titles with colored tag pills.
  - *Right Pane (55% width):* Preview displaying the full command, description, and detected `{{variables}}`.
- **FR-2.3 (Keyboard Navigation):**
  - `Up` / `Down` (or `Ctrl+P` / `Ctrl+N`): Move selection cursor.
  - `Enter`: Copy resolved command to Windows clipboard via `arboard` and exit.
  - `Ctrl+X`: Execute resolved command in the active Windows shell (`powershell -NoProfile -Command` or `cmd /C`) and exit.
  - `Ctrl+A`: Switch to inline "Add Snippet" modal form.
  - `Ctrl+D`: Delete selected snippet (with `[y/N]` confirmation prompt).
  - `Esc` / `Ctrl+C`: Exit cleanly without side effects.

### 3.3 Dynamic Variable Interpolation
- **FR-3.1:** Detect `{{variable_name}}` tokens inside a command string (e.g., `netstat -ano | findstr :{{port}}`).
- **FR-3.2:** If variables exist when `Enter` or `Ctrl+X` is pressed, open a lightweight popup prompt asking for each variable's value before copying/executing.

### 3.4 CLI Subcommands (`main.rs` via `clap`)
- `snip` (no args): Launch the interactive TUI.
- `snip add`: Interactive CLI prompts to append a new snippet to `snippets.toml`.
- `snip list [--tag <TAG>]`: Print matching snippets directly to `stdout` without entering TUI mode.
- `snip edit`: Open `snippets.toml` in the user's default editor (`%EDITOR%`, falling back to `notepad.exe` on Windows).
- `snip path`: Print the exact path to `snippets.toml`.

---

## 4. Non-Functional Requirements & Architecture

### 4.1 Terminal Safety (Windows Specific)
- Must enable Crossterm raw mode and alternate screen buffer on startup.
- Must implement a `Drop` guard and a custom `std::panic::set_hook` so that if the app panics during development, raw mode is disabled and the cursor reappears (preventing a broken PowerShell prompt).

### 4.2 Module Architecture
```text
src/
├── main.rs      # CLI argument parsing (clap), terminal lifecycle, main event loop
├── model.rs     # AppState, AppMode enum, Snippet struct, search & variable logic
├── storage.rs   # Windows %APPDATA% path resolution, TOML serialization/deserialization
└── ui.rs        # Stateless Ratatui rendering functions (layout, panes, modals)
