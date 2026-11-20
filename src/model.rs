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

pub struct App {
    pub all_snippets: Vec<Snippet>,
    pub filtered_indices: Vec<usize>,
    pub search_query: String,
    pub selected: usize,
    pub mode: AppMode,
    pub status_message: Option<String>,
}

impl App {
    pub fn new(snippets: Vec<Snippet>) -> Self {
        let filtered_indices = (0..snippets.len()).collect();
        Self {
            all_snippets: Vec::new(),
            filtered_indices,
            search_query: String::new(),
            selected: 0,
            mode: AppMode::Normal,
            status_message: None,
        }
    }

    pub fn update_filter(&mut self) {
        let q = self.search_query.to_lowercase();
        self.filtered_indices = self
            .all_snippets
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                q.is_empty()
                    || s.title.to_lowercase().contains(&q)
                    || s.command.to_lowercase().contains(&q)
                    || s.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .map(|(idx, _)| idx)
            .collect();

        if self.filtered_indices.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered_indices.len() {
            self.selected = self.filtered_indices.len() - 1;
        }
    }

    pub fn selected_snippet(&self) -> Option<&Snippet> {
        self.filtered_indices
            .get(self.selected)
            .and_then(|&idx| self.all_snippets.get(idx))
    }

    pub fn next(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = (self.selected + 1) % self.filtered_indices.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected == 0 {
                self.selected = self.filtered_indices.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }
}
