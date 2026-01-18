//! Formula Browser for the Molecular Chemistry UI
//!
//! Displays a list of available formula templates that can be instantiated (poured).

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

/// Represents a Formula Template
#[derive(Debug, Clone)]
pub struct Formula {
    pub name: String,
    pub description: String,
    pub variables: Vec<String>,
}

/// State for the Formula Browser
#[derive(Debug)]
pub struct FormulaBrowserState {
    list_state: ListState,
    search_query: String,
    is_searching: bool,
}

impl Default for FormulaBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

impl FormulaBrowserState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            list_state,
            search_query: String::new(),
            is_searching: false,
        }
    }

    pub fn is_searching(&self) -> bool {
        self.is_searching
    }

    pub fn set_searching(&mut self, searching: bool) {
        self.is_searching = searching;
    }

    pub fn insert_char(&mut self, c: char) {
        self.search_query.push(c);
    }

    pub fn backspace(&mut self) {
        self.search_query.pop();
    }

    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub fn select_next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_previous(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
    }
}

/// Formula Browser Widget
pub struct FormulaBrowser<'a> {
    formulas: Vec<Formula>,
    block_style: Style,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Default for FormulaBrowser<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> FormulaBrowser<'a> {
    pub fn new() -> Self {
        Self {
            formulas: vec![
                Formula {
                    name: "Feature".to_string(),
                    description: "Standard feature template with estimate and labels".to_string(),
                    variables: vec![
                        "title".to_string(),
                        "description".to_string(),
                        "estimate".to_string(),
                    ],
                },
                Formula {
                    name: "Bug".to_string(),
                    description: "Bug report template with steps to reproduce and priority"
                        .to_string(),
                    variables: vec![
                        "title".to_string(),
                        "repro_steps".to_string(),
                        "priority".to_string(),
                    ],
                },
                Formula {
                    name: "Chore".to_string(),
                    description: "Maintenance task or internal improvement".to_string(),
                    variables: vec!["title".to_string(), "details".to_string()],
                },
                Formula {
                    name: "Release".to_string(),
                    description: "Release checklist and deployment steps".to_string(),
                    variables: vec!["version".to_string(), "date".to_string()],
                },
            ],
            block_style: Style::default().fg(Color::Cyan),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn formulas(mut self, formulas: Vec<Formula>) -> Self {
        self.formulas = formulas;
        self
    }

    fn render_list(&self, area: Rect, buf: &mut Buffer, state: &mut FormulaBrowserState) {
        let query = state.search_query().to_lowercase();
        let filtered_formulas: Vec<&Formula> = self
            .formulas
            .iter()
            .filter(|f| {
                f.name.to_lowercase().contains(&query)
                    || f.description.to_lowercase().contains(&query)
            })
            .collect();

        let items: Vec<ListItem> = if filtered_formulas.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No formulas match",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))]
        } else {
            filtered_formulas
                .iter()
                .map(|f| {
                    ListItem::new(vec![
                        Line::from(Span::styled(
                            &f.name,
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            &f.description,
                            Style::default().fg(Color::DarkGray),
                        )),
                    ])
                })
                .collect()
        };

        let title = if state.search_query().is_empty() {
            "Formulas".to_string()
        } else {
            format!("Formulas (Search: {})", state.search_query())
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
            .highlight_symbol(">> ");

        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }

    fn render_details(&self, area: Rect, buf: &mut Buffer, state: &FormulaBrowserState) {
        let query = state.search_query().to_lowercase();
        let filtered_formulas: Vec<&Formula> = self
            .formulas
            .iter()
            .filter(|f| {
                f.name.to_lowercase().contains(&query)
                    || f.description.to_lowercase().contains(&query)
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Formula Details")
            .style(self.block_style);

        if let Some(idx) = state.selected() {
            if let Some(formula) = filtered_formulas.get(idx) {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Name
                        Constraint::Length(4), // Description
                        Constraint::Min(0),    // Variables
                    ])
                    .margin(1)
                    .split(area);

                // Render Name
                let name = Paragraph::new(Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Gray)),
                    Span::styled(&formula.name, Style::default().add_modifier(Modifier::BOLD)),
                ]));
                name.render(chunks[0], buf);

                // Render Description
                let desc = Paragraph::new(Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(Color::Gray)),
                    Span::raw(&formula.description),
                ]))
                .wrap(Wrap { trim: true });
                desc.render(chunks[1], buf);

                // Render Variables
                let var_lines: Vec<Line> = formula
                    .variables
                    .iter()
                    .map(|v| {
                        Line::from(vec![
                            Span::styled("  • ", Style::default().fg(Color::Cyan)),
                            Span::raw(v),
                        ])
                    })
                    .collect();

                let mut variables_content = vec![Line::from(Span::styled(
                    "Variables:",
                    Style::default().fg(Color::Yellow),
                ))];
                variables_content.extend(var_lines);

                let vars = Paragraph::new(variables_content);
                vars.render(chunks[2], buf);
            }
        } else {
            let placeholder = Paragraph::new("Select a formula to see details")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(ratatui::layout::Alignment::Center);
            placeholder.render(block.inner(area), buf);
        }

        block.render(area, buf);
    }
}

impl<'a> StatefulWidget for FormulaBrowser<'a> {
    type State = FormulaBrowserState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
                Constraint::Length(1), // Help
            ])
            .split(area);

        // Render Title
        let title = Paragraph::new("Formula Browser")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        title.render(chunks[0], buf);

        // Content area: List and Details
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[1]);

        self.render_list(content_chunks[0], buf, state);
        self.render_details(content_chunks[1], buf, state);

        // Render Help
        let help =
            Paragraph::new("j/k: Navigate | Enter: Pour Formula | /: Search | Esc: Clear Search")
                .style(Style::default().fg(Color::DarkGray));
        help.render(chunks[2], buf);
    }
}

// Event handling implementation
use super::super::ViewEventHandler;
use crate::models::AppState;
use crate::config::Action;
use crossterm::event::{KeyEvent, MouseEvent};

impl ViewEventHandler for FormulaBrowserState {
    fn handle_key_event(app: &mut AppState, key: KeyEvent) -> bool {
        let action = app.config.keybindings.find_action(&key.code, &key.modifiers);

        // Handle notification dismissal with Esc
        if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
            app.clear_notification();
            return true;
        }

        // Currently no view-specific actions for Molecular/Formula Browser view
        false
    }

    fn view_name() -> &'static str {
        "FormulaBrowserView"
    }
}
