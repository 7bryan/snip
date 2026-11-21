mod model;
mod storage;
mod ui;

use anyhow::Result;
use arboard::Clipboard;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use model::{App, AppMode, Snippet};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, process::Command};

#[derive(Parser)]
#[command(name = "snip", about = "Fast terminal snippet launcher")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all saved snippets
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    /// Open snippets.toml in default editor
    Edit,
    /// Print the path to snippets.toml
    Path,
}

/// Ensures Windows Terminal raw mode is always cleaned up on exit
struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

enum ExitAction {
    Quit,
    Copy(String),
    Execute(String),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Path) => {
            println!("{}", storage::get_config_path()?.display());
            return Ok(());
        }
        Some(Commands::Edit) => {
            let path = storage::get_config_path()?;
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "notepad.exe".to_string());
            Command::new(editor).arg(path).status()?;
            return Ok(());
        }
        Some(Commands::List { tag }) => {
            let snippets = storage::load_snippets()?;
            for s in snippets {
                if let Some(ref t) = tag {
                    if !s.tags.iter().any(|st| st.eq_ignore_ascii_case(t)) {
                        continue;
                    }
                }
                println!("• {}  [{}]\n  {}\n", s.title, s.tags.join(", "), s.command);
            }
            return Ok(());
        }
        None => run_tui()?,
    }

    Ok(())
}

fn run_tui() -> Result<()> {
    let snippets = storage::load_snippets()?;
    let mut app = App::new(snippets);

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let action = event_loop(&mut terminal, &mut app)?;

    // Drop guard explicitly before printing or running final commands
    drop(_guard);

    match action {
        ExitAction::Quit => {}
        ExitAction::Copy(cmd) => {
            let mut clipboard = Clipboard::new()?;
            clipboard.set_text(&cmd)?;
            println!("Copied to clipboard: {}", cmd);
        }
        ExitAction::Execute(cmd) => {
            println!("Running: {}\n", cmd);
            Command::new("powershell")
                .args(["-NoProfile", "-Command", &cmd])
                .status()?;
        }
    }

    Ok(())
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<ExitAction> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            // Crucial on Windows: Crossterm fires both Press and Release events.
            // Ignore Release events so keys don't type twice
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match &mut app.mode {
                AppMode::Normal => match (key.modifiers, key.code) {
                    (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        return Ok(ExitAction::Quit);
                    }
                    (_, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => app.next(),
                    (_, KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                        app.previous()
                    }

                    // Copy (Enter) or Execute (Ctrl+X)
                    (_, KeyCode::Enter) | (KeyModifiers::CONTROL, KeyCode::Char('x')) => {
                        let execute_after = key.modifiers.contains(KeyModifiers::CONTROL);
                        if let Some(snippet) = app.selected_snippet().cloned() {
                            let vars = App::extract_variables(&snippet.command);
                            if vars.is_empty() {
                                return Ok(if execute_after {
                                    ExitAction::Execute(snippet.command)
                                } else {
                                    ExitAction::Copy(snippet.command)
                                });
                            } else {
                                let len = vars.len();
                                app.mode = AppMode::PromptingVars {
                                    vars,
                                    values: vec![String::new(); len],
                                    current_idx: 0,
                                    execute_after,
                                };
                            }
                        }
                    }

                    // Open Add Modal (Ctrl+A)
                    (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                        app.mode = AppMode::Adding {
                            step: 0,
                            inputs: Default::default(),
                        };
                    }

                    // Delete Selected Snippet (Ctrl+D)
                    (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                        if let Some(&real_idx) = app.filtered_indices.get(app.selected) {
                            app.all_snippets.remove(real_idx);
                            storage::save_snippets(&app.all_snippets)?;
                            app.update_filter();
                        }
                    }

                    // Search typing
                    (_, KeyCode::Backspace) => {
                        app.search_query.pop();
                        app.update_filter();
                    }
                    (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                        app.search_query.push(c);
                        app.update_filter();
                    }
                    _ => {}
                },

                AppMode::PromptingVars {
                    vars,
                    values,
                    current_idx,
                    execute_after,
                } => match key.code {
                    KeyCode::Esc => app.mode = AppMode::Normal,
                    KeyCode::Backspace => {
                        values[*current_idx].pop();
                    }
                    KeyCode::Char(c) => {
                        values[*current_idx].push(c);
                    }
                    KeyCode::Enter => {
                        if *current_idx + 1 < vars.len() {
                            *current_idx += 1;
                        } else {
                            let real_idx = app.filtered_indices[app.selected];
                            let command = &app.all_snippets[real_idx].command;
                            let resolved = App::resolve_command(command, vars, values);
                            let exec = *execute_after;
                            return Ok(if exec {
                                ExitAction::Execute(resolved)
                            } else {
                                ExitAction::Copy(resolved)
                            });
                        }
                    }
                    _ => {}
                },

                AppMode::Adding { step, inputs } => match key.code {
                    KeyCode::Esc => app.mode = AppMode::Normal,
                    KeyCode::Backspace => {
                        inputs[*step].pop();
                    }
                    KeyCode::Char(c) => {
                        inputs[*step].push(c);
                    }
                    KeyCode::Enter => {
                        if *step < 3 {
                            *step += 1;
                        } else {
                            let tags = inputs[3]
                                .split(',')
                                .map(|t| t.trim().to_string())
                                .filter(|t| !t.is_empty())
                                .collect();
                            app.all_snippets.push(Snippet {
                                title: inputs[0].clone(),
                                command: inputs[1].clone(),
                                description: inputs[2].clone(),
                                tags,
                            });
                            storage::save_snippets(&app.all_snippets)?;
                            app.update_filter();
                            app.mode = AppMode::Normal;
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}
