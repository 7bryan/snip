use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snippet {
    pub title: String,
    pub command: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnippetFile {
    pub snippets: Vec<Snippet>,
}

#[derive(Debug, PartialEq)]
pub enum AppMode {
    Normal,
    PromptingVars {
        vars: Vec<String>,
        values: Vec<String>,
        current_idx: usize,
        execute_after: bool,
    },
    Adding {
        step: usize,
        inputs: [String; 4],
    },
}
