//! Command palette with fuzzy search for beads-tui

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

/// Application context for context-sensitive commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppContext {
    /// Global commands available everywhere
    Global,
    /// Commands specific to the Issues view
    Issues,
    /// Commands specific to the Issue Detail view
    IssueDetail,
    /// Commands specific to the Dependencies view
    Dependencies,
    /// Commands specific to the Dependency Graph view
    DependencyGraph,
    /// Commands specific to the Labels view
    Labels,
    /// Commands specific to the Database view
    Database,
    /// Commands specific to the Help view
    Help,
    /// Commands specific to the Settings view
    Settings,
}

/// Available commands in the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: CommandCategory,
    pub keys: Vec<String>,
    pub contexts: Vec<AppContext>,
}

/// Command categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    Navigation,
    Issue,
    Dependency,
    Label,
    Database,
    View,
    System,
}

impl std::fmt::Display for CommandCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandCategory::Navigation => write!(f, "Navigation"),
            CommandCategory::Issue => write!(f, "Issue"),
            CommandCategory::Dependency => write!(f, "Dependency"),
            CommandCategory::Label => write!(f, "Label"),
            CommandCategory::Database => write!(f, "Database"),
            CommandCategory::View => write!(f, "View"),
            CommandCategory::System => write!(f, "System"),
        }
    }
}

/// Command palette state and logic
pub struct CommandPalette {
    commands: Vec<Command>,
    matcher: SkimMatcherV2,
    search_query: String,
    selected_index: usize,
    history: Vec<String>,
    max_history: usize,
    current_context: AppContext,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    /// Create a new command palette with default commands
    pub fn new() -> Self {
        let mut palette = Self {
            commands: Vec::new(),
            matcher: SkimMatcherV2::default(),
            search_query: String::new(),
            selected_index: 0,
            history: Vec::new(),
            max_history: 100,
            current_context: AppContext::Global,
        };

        palette.register_default_commands();
        palette
    }

    /// Set the current application context
    pub fn set_context(&mut self, context: AppContext) {
        self.current_context = context;
        self.selected_index = 0; // Reset selection when context changes
    }

    /// Get the current context
    pub fn context(&self) -> AppContext {
        self.current_context
    }

    /// Register default commands
    fn register_default_commands(&mut self) {
        // Navigation commands (available globally)
        self.add_command(Command {
            id: "nav.issues".to_string(),
            name: "Go to Issues".to_string(),
            description: "Navigate to issues view".to_string(),
            category: CommandCategory::Navigation,
            keys: vec!["1".to_string()],
            contexts: vec![AppContext::Global],
        });

        self.add_command(Command {
            id: "nav.dependencies".to_string(),
            name: "Go to Dependencies".to_string(),
            description: "Navigate to dependencies view".to_string(),
            category: CommandCategory::Navigation,
            keys: vec!["2".to_string()],
            contexts: vec![AppContext::Global],
        });

        self.add_command(Command {
            id: "nav.labels".to_string(),
            name: "Go to Labels".to_string(),
            description: "Navigate to labels view".to_string(),
            category: CommandCategory::Navigation,
            keys: vec!["3".to_string()],
            contexts: vec![AppContext::Global],
        });

        self.add_command(Command {
            id: "nav.database".to_string(),
            name: "Go to Database".to_string(),
            description: "Navigate to database view".to_string(),
            category: CommandCategory::Navigation,
            keys: vec!["4".to_string()],
            contexts: vec![AppContext::Global],
        });

        self.add_command(Command {
            id: "nav.help".to_string(),
            name: "Show Help".to_string(),
            description: "Show help and keyboard shortcuts".to_string(),
            category: CommandCategory::Navigation,
            keys: vec!["?".to_string()],
            contexts: vec![AppContext::Global],
        });

        self.add_command(Command {
            id: "nav.back".to_string(),
            name: "Go Back".to_string(),
            description: "Navigate to previous view".to_string(),
            category: CommandCategory::Navigation,
            keys: vec!["Backspace".to_string()],
            contexts: vec![AppContext::Global],
        });

        self.add_command(Command {
            id: "nav.forward".to_string(),
            name: "Go Forward".to_string(),
            description: "Navigate to next view in history".to_string(),
            category: CommandCategory::Navigation,
            keys: vec!["Shift+Backspace".to_string()],
            contexts: vec![AppContext::Global],
        });

        // Issue commands (context-sensitive)
        self.add_command(Command {
            id: "issue.new".to_string(),
            name: "New Issue".to_string(),
            description: "Create a new issue".to_string(),
            category: CommandCategory::Issue,
            keys: vec!["n".to_string()],
            contexts: vec![AppContext::Issues, AppContext::Global],
        });

        self.add_command(Command {
            id: "issue.close".to_string(),
            name: "Close Issue".to_string(),
            description: "Close the selected issue".to_string(),
            category: CommandCategory::Issue,
            keys: vec!["c".to_string()],
            contexts: vec![AppContext::Issues, AppContext::IssueDetail],
        });

        self.add_command(Command {
            id: "issue.edit".to_string(),
            name: "Edit Issue".to_string(),
            description: "Edit the selected issue".to_string(),
            category: CommandCategory::Issue,
            keys: vec!["e".to_string()],
            contexts: vec![AppContext::Issues, AppContext::IssueDetail],
        });

        // View commands (global)
        self.add_command(Command {
            id: "view.fullscreen".to_string(),
            name: "Toggle Fullscreen".to_string(),
            description: "Toggle fullscreen mode for focused pane".to_string(),
            category: CommandCategory::View,
            keys: vec!["f".to_string()],
            contexts: vec![AppContext::Global],
        });

        self.add_command(Command {
            id: "view.split_horizontal".to_string(),
            name: "Split Horizontal".to_string(),
            description: "Split current pane horizontally".to_string(),
            category: CommandCategory::View,
            keys: vec!["Ctrl+|".to_string()],
            contexts: vec![AppContext::Global],
        });

        self.add_command(Command {
            id: "view.split_vertical".to_string(),
            name: "Split Vertical".to_string(),
            description: "Split current pane vertically".to_string(),
            category: CommandCategory::View,
            keys: vec!["Ctrl+-".to_string()],
            contexts: vec![AppContext::Global],
        });

        // System commands (global)
        self.add_command(Command {
            id: "system.quit".to_string(),
            name: "Quit".to_string(),
            description: "Exit the application".to_string(),
            category: CommandCategory::System,
            keys: vec!["q".to_string()],
            contexts: vec![AppContext::Global],
        });

        self.add_command(Command {
            id: "system.reload".to_string(),
            name: "Reload".to_string(),
            description: "Reload data from beads database".to_string(),
            category: CommandCategory::System,
            keys: vec!["r".to_string()],
            contexts: vec![AppContext::Global],
        });
    }

    /// Add a new command
    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command);
    }

    /// Set the search query
    pub fn set_query(&mut self, query: String) {
        self.search_query = query;
        self.selected_index = 0;
    }

    /// Get the search query
    pub fn query(&self) -> &str {
        &self.search_query
    }

    /// Search commands with fuzzy matching and context filtering
    pub fn search(&self) -> Vec<(&Command, i64)> {
        let context_filtered: Vec<&Command> = self
            .commands
            .iter()
            .filter(|cmd| {
                cmd.contexts.contains(&AppContext::Global)
                    || cmd.contexts.contains(&self.current_context)
            })
            .collect();

        if self.search_query.is_empty() {
            context_filtered.iter().map(|&cmd| (cmd, 0i64)).collect()
        } else {
            let mut matches: Vec<(&Command, i64)> = context_filtered
                .iter()
                .filter_map(|&cmd| {
                    let name_score = self.matcher.fuzzy_match(&cmd.name, &self.search_query);
                    let desc_score = self.matcher.fuzzy_match(&cmd.description, &self.search_query);
                    let score = name_score.unwrap_or(0).max(desc_score.unwrap_or(0));

                    if score > 0 {
                        Some((cmd, score))
                    } else {
                        None
                    }
                })
                .collect();

            matches.sort_by(|a, b| b.1.cmp(&a.1));
            matches
        }
    }

    /// Get count of commands available in current context
    pub fn available_command_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|cmd| {
                cmd.contexts.contains(&AppContext::Global)
                    || cmd.contexts.contains(&self.current_context)
            })
            .count()
    }

    /// Get the currently selected command
    pub fn selected(&self) -> Option<&Command> {
        let results = self.search();
        results.get(self.selected_index).map(|(cmd, _)| *cmd)
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        let max_index = self.search().len().saturating_sub(1);
        if self.selected_index < max_index {
            self.selected_index += 1;
        }
    }

    /// Add command to history
    pub fn add_to_history(&mut self, command_id: String) {
        self.history.push(command_id);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Get command history
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Get command by ID
    pub fn get_command(&self, id: &str) -> Option<&Command> {
        self.commands.iter().find(|cmd| cmd.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_palette_creation() {
        let palette = CommandPalette::new();
        assert!(!palette.commands.is_empty());
    }

    #[test]
    fn test_fuzzy_search() {
        let palette = CommandPalette::new();
        let results = palette.search();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_command_search() {
        let mut palette = CommandPalette::new();
        palette.set_query("issue".to_string());

        let results = palette.search();
        assert!(!results.is_empty());

        // All results should be related to issues
        for (cmd, _) in results {
            let matches = cmd.name.to_lowercase().contains("issue")
                || cmd.description.to_lowercase().contains("issue");
            assert!(matches);
        }
    }

    #[test]
    fn test_selection() {
        let mut palette = CommandPalette::new();
        palette.select_next();
        assert_eq!(palette.selected_index, 1);

        palette.select_previous();
        assert_eq!(palette.selected_index, 0);
    }

    #[test]
    fn test_context_filtering() {
        let mut palette = CommandPalette::new();

        // Global context should show all global commands
        palette.set_context(AppContext::Global);
        let global_results = palette.search();
        let global_count = global_results.len();
        assert!(global_count > 0);

        // Issues context should show issue-specific + global commands
        palette.set_context(AppContext::Issues);
        let issues_results = palette.search();
        
        // Should have issue-specific commands
        assert!(issues_results
            .iter()
            .any(|(cmd, _)| cmd.id == "issue.new"));
        assert!(issues_results
            .iter()
            .any(|(cmd, _)| cmd.id == "issue.close"));

        // Should still have global navigation commands
        assert!(issues_results
            .iter()
            .any(|(cmd, _)| cmd.id == "nav.issues"));
    }

    #[test]
    fn test_context_switching() {
        let mut palette = CommandPalette::new();

        palette.set_context(AppContext::Issues);
        assert_eq!(palette.context(), AppContext::Issues);
        assert_eq!(palette.selected_index, 0);

        palette.selected_index = 5;
        palette.set_context(AppContext::Dependencies);
        assert_eq!(palette.context(), AppContext::Dependencies);
        assert_eq!(palette.selected_index, 0); // Reset on context change
    }

    #[test]
    fn test_available_command_count() {
        let palette = CommandPalette::new();
        
        // Should have commands available in all contexts
        assert!(palette.available_command_count() > 0);
    }
}
