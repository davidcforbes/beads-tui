# Accessibility Features

beads-tui is designed to be accessible to users with various visual impairments and accessibility needs.

## Color Themes

The application supports multiple color themes optimized for different accessibility needs:

### Available Themes

1. **Dark** (default)
   - Standard dark theme with good contrast
   - Suitable for general use

2. **High Contrast**
   - Black and white theme with maximum contrast
   - WCAG AAA compliant contrast ratios (4.5:1 minimum for text)
   - Recommended for users with low vision

3. **Deuteranopia**
   - Optimized for red-green color blindness (deuteranopia)
   - Uses blue-yellow color distinctions instead of red-green
   - Affects ~5% of males, ~0.4% of females

4. **Protanopia**
   - Optimized for red-green color blindness (protanopia)
   - Similar to deuteranopia palette
   - Affects ~1% of males

5. **Tritanopia**
   - Optimized for blue-yellow color blindness (tritanopia)
   - Uses red-green color distinctions instead of blue-yellow
   - Rare condition affecting ~0.01% of population

### Changing Themes

To change your theme, edit the configuration file:

**Location:** `~/.config/beads-tui/config.yaml` (Linux/macOS) or `%APPDATA%\beads-tui\config.yaml` (Windows)

```yaml
theme:
  name: high-contrast  # Options: dark, high-contrast, deuteranopia, protanopia, tritanopia
```

The theme will be applied the next time you start beads-tui.

## Redundant Encoding

The application follows accessibility best practices by not relying on color alone to convey information:

### Status Indicators

All status information uses multiple visual cues:

- **Icons** - Each notification type has a unique icon:
  - ✖ Error
  - ✓ Success
  - ℹ Info
  - ⚠ Warning
  - ? Confirmation

- **Text Labels** - Clear text descriptions accompany all visual indicators

- **Color** - Used as an additional visual cue, not the sole indicator

### Examples

**Toast Notifications:**
- Success: Green background + ✓ icon + "Success" label
- Error: Red background + ✖ icon + "Error" label
- Info: Blue background + ℹ icon + "Info" label
- Warning: Yellow background + ⚠ icon + "Warning" label

**Dialog Boxes:**
- Each dialog type has a unique symbol and color
- Button selection is indicated by highlighting (not color alone)

## Keyboard Navigation

The application is fully navigable using only a keyboard:

### Global Navigation

- `Tab` / `Shift+Tab` - Navigate between tabs
- `Arrow Keys` - Navigate within lists and tables
- `Enter` - Select/activate items
- `Esc` - Cancel dialogs, close overlays
- `/` - Open search
- `?` - Show keyboard shortcuts help

### Issue Management

- `n` - Create new issue
- `e` - Edit selected issue
- `d` - Delete selected issue
- `c` - Close selected issue
- `Space` - Mark issue for batch operations

### View-Specific Shortcuts

Each view has context-specific keyboard shortcuts. Press `?` in any view to see available shortcuts.

## Screen Reader Support

While beads-tui is a terminal application and direct screen reader integration is limited by terminal capabilities, we follow these practices:

1. **Semantic Structure** - UI elements are organized logically
2. **Clear Labels** - All interactive elements have descriptive labels
3. **Status Updates** - Important state changes are announced via notifications
4. **Consistent Navigation** - Predictable keyboard navigation patterns

## Terminal Requirements

For the best accessibility experience, use a terminal that supports:

- **Unicode Characters** - For icons and symbols
- **256 Colors** - For proper theme rendering
- **Alt/Meta Keys** - For extended keyboard shortcuts

### Recommended Terminals

- **Windows:** Windows Terminal, ConEmu
- **macOS:** iTerm2, Terminal.app
- **Linux:** GNOME Terminal, Konsole, Alacritty

## Accessibility Limitations

As a terminal-based application, beads-tui has some inherent limitations:

1. **Screen Reader Integration** - Limited by terminal capabilities
2. **Font Scaling** - Controlled by terminal settings, not the application
3. **Custom Key Remapping** - Limited (future enhancement planned)

## Future Enhancements

Planned accessibility improvements:

- [ ] Configurable keyboard shortcuts
- [ ] Audio feedback option
- [ ] Customizable font sizes (via terminal configuration guide)
- [ ] Enhanced screen reader annotations
- [ ] Motion-reduced option for animations

## Feedback

If you encounter accessibility issues or have suggestions for improvements, please:

1. Open an issue on GitHub: https://github.com/davidcforbes/beads-tui/issues
2. Include details about:
   - Your accessibility needs
   - Specific issues encountered
   - Terminal and OS you're using
   - Suggested improvements

## Standards Compliance

beads-tui strives to follow:

- **WCAG 2.1 Level AA** - Web Content Accessibility Guidelines
  - Minimum contrast ratio of 4.5:1 for normal text
  - 3:1 for large text
  - Color is not the only means of conveying information
  - Full keyboard accessibility

## Resources

For more information about terminal accessibility:

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [Color Blindness Simulator](https://www.color-blindness.com/coblis-color-blindness-simulator/)
- [Terminal Accessibility Best Practices](https://wiki.archlinux.org/title/Accessibility)
