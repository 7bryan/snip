# `snip`

A fast, keyboard-driven terminal snippet manager and command launcher built in Rust. Save your most-used `git`, `docker`, `ffmpeg`, and PowerShell commands in a plain TOML file, search them live as you type, fill in `{{placeholders}}`, and copy or run them straight away.

---

## Features

- **Live Search:** Filter snippets as you type by title, command text, or tag (case-insensitive).
- **Split-Pane TUI:** Snippet list on the left, full preview (command, description, required variables) on the right.
- **Dynamic Placeholders:** Use `{{variable}}` syntax (e.g. `taskkill /PID {{pid}} /F`) and get prompted for each value before the command is copied or run.
- **Clipboard & Direct Execution:** Press `Enter` to copy to the clipboard, or `Ctrl+X` to run the command in PowerShell.
- **In-App Add & Delete:** Create and remove snippets without leaving the TUI.
- **Human-Readable Storage:** Everything lives in one `snippets.toml` file. Edit it in any editor or sync it via Git.
- **Zero-Config Setup:** The config folder and a set of starter snippets are created on first launch.

---

## Prerequisites (Windows)

1. **Rust toolchain:** install via [rustup.rs](https://rustup.rs/).
2. **Terminal (recommended):** [Windows Terminal](https://aka.ms/terminal) for 24-bit colors and clean Unicode borders. The legacy `conhost.exe` also works.

---

## Installation

Clone the repository and install the binary into your Cargo bin folder (`%USERPROFILE%\.cargo\bin`, which rustup adds to your `PATH`):

```powershell
git clone https://github.com/7bryan/snip.git
cd snip
cargo install --path .
```

To update after pulling or changing the code, reinstall:

```powershell
cargo install --path . --force
```

> On Windows the install fails if `snip.exe` is still running (the file is locked). Close any open `snip` sessions first.

---

## Usage

Run `snip` from any terminal to open the interactive TUI.

### CLI commands

| Command | Description |
|---|---|
| `snip` | Launch the interactive TUI |
| `snip list` | Print all saved snippets |
| `snip list --tag <TAG>` | Print only snippets with the given tag |
| `snip edit` | Open `snippets.toml` in `%EDITOR%` (falls back to `notepad.exe`) |
| `snip path` | Print the path to `snippets.toml` |
| `snip help` | Show a short help message |

### TUI keybindings

| Key | Action |
|---|---|
| *Type anything* | Filter snippets |
| `↑` / `↓` or `Ctrl+P` / `Ctrl+N` | Move the selection (wraps around) |
| `Enter` | Copy the selected command to the clipboard and exit |
| `Ctrl+X` | Run the selected command in PowerShell and exit |
| `Ctrl+A` | Open the "Add Snippet" form |
| `Ctrl+D` | Delete the selected snippet (**immediate, no confirmation**) |
| `Esc` / `Ctrl+C` | Quit without doing anything |

### Placeholders

If a command contains `{{variables}}`, a popup asks for each value in order. Press `Enter` to confirm each one; the last `Enter` copies or runs the resolved command. `Esc` cancels.

```text
taskkill /PID {{pid}} /F      ->  prompts for "pid"
```

### Adding a snippet

Press `Ctrl+A`, then fill in the four fields (title, command, description, tags). `Enter` moves to the next field, and `Enter` on the last field saves. Tags are comma-separated. `Esc` cancels.

---

## Configuration

Snippets are stored at:

```text
%APPDATA%\snip\snippets.toml
```

(Run `snip path` to print the exact location.) You can edit the file by hand:

```toml
[[snippets]]
title = "Git log pretty"
command = "git log --oneline --graph -n {{count}}"
description = "Compact commit graph."
tags = ["git"]
```

`title` and `command` are required. `description` and `tags` are optional. If the TOML is invalid, `snip` exits with an error naming the file.

Reinstalling `snip` never touches your snippets, since the config is separate from the binary.

### Starter snippets

On first launch these are created for you:

- Find process using a port (Windows)
- Force kill process by PID
- Git undo last commit (keep staged)
- Docker prune unused data

---

## Development

```powershell
cargo run                      # run the TUI from source
cargo run -- list --tag git    # pass CLI args after --
cargo check                    # check for compile errors without building
```

### Project structure

```text
src/
├── main.rs      # CLI parsing (clap), terminal lifecycle, event loop
├── model.rs     # Snippet struct, AppMode, search and variable logic
├── storage.rs   # Config path resolution, TOML load/save, starter snippets
└── ui.rs        # Stateless Ratatui rendering (layout, panes, modals)
```

---

## Known Limitations

- Command execution is hardcoded to PowerShell, so `Ctrl+X` is Windows-focused for now.
- Search is case-insensitive substring matching, not true fuzzy matching.
- `Ctrl+D` deletes immediately and saves to disk. There is no undo.
- `snip edit` does not support an `EDITOR` value that includes arguments (e.g. `code --wait`).

## Roadmap

- [ ] `[y/N]` confirmation before deleting a snippet
- [ ] `snip add` interactive CLI command
- [ ] Panic hook to always restore the terminal
- [ ] Input validation (no empty titles, commands, or variable values)
- [ ] True fuzzy search
- [ ] Cross-platform command execution``
