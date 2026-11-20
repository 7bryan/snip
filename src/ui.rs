use crate::model::{App, AppMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let search_text = format!(" > {}_", app.search_query);
    let search_bar = Paragraph::new(search_text)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" snip — Search (title, command, or tag) "),
        );
    frame.render_widget(search_bar, chunks[0]);

    // 3. Middle Split: Left list (45%) | Right preview (55%)
    let main_panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    draw_snippet_list(frame, app, main_panes[0]);
    draw_preview_pane(frame, app, main_panes[1]);

    // 4. Bottom Footer
    let footer_text = " [Enter] Copy  [Ctrl+X] Run  [Ctrl+A] New  [Ctrl+D] Delete  [Esc] Quit";
    let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[2]);

    // 5. Render Popup Modal if in PromptingVars or Adding mode
    match &app.mode {
        AppMode::PromptingVars {
            vars,
            values,
            current_idx,
            ..
        } => {
            let area = centered_rect(60, 30, frame.area());
            frame.render_widget(Clear, area); // Clears background text underneath modal

            let var_name = &vars[*current_idx];
            let current_val = &values[*current_idx];
            let content = vec![
                Line::from(format!("Variable {} of {}:", current_idx + 1, vars.len())),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        format!(" {} = ", var_name),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!("{}_", current_val)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Press [Enter] to confirm, [Esc] to cancel",
                    Style::default().fg(Color::DarkGray),
                )),
            ];

            let popup = Paragraph::new(content).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Fill Variable "),
            );
            frame.render_widget(popup, area);
        }
        AppMode::Adding { step, inputs } => {
            let area = centered_rect(65, 45, frame.area());
            frame.render_widget(Clear, area);

            let labels = [
                "Title",
                "Command (use {{var}} for inputs)",
                "Description",
                "Tags (comma-separated)",
            ];
            let mut lines = vec![
                Line::from("Create New Snippet ([Enter] next, [Esc] cancel):"),
                Line::from(""),
            ];

            for (i, label) in labels.iter().enumerate() {
                let marker = if i == *step { "> " } else { "  " };
                let style = if i == *step {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                lines.push(Line::from(Span::styled(
                    format!("{}{}: {}", marker, label, inputs[i]),
                    style,
                )));
            }

            let popup = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .title(" Add Snippet "),
            );
            frame.render_widget(popup, area);
        }
        AppMode::Normal => {}
    }
}

fn draw_snippet_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .map(|&idx| {
            let s = &app.all_snippets[idx];
            let tags = if s.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", s.tags.join(", "))
            };
            ListItem::new(Line::from(vec![
                Span::raw(&s.title),
                Span::styled(tags, Style::default().fg(Color::Cyan)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Snippets ({}) ", app.filtered_indices.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !app.filtered_indices.is_empty() {
        state.select(Some(app.selected));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_preview_pane(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Preview ");

    if let Some(snippet) = app.selected_snippet() {
        let vars = App::extract_variables(&snippet.command);
        let mut lines = vec![
            Line::from(Span::styled(
                &snippet.title,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Command:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                &snippet.command,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Description:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(snippet.description.as_str()),
        ];

        if !vars.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("Variables required: {}", vars.join(", ")),
                Style::default().fg(Color::Yellow),
            )));
        }

        let para = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(block);
        frame.render_widget(para, area);
    } else {
        let empty = Paragraph::new("No matching snippets. Press Ctrl+A to create one.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(empty, area);
    }
}

/// Helper to center modal popups on screen
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
