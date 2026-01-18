use crossterm::event::{KeyEvent, MouseEvent};
use crate::models::AppState;

/// Trait for handling events in view modules
///
/// Each view state type implements this trait to handle keyboard and mouse events
/// independently, enabling better encapsulation and modular development.
///
/// Note: These are associated functions (not methods) because the view state
/// is stored inside AppState. This avoids borrow checker issues.
pub trait ViewEventHandler {
    /// Handle keyboard event, returns true if the event was consumed
    ///
    /// If this returns false, the event should propagate to global handlers.
    fn handle_key_event(app: &mut AppState, key: KeyEvent) -> bool;

    /// Handle mouse event (optional), returns true if the event was consumed
    ///
    /// Default implementation ignores mouse events.
    /// If this returns false, the event should propagate to global handlers.
    fn handle_mouse_event(app: &mut AppState, mouse: MouseEvent) -> bool {
        let _ = (mouse, app);
        false
    }

    /// Returns the name of this view for debugging/logging purposes
    fn view_name() -> &'static str;
}
