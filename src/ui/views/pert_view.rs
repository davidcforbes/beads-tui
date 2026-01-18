//! PERT chart view for dependency and critical path visualization

use crate::beads::models::Issue;
use crate::models::pert_layout::PertGraph;
use crate::ui::widgets::pert_chart::{PertChart, PertChartConfig};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// State for PERT chart view
#[derive(Debug)]
pub struct PertViewState {
    /// All issues to display
    issues: Vec<Issue>,
    /// Selected issue index in topological order
    selected_issue: usize,
    /// PERT chart configuration
    config: PertChartConfig,
    /// Default duration for issues without estimates (in hours)
    default_duration: f64,
    /// Cached PERT graph
    graph: Option<PertGraph>,
}

impl PertViewState {
    /// Create a new PERT view state
    pub fn new(issues: Vec<Issue>) -> Self {
        let mut state = Self {
            issues,
            selected_issue: 0,
            config: PertChartConfig::default(),
            default_duration: 24.0, // 1 day default
            graph: None,
        };
        state.rebuild_graph();
        state
    }

    /// Rebuild the PERT graph from current issues
    fn rebuild_graph(&mut self) {
        self.graph = Some(PertGraph::new(&self.issues, self.default_duration));

        // Update selected node in config if we have a graph
        if let Some(graph) = &self.graph {
            if !graph.topological_order.is_empty()
                && self.selected_issue < graph.topological_order.len()
            {
                self.config.selected_node =
                    Some(graph.topological_order[self.selected_issue].clone());
            }
        }
    }

    /// Get the issues for display
    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    /// Set the issues
    pub fn set_issues(&mut self, issues: Vec<Issue>) {
        self.issues = issues;
        self.rebuild_graph();

        // Adjust selection if needed
        if let Some(graph) = &self.graph {
            if self.selected_issue >= graph.topological_order.len()
                && !graph.topological_order.is_empty()
            {
                self.selected_issue = graph.topological_order.len() - 1;
            }
        }
    }

    /// Get the selected issue
    pub fn selected_issue(&self) -> Option<&Issue> {
        if let Some(graph) = &self.graph {
            if self.selected_issue < graph.topological_order.len() {
                let node_id = &graph.topological_order[self.selected_issue];
                return self.issues.iter().find(|issue| &issue.id == node_id);
            }
        }
        None
    }

    /// Move selection to next issue (in topological order)
    pub fn next_issue(&mut self) {
        if let Some(graph) = &self.graph {
            if !graph.topological_order.is_empty() {
                self.selected_issue = (self.selected_issue + 1) % graph.topological_order.len();
                self.config.selected_node =
                    Some(graph.topological_order[self.selected_issue].clone());
            }
        }
    }

    /// Move selection to previous issue (in topological order)
    pub fn previous_issue(&mut self) {
        if let Some(graph) = &self.graph {
            if !graph.topological_order.is_empty() {
                self.selected_issue = if self.selected_issue == 0 {
                    graph.topological_order.len() - 1
                } else {
                    self.selected_issue - 1
                };
                self.config.selected_node =
                    Some(graph.topological_order[self.selected_issue].clone());
            }
        }
    }

    /// Pan the view
    pub fn pan(&mut self, dx: i32, dy: i32) {
        self.config.pan(dx, dy);
    }

    /// Zoom in
    pub fn zoom_in(&mut self) {
        self.config.adjust_zoom(1.2);
    }

    /// Zoom out
    pub fn zoom_out(&mut self) {
        self.config.adjust_zoom(0.8);
    }

    /// Toggle critical path highlighting
    pub fn toggle_critical_path(&mut self) {
        self.config.toggle_critical_path();
    }

    /// Toggle focus mode
    pub fn toggle_focus_mode(&mut self) {
        self.config.toggle_focus_mode();
    }

    /// Cycle focus direction
    pub fn cycle_focus_direction(&mut self) {
        self.config.focus_direction = match self.config.focus_direction.as_str() {
            "upstream" => "downstream".to_string(),
            "downstream" => "both".to_string(),
            "both" => "upstream".to_string(),
            _ => "both".to_string(),
        };
    }

    /// Increase focus depth
    pub fn increase_focus_depth(&mut self) {
        self.config.focus_depth = (self.config.focus_depth + 1).min(10);
    }

    /// Decrease focus depth
    pub fn decrease_focus_depth(&mut self) {
        self.config.focus_depth = self.config.focus_depth.saturating_sub(1).max(1);
    }

    /// Reset view (pan and zoom)
    pub fn reset_view(&mut self) {
        self.config.offset_x = 0;
        self.config.offset_y = 0;
        self.config.zoom = 1.0;
    }

    /// Get the PERT configuration
    pub fn config(&self) -> &PertChartConfig {
        &self.config
    }

    /// Get the PERT graph
    pub fn graph(&self) -> Option<&PertGraph> {
        self.graph.as_ref()
    }
}

use ratatui::widgets::StatefulWidget;

/// PERT chart view widget
pub struct PertView;

impl StatefulWidget for PertView {
    type State = PertViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Self::render_with_state(area, buf, state);
    }
}

impl Widget for PertView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("PERT Chart")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let inner = block.inner(area);
        block.render(area, buf);

        let text = vec![
            Line::from(Span::styled(
                "PERT Chart View",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Press Up/Down or j/k to navigate nodes"),
            Line::from("Press +/- to zoom in/out"),
            Line::from("Press c to configure chart settings"),
        ];

        let paragraph = Paragraph::new(text);
        paragraph.render(inner, buf);
    }
}

impl PertView {
    /// Create a new PERT view
    pub fn new() -> Self {
        Self
    }

    /// Render the PERT chart with state
    pub fn render_with_state(area: Rect, buf: &mut Buffer, state: &PertViewState) {
        if state.issues.is_empty() {
            let block = Block::default().title("PERT Chart").borders(Borders::ALL);
            let inner = block.inner(area);
            block.render(area, buf);

            let text = Line::from("No issues to display");
            let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
            paragraph.render(inner, buf);
            return;
        }

        // Check for graph
        if let Some(graph) = &state.graph {
            // Check for cycles
            if graph.cycle_detection.has_cycle {
                let block = Block::default()
                    .title("PERT Chart - Cycle Detected")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red));
                let inner = block.inner(area);
                block.render(area, buf);

                let text = vec![
                    Line::from(Span::styled(
                        "[!] Dependency Cycle Detected",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from("Cannot compute PERT chart with circular dependencies."),
                    Line::from("Please fix the dependency graph first."),
                ];

                let paragraph = Paragraph::new(text);
                paragraph.render(inner, buf);
                return;
            }

            // Render the PERT chart
            let pert_chart = PertChart::new(graph).config(state.config.clone());
            pert_chart.render(area, buf);
        } else {
            // No graph available
            let block = Block::default().title("PERT Chart").borders(Borders::ALL);
            let inner = block.inner(area);
            block.render(area, buf);

            let text = Line::from("Building dependency graph...");
            let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
            paragraph.render(inner, buf);
        }
    }
}

impl Default for PertView {
    fn default() -> Self {
        Self::new()
    }
}

// Event handling implementation
use super::ViewEventHandler;
use crate::models::AppState;
use crate::config::Action;
use crossterm::event::{KeyEvent, MouseEvent};

impl ViewEventHandler for PertViewState {
    fn handle_key_event(app: &mut AppState, key: KeyEvent) -> bool {
        let action = app.config.keybindings.find_action(&key.code, &key.modifiers);

        // Handle notification dismissal with Esc
        if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
            app.clear_notification();
            return true;
        }

        // Currently no view-specific actions for PERT view
        false
    }

    fn view_name() -> &'static str {
        "PertView"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueStatus, IssueType, Priority};
    use chrono::Utc;

    fn create_test_issue(id: &str, title: &str, dependencies: Vec<String>) -> Issue {
        Issue {
            id: id.to_string(),
            title: title.to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            assignee: None,
            labels: vec![],
            description: None,
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            dependencies,
            blocks: vec![],
            notes: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn test_pert_view_state_new() {
        let issues = vec![create_test_issue("TEST-1", "Test Issue", vec![])];
        let state = PertViewState::new(issues.clone());

        assert_eq!(state.issues().len(), 1);
        assert_eq!(state.selected_issue, 0);
        assert!(state.graph.is_some());
    }

    #[test]
    fn test_navigation() {
        let issues = vec![
            create_test_issue("TEST-1", "Issue 1", vec![]),
            create_test_issue("TEST-2", "Issue 2", vec!["TEST-1".to_string()]),
        ];
        let mut state = PertViewState::new(issues);

        // Next issue
        state.next_issue();
        assert_eq!(state.selected_issue, 1);

        // Next issue (wraps)
        state.next_issue();
        assert_eq!(state.selected_issue, 0);

        // Previous issue (wraps)
        state.previous_issue();
        assert_eq!(state.selected_issue, 1);
    }

    #[test]
    fn test_pan() {
        let mut state = PertViewState::new(vec![]);

        assert_eq!(state.config.offset_x, 0);
        assert_eq!(state.config.offset_y, 0);

        state.pan(10, 5);
        assert_eq!(state.config.offset_x, 10);
        assert_eq!(state.config.offset_y, 5);

        state.pan(-5, -3);
        assert_eq!(state.config.offset_x, 5);
        assert_eq!(state.config.offset_y, 2);
    }

    #[test]
    fn test_zoom() {
        let mut state = PertViewState::new(vec![]);

        assert_eq!(state.config.zoom, 1.0);

        // Zoom in
        state.zoom_in();
        assert!(state.config.zoom > 1.0);

        // Zoom out
        state.zoom_out();
        assert!((state.config.zoom - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_critical_path_toggle() {
        let mut state = PertViewState::new(vec![]);

        let initial = state.config.show_critical_path;
        state.toggle_critical_path();
        assert_eq!(state.config.show_critical_path, !initial);
        state.toggle_critical_path();
        assert_eq!(state.config.show_critical_path, initial);
    }

    #[test]
    fn test_focus_mode() {
        let mut state = PertViewState::new(vec![]);

        assert!(!state.config.focus_mode);
        state.toggle_focus_mode();
        assert!(state.config.focus_mode);
        state.toggle_focus_mode();
        assert!(!state.config.focus_mode);
    }

    #[test]
    fn test_focus_direction_cycle() {
        let mut state = PertViewState::new(vec![]);

        assert_eq!(state.config.focus_direction, "both");
        state.cycle_focus_direction();
        assert_eq!(state.config.focus_direction, "upstream");
        state.cycle_focus_direction();
        assert_eq!(state.config.focus_direction, "downstream");
        state.cycle_focus_direction();
        assert_eq!(state.config.focus_direction, "both");
    }

    #[test]
    fn test_focus_depth() {
        let mut state = PertViewState::new(vec![]);

        assert_eq!(state.config.focus_depth, 1);

        state.increase_focus_depth();
        assert_eq!(state.config.focus_depth, 2);

        state.increase_focus_depth();
        assert_eq!(state.config.focus_depth, 3);

        state.decrease_focus_depth();
        assert_eq!(state.config.focus_depth, 2);

        state.decrease_focus_depth();
        assert_eq!(state.config.focus_depth, 1);

        // Can't go below 1
        state.decrease_focus_depth();
        assert_eq!(state.config.focus_depth, 1);
    }

    #[test]
    fn test_reset_view() {
        let mut state = PertViewState::new(vec![]);

        state.pan(50, 30);
        state.zoom_in();
        state.zoom_in();

        assert_ne!(state.config.offset_x, 0);
        assert_ne!(state.config.offset_y, 0);
        assert_ne!(state.config.zoom, 1.0);

        state.reset_view();
        assert_eq!(state.config.offset_x, 0);
        assert_eq!(state.config.offset_y, 0);
        assert_eq!(state.config.zoom, 1.0);
    }

    #[test]
    fn test_set_issues_updates_selection() {
        let mut state = PertViewState::new(vec![
            create_test_issue("TEST-1", "Issue 1", vec![]),
            create_test_issue("TEST-2", "Issue 2", vec![]),
            create_test_issue("TEST-3", "Issue 3", vec![]),
        ]);

        state.selected_issue = 2;

        // Reduce issues - selection should adjust
        state.set_issues(vec![create_test_issue("TEST-1", "Issue 1", vec![])]);
        assert_eq!(state.selected_issue, 0);
    }

    #[test]
    fn test_dependency_chain() {
        let issues = vec![
            create_test_issue("TEST-1", "Root", vec![]),
            create_test_issue("TEST-2", "Dep 1", vec!["TEST-1".to_string()]),
            create_test_issue("TEST-3", "Dep 2", vec!["TEST-2".to_string()]),
        ];
        let state = PertViewState::new(issues);

        assert!(state.graph.is_some());
        let graph = state.graph.as_ref().unwrap();
        assert!(!graph.cycle_detection.has_cycle);
        assert_eq!(graph.topological_order.len(), 3);
    }

    #[test]
    fn test_selected_issue() {
        let issues = vec![
            create_test_issue("TEST-1", "Issue 1", vec![]),
            create_test_issue("TEST-2", "Issue 2", vec!["TEST-1".to_string()]),
        ];
        let mut state = PertViewState::new(issues);

        let selected = state.selected_issue();
        assert!(selected.is_some());

        state.next_issue();
        let selected = state.selected_issue();
        assert!(selected.is_some());
    }

    #[test]
    fn test_empty_issues() {
        let state = PertViewState::new(vec![]);

        assert_eq!(state.issues().len(), 0);
        assert!(state.selected_issue().is_none());
        assert!(state.graph.is_some());
    }
}
