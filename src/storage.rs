use crate::model::{Snippet, SnippetFile};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not find Windows AppData/Roaming config directory")?
        .join("snip");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create directory {:?}", config_dir))?;
    }

    Ok(config_dir.join("snippets.toml"))
}

pub fn load_snippets() -> Result<Vec<Snippet>> {
    let path = get_config_path()?;

    if !path.exists() {
        let defaults = default_snippets();
        save_snippets(&defaults)?;
        return Ok(defaults);
    }

    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {:?}", path))?;

    let parsed: SnippetFile =
        toml::from_str(&content).with_context(|| format!("Invalid TOML syntax in {:?}", path))?;

    Ok(parsed.snippets)
}

pub fn save_snippets(snippets: &[Snippet]) -> Result<()> {
    let path = get_config_path()?;
    let wrapper = SnippetFile {
        snippets: snippets.to_vec(),
    };
    let toml_string =
        toml::to_string_pretty(&wrapper).with_context(|| format!("Failed to write {:?}", path))?;
    fs::write(&path, toml_string).with_context(|| format!("Failed to write {:?}", path))?;
    Ok(())
}

fn default_snippets() -> Vec<Snippet> {
    vec![
        Snippet {
            title: "Find process using a port (Windows)".into(),
            command: "netstat -ano | findstr :{{port}}".into(),
            description: "Finds the PID of whichever Windows app is using a local port.".into(),
            tags: vec!["windows".into(), "network".into()],
        },
        Snippet {
            title: "Force kill process by PID".into(),
            command: "taskkill /PID {{pid}} /F".into(),
            description: "Immediately terminates a process using its PID.".into(),
            tags: vec!["windows".into(), "system".into()],
        },
        Snippet {
            title: "Git undo last commit (keep staged)".into(),
            command: "git reset --soft HEAD~1".into(),
            description: "Moves HEAD back one commit while keeping your changes staged.".into(),
            tags: vec!["git".into()],
        },
        Snippet {
            title: "Docker prune unused data".into(),
            command: "docker system prune -af".into(),
            description: "Removes all stopped containers and unused images to free disk space."
                .into(),
            tags: vec!["docker".into()],
        },
    ]
}
