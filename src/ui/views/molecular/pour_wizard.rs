//! Pour Wizard for instantiating formula templates

use super::formula_browser::Formula;
use crate::ui::widgets::{Form, FormField, FormState};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

/// Steps in the Pour Wizard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PourStep {
    Variables,
    Preview,
    Execution,
}

/// State for the Pour Wizard
#[derive(Debug)]
pub struct PourWizardState {
    pub formula: Formula,
    pub step: PourStep,
    pub form_state: FormState,
    pub is_executing: bool,
    pub result_message: Option<String>,
}

impl PourWizardState {
    pub fn new(formula: Formula) -> Self {
        let fields = formula
            .variables
            .iter()
            .map(|v| FormField::text(v, v).required())
            .collect();

        Self {
            formula,
            step: PourStep::Variables,
            form_state: FormState::new(fields),
            is_executing: false,
            result_message: None,
        }
    }

    pub fn next_step(&mut self) {
        match self.step {
            PourStep::Variables => {
                if self.form_state.validate() {
                    self.step = PourStep::Preview;
                }
            }
            PourStep::Preview => {
                self.step = PourStep::Execution;
            }
            PourStep::Execution => {}
        }
    }

    pub fn prev_step(&mut self) {
        match self.step {
            PourStep::Variables => {}
            PourStep::Preview => {
                self.step = PourStep::Variables;
            }
            PourStep::Execution => {
                if !self.is_executing {
                    self.step = PourStep::Preview;
                }
            }
        }
    }
}

/// Pour Wizard Widget
pub struct PourWizard<'a> {
    block_style: Style,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Default for PourWizard<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PourWizard<'a> {
    pub fn new() -> Self {
        Self {
            block_style: Style::default().fg(Color::Cyan),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> StatefulWidget for PourWizard<'a> {
    type State = PourWizardState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title/Step Indicator
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Help/Navigation
            ])
            .split(area);

        // Render Title
        let step_text = match state.step {
            PourStep::Variables => "Step 1: Fill Variables",
            PourStep::Preview => "Step 2: Preview Issues",
            PourStep::Execution => "Step 3: Pouring...",
        };

        let title = Paragraph::new(format!(
            "Pouring Formula: {} - {}",
            state.formula.name, step_text
        ))
        .style(self.block_style.add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        title.render(chunks[0], buf);

        // Render Content based on step
        match state.step {
            PourStep::Variables => {
                let form = Form::new()
                    .title("Variables")
                    .focused_style(Style::default().fg(Color::Yellow));
                StatefulWidget::render(form, chunks[1], buf, &mut state.form_state);
            }
            PourStep::Preview => {
                let mut preview_lines = vec![
                    Line::from(Span::styled(
                        "The following issues will be created:",
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                ];

                // Simple preview logic
                let title_val = state.form_state.get_value("title").unwrap_or("New Issue");
                preview_lines.push(Line::from(vec![
                    Span::styled("  • ", Style::default().fg(Color::Green)),
                    Span::raw(format!("[{}] {}", state.formula.name, title_val)),
                ]));

                let preview = Paragraph::new(preview_lines).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Preview")
                        .style(self.block_style),
                );
                preview.render(chunks[1], buf);
            }
            PourStep::Execution => {
                let msg = state
                    .result_message
                    .as_deref()
                    .unwrap_or("Pouring issues into database...");
                let execution = Paragraph::new(msg).alignment(Alignment::Center).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Execution Status")
                        .style(self.block_style),
                );
                execution.render(chunks[1], buf);
            }
        }

        // Render Navigation Help
        let help_text = match state.step {
            PourStep::Variables => "Tab: Next Field | Enter: Preview | Esc: Cancel",
            PourStep::Preview => "Enter: Confirm Pour | Backspace: Back | Esc: Cancel",
            PourStep::Execution => "Esc: Close",
        };
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        help.render(chunks[2], buf);
    }
}
