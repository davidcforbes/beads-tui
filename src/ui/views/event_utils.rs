use crossterm::event::{KeyCode, KeyEvent};
use crate::models::AppState;
use crate::config::Action;
use crate::ui::widgets::{FilterBarState, FilterDropdownType};

/// Handle notification dismissal with ESC key
///
/// Returns true if a notification was dismissed, false otherwise.
/// This should be called first in event handlers to give notifications priority.
pub fn handle_notification_dismissal(action: Option<Action>, app: &mut AppState) -> bool {
    if !app.notifications.is_empty() && matches!(action, Some(Action::DismissNotification)) {
        app.clear_notification();
        true
    } else {
        false
    }
}

/// Handle filter dropdown hotkeys (1-7) for filter bars
///
/// Returns true if a filter hotkey was handled, false otherwise.
/// Keys 1-6 toggle respective filter dropdowns, key 7 resets all filters.
pub fn handle_filter_hotkeys(
    key: KeyEvent,
    filter_bar_state: &mut Option<FilterBarState>,
    app: &mut AppState,
) -> bool {
    if filter_bar_state.is_none() {
        return false;
    }

    match key.code {
        KeyCode::Char('1') => {
            if let Some(ref mut state) = filter_bar_state {
                state.toggle_dropdown(FilterDropdownType::Status);
                app.mark_dirty();
            }
            true
        }
        KeyCode::Char('2') => {
            if let Some(ref mut state) = filter_bar_state {
                state.toggle_dropdown(FilterDropdownType::Type);
                app.mark_dirty();
            }
            true
        }
        KeyCode::Char('3') => {
            if let Some(ref mut state) = filter_bar_state {
                state.toggle_dropdown(FilterDropdownType::Priority);
                app.mark_dirty();
            }
            true
        }
        KeyCode::Char('4') => {
            if let Some(ref mut state) = filter_bar_state {
                state.toggle_dropdown(FilterDropdownType::Labels);
                app.mark_dirty();
            }
            true
        }
        KeyCode::Char('5') => {
            if let Some(ref mut state) = filter_bar_state {
                state.toggle_dropdown(FilterDropdownType::Created);
                app.mark_dirty();
            }
            true
        }
        KeyCode::Char('6') => {
            if let Some(ref mut state) = filter_bar_state {
                state.toggle_dropdown(FilterDropdownType::Updated);
                app.mark_dirty();
            }
            true
        }
        KeyCode::Char('7') => {
            if let Some(ref mut state) = filter_bar_state {
                state.close_dropdown();
                state.status_dropdown.clear_selection();
                state.priority_dropdown.clear_selection();
                state.type_dropdown.clear_selection();
                state.labels_dropdown.clear_selection();
                state.created_dropdown.clear_selection();
                state.updated_dropdown.clear_selection();
                app.mark_dirty();
            }
            true
        }
        _ => false,
    }
}

/// Handle filter dropdown navigation when a dropdown is open
///
/// Returns true if an event was handled, false otherwise.
/// Handles up/down navigation, selection toggling, and confirm/cancel actions.
pub fn handle_filter_dropdown_navigation(
    action: Option<Action>,
    filter_bar_state: &mut Option<FilterBarState>,
    app: &mut AppState,
    on_apply: impl FnOnce(&mut AppState),
) -> bool {
    let Some(ref mut state) = filter_bar_state else {
        return false;
    };

    if state.active_dropdown.is_none() {
        return false;
    }

    match action {
        Some(Action::MoveUp) => {
            if let Some(mut dropdown) = state.active_dropdown_mut() {
                dropdown.previous();
                app.mark_dirty();
            }
            true
        }
        Some(Action::MoveDown) => {
            if let Some(mut dropdown) = state.active_dropdown_mut() {
                dropdown.next();
                app.mark_dirty();
            }
            true
        }
        Some(Action::ToggleSelection) => {
            if let Some(mut dropdown) = state.active_dropdown_mut() {
                dropdown.toggle_selected();
                app.mark_dirty();
            }
            true
        }
        Some(Action::ConfirmDialog) => {
            state.close_dropdown();
            on_apply(app);
            app.mark_dirty();
            true
        }
        Some(Action::CancelDialog) => {
            state.close_dropdown();
            app.mark_dirty();
            true
        }
        _ => false,
    }
}

/// Handle dialog button navigation
///
/// Returns true if an event was handled, false otherwise.
pub fn handle_dialog_navigation(
    action: Option<Action>,
    dialog_state: &mut Option<crate::ui::widgets::DialogState>,
    button_count: usize,
) -> bool {
    let Some(ref mut state) = dialog_state else {
        return false;
    };

    match action {
        Some(Action::MoveLeft) | Some(Action::PrevDialogButton) => {
            state.select_previous(button_count);
            true
        }
        Some(Action::MoveRight) | Some(Action::NextDialogButton) | Some(Action::NextTab) => {
            state.select_next(button_count);
            true
        }
        _ => false,
    }
}
