# `snip`

A blazing-fast, keyboard-driven terminal snippet manager and command launcher built in Rust. Save your most-used `git`, `docker`, `ffmpeg`, and PowerShell commands in a clean TOML file, fuzzy-search them in milliseconds, fill in dynamic placeholders, and copy or run them immediately.

---

## Features

- **Instant Fuzzy Search:** Filter snippets live by title, command text, or tags as you type.
- **Dynamic Placeholders:** Use `{{variable}}` syntax (e.g., `taskkill /PID {{pid}} /F`) and get prompted for values on the fly.
- **Clipboard & Direct Execution:** Press `Enter` to copy to your Windows clipboard, or `Ctrl+X` to run immediately.
- **Human-Readable Storage:** All snippets live in a single `snippets.toml` file—easy to edit in VS Code or sync via Git.
- **Zero-Config Setup:** Automatically generates your config folder and starter snippets on first launch.

---

## Prerequisites (Windows)

1. **Rust Toolchain:** Install via [rustup.rs](https://rustup.rs/).
2. **Terminal (Recommended):** [Windows Terminal](https://aka.ms/terminal) (supports full 24-bit colors and crisp Unicode box borders; legacy `conhost.exe` works too).

---

## Installation

Clone the repository and install the binary globally into your Cargo bin folder (`%USERPROFILE%\.cargo\bin`):

```powershell
git clone [https://github.com/yourusername/snip.git](https://github.com/yourusername/snip.git)
cd snip
cargo install --path .
```
