# Widget Development Guide

## Principles
- Keep render functions pure. Do not mutate external state in render.
- Separate widget configuration from widget state.
- Favor small, reusable widgets over large monoliths.
- Use consistent styling for focus, selection, and disabled states.

## File layout
- src/ui/widgets: widget implementations.
- src/ui/views: composition of widgets into screens.
- docs/widgets.md: widget catalog and usage examples.

## Recommended pattern
1. Define a state struct that owns selection and input state.
2. Define a widget struct that receives data and styling.
3. Implement StatefulWidget for the widget and render from state.
4. Provide builder methods for configuration.

## Example checklist
- Use ratatui primitives (Block, List, Paragraph, etc).
- Provide a focused style for keyboard navigation.
- Keep layout math in helper functions.
- Add unit tests for state transitions and rendering helpers.

## Testing
- Unit tests for state transitions and helpers.
- Snapshot tests for visual layout when appropriate.
- Integration tests when widgets are wired into flows.

## Tips
- Prefer ASCII symbols in UI to avoid rendering differences.
- Keep minimum widths in mind for 80x24 terminals.
- Keep labels short so lists do not wrap unexpectedly.
