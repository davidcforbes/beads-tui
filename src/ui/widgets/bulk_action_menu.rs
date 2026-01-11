//! Bulk action menu widget for multi-issue operations

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

/// Bulk action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulkAction {
    /// Close selected issues
    Close,
    /// Reopen selected issues
    Reopen,
    /// Change status to in_progress
    SetInProgress,
    /// Change status to blocked
    SetBlocked,
    /// Change priority
    SetPriority,
    /// Add labels
    AddLabels,
    /// Remove labels
    RemoveLabels,
    /// Set assignee
    SetAssignee,
    /// Clear assignee
    ClearAssignee,
    /// Delete issues
    Delete,
    /// Export to file
    Export,
    /// Cancel operation
    Cancel,
}

impl BulkAction {
    /// Get display name for the action
    pub fn display_name(&self) -> &str {
        match self {
            Self::Close => "Close selected issues",
            Self::Reopen => "Reopen selected issues",
            Self::SetInProgress => "Set status: In Progress",
            Self::SetBlocked => "Set status: Blocked",
            Self::SetPriority => "Change priority...",
            Self::AddLabels => "Add labels...",
            Self::RemoveLabels => "Remove labels...",
            Self::SetAssignee => "Set assignee...",
            Self::ClearAssignee => "Clear assignee",
            Self::Delete => "Delete selected issues",
            Self::Export => "Export to file...",
            Self::Cancel => "Cancel",
        }
    }

    /// Get icon/symbol for the action
    pub fn icon(&self) -> &str {
        match self {
            Self::Close => "✓",
            Self::Reopen => "↻",
            Self::SetInProgress => "▶",
            Self::SetBlocked => "⊘",
            Self::SetPriority => "!",
            Self::AddLabels => "+",
            Self::RemoveLabels => "-",
            Self::SetAssignee => "@",
            Self::ClearAssignee => "∅",
            Self::Delete => "✗",
            Self::Export => "↓",
            Self::Cancel => "←",
        }
    }

    /// Get color for the action
    pub fn color(&self) -> Color {
        match self {
            Self::Close => Color::Green,
            Self::Reopen => Color::Cyan,
            Self::SetInProgress => Color::Yellow,
            Self::SetBlocked => Color::Red,
            Self::SetPriority => Color::Magenta,
            Self::AddLabels | Self::RemoveLabels => Color::Blue,
            Self::SetAssignee | Self::ClearAssignee => Color::Cyan,
            Self::Delete => Color::Red,
            Self::Export => Color::Green,
            Self::Cancel => Color::Gray,
        }
    }

    /// Check if action is destructive (requires confirmation)
    pub fn is_destructive(&self) -> bool {
        matches!(self, Self::Delete | Self::Close)
    }

    /// Check if action requires additional input
    pub fn requires_input(&self) -> bool {
        matches!(
            self,
            Self::SetPriority
                | Self::AddLabels
                | Self::RemoveLabels
                | Self::SetAssignee
                | Self::Export
        )
    }

    /// Get all available actions
    pub fn all() -> Vec<BulkAction> {
        vec![
            Self::Close,
            Self::Reopen,
            Self::SetInProgress,
            Self::SetBlocked,
            Self::SetPriority,
            Self::AddLabels,
            Self::RemoveLabels,
            Self::SetAssignee,
            Self::ClearAssignee,
            Self::Delete,
            Self::Export,
            Self::Cancel,
        ]
    }
}

/// Bulk action menu state
#[derive(Debug, Clone)]
pub struct BulkActionMenuState {
    actions: Vec<BulkAction>,
    list_state: ListState,
    selected_count: usize,
    confirmed_action: Option<BulkAction>,
}

impl BulkActionMenuState {
    /// Create a new bulk action menu state
    pub fn new(selected_count: usize) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            actions: BulkAction::all(),
            list_state,
            selected_count,
            confirmed_action: None,
        }
    }

    /// Create with custom action list
    pub fn with_actions(actions: Vec<BulkAction>, selected_count: usize) -> Self {
        let mut list_state = ListState::default();
        if !actions.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            actions,
            list_state,
            selected_count,
            confirmed_action: None,
        }
    }

    /// Get the number of selected issues
    pub fn selected_count(&self) -> usize {
        self.selected_count
    }

    /// Set the number of selected issues
    pub fn set_selected_count(&mut self, count: usize) {
        self.selected_count = count;
    }

    /// Get available actions
    pub fn actions(&self) -> &[BulkAction] {
        &self.actions
    }

    /// Get currently highlighted action
    pub fn highlighted_action(&self) -> Option<BulkAction> {
        self.list_state
            .selected()
            .and_then(|i| self.actions.get(i).copied())
    }

    /// Get confirmed action
    pub fn confirmed_action(&self) -> Option<BulkAction> {
        self.confirmed_action
    }

    /// Clear confirmed action
    pub fn clear_confirmed(&mut self) {
        self.confirmed_action = None;
    }

    /// Select next action
    pub fn select_next(&mut self) {
        let count = self.actions.len();
        if count == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Select previous action
    pub fn select_previous(&mut self) {
        let count = self.actions.len();
        if count == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Confirm current selection
    pub fn confirm_selection(&mut self) -> Option<BulkAction> {
        if let Some(action) = self.highlighted_action() {
            self.confirmed_action = Some(action);
            Some(action)
        } else {
            None
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.list_state.select(Some(0));
        self.confirmed_action = None;
    }
}

impl Default for BulkActionMenuState {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Bulk action menu widget
pub struct BulkActionMenu<'a> {
    title: Option<&'a str>,
    style: Style,
    selected_style: Style,
    block: Option<Block<'a>>,
    show_icons: bool,
    show_count: bool,
}

impl<'a> BulkActionMenu<'a> {
    /// Create a new bulk action menu
    pub fn new() -> Self {
        Self {
            title: Some("Bulk Actions"),
            style: Style::default(),
            selected_style: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            block: None,
            show_icons: true,
            show_count: true,
        }
    }

    /// Set title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Set style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set selected action style
    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Show or hide action icons
    pub fn show_icons(mut self, show: bool) -> Self {
        self.show_icons = show;
        self
    }

    /// Show or hide selected count
    pub fn show_count(mut self, show: bool) -> Self {
        self.show_count = show;
        self
    }
}

impl<'a> Default for BulkActionMenu<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for BulkActionMenu<'a> {
    type State = BulkActionMenuState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Build title
        let title = if self.show_count && state.selected_count > 0 {
            format!(
                "{} ({} selected)",
                self.title.unwrap_or("Bulk Actions"),
                state.selected_count
            )
        } else {
            self.title.unwrap_or("Bulk Actions").to_string()
        };

        // Build block
        let block = if let Some(mut block) = self.block {
            block = block.title(title);
            block
        } else {
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(self.style)
        };

        let inner = block.inner(area);
        block.render(area, buf);

        // Build list items
        let items: Vec<ListItem> = state
            .actions
            .iter()
            .map(|action| {
                let mut spans = Vec::new();

                // Add icon if enabled
                if self.show_icons {
                    spans.push(Span::styled(
                        format!("{} ", action.icon()),
                        Style::default().fg(action.color()),
                    ));
                }

                // Add action name
                let name = action.display_name();
                let name_style = if action.is_destructive() {
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                spans.push(Span::styled(name, name_style));

                // Add warning for destructive actions
                if action.is_destructive() {
                    spans.push(Span::styled(
                        " [!]",
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        // Render list
        let list = if items.is_empty() {
            let empty_items = vec![ListItem::new(Line::from(Span::styled(
                "No actions available",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))];
            List::new(empty_items)
        } else {
            List::new(items)
                .highlight_style(self.selected_style)
                .highlight_symbol("> ")
        };

        StatefulWidget::render(list, inner, buf, &mut state.list_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_action_display_name() {
        assert_eq!(BulkAction::Close.display_name(), "Close selected issues");
        assert_eq!(BulkAction::Reopen.display_name(), "Reopen selected issues");
        assert_eq!(BulkAction::Delete.display_name(), "Delete selected issues");
    }

    #[test]
    fn test_bulk_action_destructive() {
        assert!(BulkAction::Delete.is_destructive());
        assert!(BulkAction::Close.is_destructive());
        assert!(!BulkAction::Reopen.is_destructive());
        assert!(!BulkAction::SetPriority.is_destructive());
    }

    #[test]
    fn test_bulk_action_requires_input() {
        assert!(BulkAction::SetPriority.requires_input());
        assert!(BulkAction::AddLabels.requires_input());
        assert!(BulkAction::SetAssignee.requires_input());
        assert!(!BulkAction::Close.requires_input());
        assert!(!BulkAction::Reopen.requires_input());
    }

    #[test]
    fn test_bulk_action_menu_state_creation() {
        let state = BulkActionMenuState::new(5);
        assert_eq!(state.selected_count(), 5);
        assert!(state.highlighted_action().is_some());
        assert_eq!(state.confirmed_action(), None);
    }

    #[test]
    fn test_set_selected_count() {
        let mut state = BulkActionMenuState::new(5);
        assert_eq!(state.selected_count(), 5);

        state.set_selected_count(10);
        assert_eq!(state.selected_count(), 10);
    }

    #[test]
    fn test_navigation() {
        let mut state = BulkActionMenuState::new(5);
        let initial = state.highlighted_action().unwrap();

        state.select_next();
        let next = state.highlighted_action().unwrap();
        assert_ne!(initial, next);

        state.select_previous();
        let prev = state.highlighted_action().unwrap();
        assert_eq!(initial, prev);
    }

    #[test]
    fn test_navigation_wraparound() {
        let mut state = BulkActionMenuState::new(5);

        // Go to first item
        state.list_state.select(Some(0));

        // Go previous should wrap to last
        state.select_previous();
        let last_idx = state.actions.len() - 1;
        assert_eq!(state.list_state.selected(), Some(last_idx));

        // Go next should wrap to first
        state.select_next();
        assert_eq!(state.list_state.selected(), Some(0));
    }

    #[test]
    fn test_confirm_selection() {
        let mut state = BulkActionMenuState::new(5);

        // Select the first action
        state.list_state.select(Some(0));
        let action = state.highlighted_action().unwrap();

        // Confirm it
        let confirmed = state.confirm_selection();
        assert_eq!(confirmed, Some(action));
        assert_eq!(state.confirmed_action(), Some(action));
    }

    #[test]
    fn test_clear_confirmed() {
        let mut state = BulkActionMenuState::new(5);

        state.confirm_selection();
        assert!(state.confirmed_action().is_some());

        state.clear_confirmed();
        assert!(state.confirmed_action().is_none());
    }

    #[test]
    fn test_reset() {
        let mut state = BulkActionMenuState::new(5);

        // Navigate and confirm
        state.select_next();
        state.select_next();
        state.confirm_selection();

        // Reset should go back to first item and clear confirmation
        state.reset();
        assert_eq!(state.list_state.selected(), Some(0));
        assert_eq!(state.confirmed_action(), None);
    }

    #[test]
    fn test_custom_actions() {
        let custom_actions = vec![
            BulkAction::Close,
            BulkAction::Reopen,
            BulkAction::Cancel,
        ];

        let state = BulkActionMenuState::with_actions(custom_actions.clone(), 3);
        assert_eq!(state.actions(), &custom_actions);
        assert_eq!(state.selected_count(), 3);
    }

    #[test]
    fn test_empty_actions() {
        let state = BulkActionMenuState::with_actions(Vec::new(), 0);
        assert_eq!(state.actions().len(), 0);
        assert_eq!(state.highlighted_action(), None);
    }

    #[test]
    fn test_all_actions_count() {
        let all_actions = BulkAction::all();
        assert!(all_actions.len() > 0);
        assert!(all_actions.contains(&BulkAction::Close));
        assert!(all_actions.contains(&BulkAction::Cancel));
    }

    #[test]
    fn test_action_colors() {
        assert_eq!(BulkAction::Close.color(), Color::Green);
        assert_eq!(BulkAction::Delete.color(), Color::Red);
        assert_eq!(BulkAction::SetPriority.color(), Color::Magenta);
    }

    #[test]
    fn test_action_icons() {
        assert_eq!(BulkAction::Close.icon(), "✓");
        assert_eq!(BulkAction::Delete.icon(), "✗");
        assert_eq!(BulkAction::Cancel.icon(), "←");
    }
}
