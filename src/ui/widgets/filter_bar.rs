use std::collections::HashSet;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Widget},
};

use crate::beads::models::{IssueStatus, IssueType, Priority};
use crate::ui::themes::Theme;

/// Multi-select dropdown state for filter options
#[derive(Debug, Clone)]
pub struct MultiSelectDropdownState<T> {
    items: Vec<T>,
    selected: HashSet<usize>,
    list_state: ListState,
    is_open: bool,
}

impl<T: Clone> MultiSelectDropdownState<T> {
    pub fn new(items: Vec<T>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            items,
            selected: HashSet::new(),
            list_state,
            is_open: false,
        }
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn toggle_open(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn toggle_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            // Index 0 is "All"
            if i == 0 {
                // Selecting "All" deselects everything else
                self.selected.clear();
            } else {
                // Selecting a specific item deselects "All" and toggles this item
                if self.selected.contains(&i) {
                    self.selected.remove(&i);
                } else {
                    self.selected.insert(i);
                }
            }
        }
    }

    pub fn is_selected(&self, index: usize) -> bool {
        if self.selected.is_empty() {
            // Empty selection means "All" is selected
            index == 0
        } else {
            self.selected.contains(&index)
        }
    }

    pub fn selected_items(&self) -> Vec<T> {
        if self.selected.is_empty() {
            // "All" selected - return empty vec to indicate no specific filter
            vec![]
        } else {
            self.selected
                .iter()
                // Subtract 1 from idx because idx includes "All" at position 0,
                // but self.items doesn't include "All"
                .filter_map(|&idx| {
                    if idx > 0 {
                        self.items.get(idx - 1).cloned()
                    } else {
                        None
                    }
                })
                .collect()
        }
    }

    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }
}

/// Type of filter dropdown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterDropdownType {
    Status,
    Priority,
    Type,
    Labels,
    Assignee,
    Created,
    Updated,
    Closed,
}

impl FilterDropdownType {
    pub fn title(&self) -> &str {
        match self {
            FilterDropdownType::Status => "Status Filter",
            FilterDropdownType::Priority => "Priority Filter",
            FilterDropdownType::Type => "Type Filter",
            FilterDropdownType::Labels => "Labels Filter",
            FilterDropdownType::Assignee => "Assignee Filter",
            FilterDropdownType::Created => "Created Date Filter",
            FilterDropdownType::Updated => "Updated Date Filter",
            FilterDropdownType::Closed => "Closed Date Filter",
        }
    }
}

/// State for the filter bar
#[derive(Debug)]
pub struct FilterBarState {
    pub status_dropdown: MultiSelectDropdownState<IssueStatus>,
    pub priority_dropdown: MultiSelectDropdownState<Priority>,
    pub type_dropdown: MultiSelectDropdownState<IssueType>,
    pub labels_dropdown: MultiSelectDropdownState<String>,
    pub assignee_dropdown: MultiSelectDropdownState<String>,
    pub created_dropdown: MultiSelectDropdownState<String>,
    pub updated_dropdown: MultiSelectDropdownState<String>,
    pub closed_dropdown: MultiSelectDropdownState<String>,
    pub active_dropdown: Option<FilterDropdownType>,
}

impl FilterBarState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        statuses: Vec<IssueStatus>,
        priorities: Vec<Priority>,
        types: Vec<IssueType>,
        labels: Vec<String>,
        assignees: Vec<String>,
        created_dates: Vec<String>,
        updated_dates: Vec<String>,
        closed_dates: Vec<String>,
    ) -> Self {
        Self {
            status_dropdown: MultiSelectDropdownState::new(statuses),
            priority_dropdown: MultiSelectDropdownState::new(priorities),
            type_dropdown: MultiSelectDropdownState::new(types),
            labels_dropdown: MultiSelectDropdownState::new(labels),
            assignee_dropdown: MultiSelectDropdownState::new(assignees),
            created_dropdown: MultiSelectDropdownState::new(created_dates),
            updated_dropdown: MultiSelectDropdownState::new(updated_dates),
            closed_dropdown: MultiSelectDropdownState::new(closed_dates),
            active_dropdown: None,
        }
    }

    /// Check if an issue matches all the selected filters
    /// Returns true if the issue passes all filters
    pub fn matches_issue(&self, issue: &crate::beads::models::Issue) -> bool {
        // Check status filter
        let selected_statuses = self.status_dropdown.selected_items();
        if !selected_statuses.is_empty() {
            // If specific statuses are selected, issue must match one of them
            if !selected_statuses.contains(&issue.status) {
                return false;
            }
        }

        // Check priority filter
        let selected_priorities = self.priority_dropdown.selected_items();
        if !selected_priorities.is_empty() {
            // If specific priorities are selected, issue must match one of them
            if !selected_priorities.contains(&issue.priority) {
                return false;
            }
        }

        // Check type filter
        let selected_types = self.type_dropdown.selected_items();
        if !selected_types.is_empty() {
            // If specific types are selected, issue must match one of them
            if !selected_types.contains(&issue.issue_type) {
                return false;
            }
        }

        // Check labels filter (OR logic - issue must have at least one of the selected labels)
        let selected_labels = self.labels_dropdown.selected_items();
        if !selected_labels.is_empty() {
            // If specific labels are selected, issue must have at least one of them
            let has_matching_label = selected_labels.iter().any(|filter_label| {
                issue.labels.iter().any(|issue_label| {
                    issue_label.to_lowercase().contains(&filter_label.to_lowercase())
                })
            });
            if !has_matching_label {
                return false;
            }
        }

        // Check assignee filter
        let selected_assignees = self.assignee_dropdown.selected_items();
        if !selected_assignees.is_empty() {
            let issue_assignee = issue.assignee.as_deref().unwrap_or("-");
            if !selected_assignees.iter().any(|a| a == issue_assignee) {
                return false;
            }
        }

        // Check created date filter
        let selected_created = self.created_dropdown.selected_items();
        if !selected_created.is_empty() {
            use chrono::Datelike;
            let created_str = format!("{:04}-{:02}-{:02}",
                issue.created.year(),
                issue.created.month(),
                issue.created.day());
            if !selected_created.contains(&created_str) {
                return false;
            }
        }

        // Check updated date filter
        let selected_updated = self.updated_dropdown.selected_items();
        if !selected_updated.is_empty() {
            use chrono::Datelike;
            let updated_str = format!("{:04}-{:02}-{:02}",
                issue.updated.year(),
                issue.updated.month(),
                issue.updated.day());
            if !selected_updated.contains(&updated_str) {
                return false;
            }
        }

        // Check closed date filter
        let selected_closed = self.closed_dropdown.selected_items();
        if !selected_closed.is_empty() {
            if let Some(ref closed_date) = issue.closed {
                use chrono::Datelike;
                let closed_str = format!("{:04}-{:02}-{:02}",
                    closed_date.year(),
                    closed_date.month(),
                    closed_date.day());
                if !selected_closed.contains(&closed_str) {
                    return false;
                }
            } else {
                // If closed date is selected but issue is not closed, filter it out
                return false;
            }
        }

        // Issue passed all filters
        true
    }

    pub fn toggle_dropdown(&mut self, dropdown_type: FilterDropdownType) {
        // Close any other open dropdown
        if self.active_dropdown.is_some() && self.active_dropdown != Some(dropdown_type) {
            self.close_dropdown();
        }

        // Toggle the requested dropdown
        match dropdown_type {
            FilterDropdownType::Status => {
                self.status_dropdown.toggle_open();
                if self.status_dropdown.is_open() {
                    self.active_dropdown = Some(FilterDropdownType::Status);
                } else {
                    self.active_dropdown = None;
                }
            }
            FilterDropdownType::Priority => {
                self.priority_dropdown.toggle_open();
                if self.priority_dropdown.is_open() {
                    self.active_dropdown = Some(FilterDropdownType::Priority);
                } else {
                    self.active_dropdown = None;
                }
            }
            FilterDropdownType::Type => {
                self.type_dropdown.toggle_open();
                if self.type_dropdown.is_open() {
                    self.active_dropdown = Some(FilterDropdownType::Type);
                } else {
                    self.active_dropdown = None;
                }
            }
            FilterDropdownType::Labels => {
                self.labels_dropdown.toggle_open();
                if self.labels_dropdown.is_open() {
                    self.active_dropdown = Some(FilterDropdownType::Labels);
                } else {
                    self.active_dropdown = None;
                }
            }
            FilterDropdownType::Assignee => {
                self.assignee_dropdown.toggle_open();
                if self.assignee_dropdown.is_open() {
                    self.active_dropdown = Some(FilterDropdownType::Assignee);
                } else {
                    self.active_dropdown = None;
                }
            }
            FilterDropdownType::Created => {
                self.created_dropdown.toggle_open();
                if self.created_dropdown.is_open() {
                    self.active_dropdown = Some(FilterDropdownType::Created);
                } else {
                    self.active_dropdown = None;
                }
            }
            FilterDropdownType::Updated => {
                self.updated_dropdown.toggle_open();
                if self.updated_dropdown.is_open() {
                    self.active_dropdown = Some(FilterDropdownType::Updated);
                } else {
                    self.active_dropdown = None;
                }
            }
            FilterDropdownType::Closed => {
                self.closed_dropdown.toggle_open();
                if self.closed_dropdown.is_open() {
                    self.active_dropdown = Some(FilterDropdownType::Closed);
                } else {
                    self.active_dropdown = None;
                }
            }
        }
    }

    pub fn close_dropdown(&mut self) {
        if let Some(dropdown_type) = self.active_dropdown {
            match dropdown_type {
                FilterDropdownType::Status => self.status_dropdown.close(),
                FilterDropdownType::Priority => self.priority_dropdown.close(),
                FilterDropdownType::Type => self.type_dropdown.close(),
                FilterDropdownType::Labels => self.labels_dropdown.close(),
                FilterDropdownType::Assignee => self.assignee_dropdown.close(),
                FilterDropdownType::Created => self.created_dropdown.close(),
                FilterDropdownType::Updated => self.updated_dropdown.close(),
                FilterDropdownType::Closed => self.closed_dropdown.close(),
            }
        }
        self.active_dropdown = None;
    }

    pub fn active_dropdown_mut(&mut self) -> Option<ActiveDropdownMut<'_>> {
        match self.active_dropdown {
            Some(FilterDropdownType::Status) => {
                Some(ActiveDropdownMut::Status(&mut self.status_dropdown))
            }
            Some(FilterDropdownType::Priority) => {
                Some(ActiveDropdownMut::Priority(&mut self.priority_dropdown))
            }
            Some(FilterDropdownType::Type) => {
                Some(ActiveDropdownMut::Type(&mut self.type_dropdown))
            }
            Some(FilterDropdownType::Labels) => {
                Some(ActiveDropdownMut::Labels(&mut self.labels_dropdown))
            }
            Some(FilterDropdownType::Assignee) => {
                Some(ActiveDropdownMut::Assignee(&mut self.assignee_dropdown))
            }
            Some(FilterDropdownType::Created) => {
                Some(ActiveDropdownMut::Created(&mut self.created_dropdown))
            }
            Some(FilterDropdownType::Updated) => {
                Some(ActiveDropdownMut::Updated(&mut self.updated_dropdown))
            }
            Some(FilterDropdownType::Closed) => {
                Some(ActiveDropdownMut::Closed(&mut self.closed_dropdown))
            }
            None => None,
        }
    }
}

/// Mutable reference to active dropdown
pub enum ActiveDropdownMut<'a> {
    Status(&'a mut MultiSelectDropdownState<IssueStatus>),
    Priority(&'a mut MultiSelectDropdownState<Priority>),
    Type(&'a mut MultiSelectDropdownState<IssueType>),
    Labels(&'a mut MultiSelectDropdownState<String>),
    Assignee(&'a mut MultiSelectDropdownState<String>),
    Created(&'a mut MultiSelectDropdownState<String>),
    Updated(&'a mut MultiSelectDropdownState<String>),
    Closed(&'a mut MultiSelectDropdownState<String>),
}

impl<'a> ActiveDropdownMut<'a> {
    pub fn next(&mut self) {
        match self {
            ActiveDropdownMut::Status(s) => s.next(),
            ActiveDropdownMut::Priority(p) => p.next(),
            ActiveDropdownMut::Type(t) => t.next(),
            ActiveDropdownMut::Labels(l) => l.next(),
            ActiveDropdownMut::Assignee(a) => a.next(),
            ActiveDropdownMut::Created(c) => c.next(),
            ActiveDropdownMut::Updated(u) => u.next(),
            ActiveDropdownMut::Closed(c) => c.next(),
        }
    }

    pub fn previous(&mut self) {
        match self {
            ActiveDropdownMut::Status(s) => s.previous(),
            ActiveDropdownMut::Priority(p) => p.previous(),
            ActiveDropdownMut::Type(t) => t.previous(),
            ActiveDropdownMut::Labels(l) => l.previous(),
            ActiveDropdownMut::Assignee(a) => a.previous(),
            ActiveDropdownMut::Created(c) => c.previous(),
            ActiveDropdownMut::Updated(u) => u.previous(),
            ActiveDropdownMut::Closed(c) => c.previous(),
        }
    }

    pub fn toggle_selected(&mut self) {
        match self {
            ActiveDropdownMut::Status(s) => s.toggle_selected(),
            ActiveDropdownMut::Priority(p) => p.toggle_selected(),
            ActiveDropdownMut::Type(t) => t.toggle_selected(),
            ActiveDropdownMut::Labels(l) => l.toggle_selected(),
            ActiveDropdownMut::Assignee(a) => a.toggle_selected(),
            ActiveDropdownMut::Created(c) => c.toggle_selected(),
            ActiveDropdownMut::Updated(u) => u.toggle_selected(),
            ActiveDropdownMut::Closed(c) => c.toggle_selected(),
        }
    }
}

/// Filter bar widget
pub struct FilterBar<'a> {
    #[allow(dead_code)]
    filtered_count: usize,
    #[allow(dead_code)]
    total_count: usize,
    theme: &'a Theme,
}

impl<'a> FilterBar<'a> {
    pub fn new(
        filtered_count: usize,
        total_count: usize,
        theme: &'a Theme,
    ) -> Self {
        Self {
            filtered_count,
            total_count,
            theme,
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, state: &FilterBarState) {
        let status_text = self.dropdown_text(&state.status_dropdown, "All.▼");
        let type_text = self.dropdown_text(&state.type_dropdown, "All.▼");
        let priority_text = self.dropdown_text(&state.priority_dropdown, "All.▼");
        let labels_text = self.dropdown_text(&state.labels_dropdown, "All.▼");
        let created_text = self.dropdown_text(&state.created_dropdown, "All.▼");
        let updated_text = self.dropdown_text(&state.updated_dropdown, "All.▼");

        // Create bordered block with FILTERS header
        let block = Block::default()
            .borders(Borders::ALL)
            .title("[FILTERS]");

        let inner = block.inner(area);
        block.render(area, buf);

        // Build filter line with hotkeys - showing all 6 filters
        let line = Line::from(vec![
            Span::raw(" "),
            self.filter_with_hotkey_prefix("S:Status", &status_text, state.active_dropdown == Some(FilterDropdownType::Status)),
            Span::raw(" │ "),
            self.filter_with_hotkey_prefix("T:Type", &type_text, state.active_dropdown == Some(FilterDropdownType::Type)),
            Span::raw(" │ "),
            self.filter_with_hotkey_prefix("P:Priority", &priority_text, state.active_dropdown == Some(FilterDropdownType::Priority)),
            Span::raw(" │ "),
            self.filter_with_hotkey_prefix("L:Labels", &labels_text, state.active_dropdown == Some(FilterDropdownType::Labels)),
            Span::raw(" │ "),
            self.filter_with_hotkey_prefix("C:Created", &created_text, state.active_dropdown == Some(FilterDropdownType::Created)),
            Span::raw(" │ "),
            self.filter_with_hotkey_prefix("U:Updated", &updated_text, state.active_dropdown == Some(FilterDropdownType::Updated)),
            Span::raw(" │ "),
            Span::styled("F11:Reset", Style::default().fg(Color::Yellow)),
            Span::raw("      "),
        ]);

        line.render(inner, buf);
    }

    fn dropdown_text<T>(&self, dropdown: &MultiSelectDropdownState<T>, default: &str) -> String {
        let count = dropdown.selected.len();
        if count == 0 {
            default.to_string()
        } else {
            format!("{} selected", count)
        }
    }

    fn filter_with_hotkey_prefix(&self, label: &str, value: &str, is_active: bool) -> Span<'_> {
        let text = format!("{} [{}]", label, value);
        let style = if is_active {
            Style::default()
                .fg(self.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };
        Span::styled(text, style)
    }
}

/// Filter dropdown widget
pub struct FilterDropdown<'a> {
    dropdown_type: FilterDropdownType,
    theme: &'a Theme,
}

impl<'a> FilterDropdown<'a> {
    pub fn new(dropdown_type: FilterDropdownType, theme: &'a Theme) -> Self {
        Self {
            dropdown_type,
            theme,
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, state: &mut FilterBarState) {
        // Calculate dropdown area
        let dropdown_area = self.calculate_dropdown_area(area);

        // Clear background
        Clear.render(dropdown_area, buf);

        // Get items and selections based on dropdown type (clone selections to avoid borrow conflicts)
        let (items, selections) = match self.dropdown_type {
            FilterDropdownType::Status => {
                let items: Vec<String> = std::iter::once("All".to_string())
                    .chain(state.status_dropdown.items().iter().map(|s| format!("{:?}", s)))
                    .collect();
                (items, state.status_dropdown.selected.clone())
            }
            FilterDropdownType::Priority => {
                let items: Vec<String> = std::iter::once("All".to_string())
                    .chain(state.priority_dropdown.items().iter().map(|p| format!("{:?}", p)))
                    .collect();
                (items, state.priority_dropdown.selected.clone())
            }
            FilterDropdownType::Type => {
                let items: Vec<String> = std::iter::once("All".to_string())
                    .chain(state.type_dropdown.items().iter().map(|t| format!("{:?}", t)))
                    .collect();
                (items, state.type_dropdown.selected.clone())
            }
            FilterDropdownType::Labels => {
                let items: Vec<String> = std::iter::once("All".to_string())
                    .chain(state.labels_dropdown.items().iter().cloned())
                    .collect();
                (items, state.labels_dropdown.selected.clone())
            }
            FilterDropdownType::Assignee => {
                let items: Vec<String> = std::iter::once("All".to_string())
                    .chain(state.assignee_dropdown.items().iter().cloned())
                    .collect();
                (items, state.assignee_dropdown.selected.clone())
            }
            FilterDropdownType::Created => {
                let items: Vec<String> = std::iter::once("All".to_string())
                    .chain(state.created_dropdown.items().iter().cloned())
                    .collect();
                (items, state.created_dropdown.selected.clone())
            }
            FilterDropdownType::Updated => {
                let items: Vec<String> = std::iter::once("All".to_string())
                    .chain(state.updated_dropdown.items().iter().cloned())
                    .collect();
                (items, state.updated_dropdown.selected.clone())
            }
            FilterDropdownType::Closed => {
                let items: Vec<String> = std::iter::once("All".to_string())
                    .chain(state.closed_dropdown.items().iter().cloned())
                    .collect();
                (items, state.closed_dropdown.selected.clone())
            }
        };

        // Get mutable list_state reference separately
        let list_state = match self.dropdown_type {
            FilterDropdownType::Status => state.status_dropdown.list_state_mut(),
            FilterDropdownType::Priority => state.priority_dropdown.list_state_mut(),
            FilterDropdownType::Type => state.type_dropdown.list_state_mut(),
            FilterDropdownType::Labels => state.labels_dropdown.list_state_mut(),
            FilterDropdownType::Assignee => state.assignee_dropdown.list_state_mut(),
            FilterDropdownType::Created => state.created_dropdown.list_state_mut(),
            FilterDropdownType::Updated => state.updated_dropdown.list_state_mut(),
            FilterDropdownType::Closed => state.closed_dropdown.list_state_mut(),
        };

        // Create list items with checkboxes
        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_selected = if selections.is_empty() {
                    idx == 0 // "All" is selected
                } else {
                    selections.contains(&idx)
                };
                let checkbox = if is_selected { "[x] " } else { "[ ] " };
                let text = format!("{}{}", checkbox, item);
                ListItem::new(text)
            })
            .collect();

        // Create block with border
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.dropdown_type.title())
            .style(Style::default().bg(self.theme.background));

        // Split area for list and help footer
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(block.inner(dropdown_area));

        // Render block
        block.render(dropdown_area, buf);

        // Render list
        let list = List::new(list_items)
            .highlight_style(
                Style::default()
                    .fg(self.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        ratatui::widgets::StatefulWidget::render(list, chunks[0], buf, list_state);

        // Render help footer
        let help_line = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": Apply  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Cancel"),
        ]);
        help_line.render(chunks[1], buf);
    }

    fn calculate_dropdown_area(&self, area: Rect) -> Rect {
        let width = 35u16;
        let height = 12u16;

        // Calculate x_offset to align dropdown under each filter label
        // Based on the layout in render(): " S:Status [...] │ T:Type [...] │ P:Priority [...] │ ..."
        let x_offset = match self.dropdown_type {
            FilterDropdownType::Status => 1,    // After initial " "
            FilterDropdownType::Type => 23,     // After "S:Status [...]" + " │ "
            FilterDropdownType::Priority => 43, // After Type
            FilterDropdownType::Labels => 66,   // After Priority
            FilterDropdownType::Created => 87,  // After Labels
            FilterDropdownType::Updated => 108, // After Created
            FilterDropdownType::Assignee => 129, // After Updated (not shown but for completeness)
            FilterDropdownType::Closed => 150,  // After Assignee
        };

        let x = (area.x + x_offset).min(area.width.saturating_sub(width));
        let y = area.y + 1;

        Rect {
            x,
            y,
            width: width.min(area.width.saturating_sub(x - area.x)),
            height: height.min(area.height.saturating_sub(y - area.y)),
        }
    }
}
