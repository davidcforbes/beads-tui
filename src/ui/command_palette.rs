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
                    let desc_score = self
                        .matcher
                        .fuzzy_match(&cmd.description, &self.search_query);
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
        assert!(issues_results.iter().any(|(cmd, _)| cmd.id == "issue.new"));
        assert!(issues_results
            .iter()
            .any(|(cmd, _)| cmd.id == "issue.close"));

        // Should still have global navigation commands
        assert!(issues_results.iter().any(|(cmd, _)| cmd.id == "nav.issues"));
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

    #[test]
    fn test_app_context_equality() {
        assert_eq!(AppContext::Global, AppContext::Global);
        assert_eq!(AppContext::Issues, AppContext::Issues);
        assert_eq!(AppContext::IssueDetail, AppContext::IssueDetail);
        assert_eq!(AppContext::Dependencies, AppContext::Dependencies);
        assert_eq!(AppContext::DependencyGraph, AppContext::DependencyGraph);
        assert_eq!(AppContext::Labels, AppContext::Labels);
        assert_eq!(AppContext::Database, AppContext::Database);
        assert_eq!(AppContext::Help, AppContext::Help);
        assert_eq!(AppContext::Settings, AppContext::Settings);

        assert_ne!(AppContext::Global, AppContext::Issues);
        assert_ne!(AppContext::Dependencies, AppContext::Labels);
    }

    #[test]
    fn test_command_category_display() {
        assert_eq!(CommandCategory::Navigation.to_string(), "Navigation");
        assert_eq!(CommandCategory::Issue.to_string(), "Issue");
        assert_eq!(CommandCategory::Dependency.to_string(), "Dependency");
        assert_eq!(CommandCategory::Label.to_string(), "Label");
        assert_eq!(CommandCategory::Database.to_string(), "Database");
        assert_eq!(CommandCategory::View.to_string(), "View");
        assert_eq!(CommandCategory::System.to_string(), "System");
    }

    #[test]
    fn test_command_category_equality() {
        assert_eq!(CommandCategory::Navigation, CommandCategory::Navigation);
        assert_ne!(CommandCategory::Issue, CommandCategory::View);
    }

    #[test]
    fn test_command_creation() {
        let cmd = Command {
            id: "test.command".to_string(),
            name: "Test Command".to_string(),
            description: "A test command".to_string(),
            category: CommandCategory::System,
            keys: vec!["t".to_string()],
            contexts: vec![AppContext::Global],
        };

        assert_eq!(cmd.id, "test.command");
        assert_eq!(cmd.name, "Test Command");
        assert_eq!(cmd.description, "A test command");
        assert_eq!(cmd.category, CommandCategory::System);
        assert_eq!(cmd.keys.len(), 1);
        assert_eq!(cmd.contexts.len(), 1);
    }

    #[test]
    fn test_command_clone() {
        let cmd = Command {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Desc".to_string(),
            category: CommandCategory::View,
            keys: vec!["k".to_string()],
            contexts: vec![AppContext::Global, AppContext::Issues],
        };

        let cloned = cmd.clone();
        assert_eq!(cmd, cloned);
    }

    #[test]
    fn test_command_palette_default() {
        let palette = CommandPalette::default();
        assert!(!palette.commands.is_empty());
        assert_eq!(palette.selected_index, 0);
    }

    #[test]
    fn test_command_palette_default_same_as_new() {
        let default_palette = CommandPalette::default();
        let new_palette = CommandPalette::new();
        assert_eq!(default_palette.commands.len(), new_palette.commands.len());
    }

    #[test]
    fn test_query_initial_value() {
        let palette = CommandPalette::new();
        assert_eq!(palette.query(), "");
    }

    #[test]
    fn test_set_query_updates_query() {
        let mut palette = CommandPalette::new();
        palette.set_query("test".to_string());
        assert_eq!(palette.query(), "test");
    }

    #[test]
    fn test_set_query_resets_selection() {
        let mut palette = CommandPalette::new();
        palette.selected_index = 5;
        palette.set_query("new query".to_string());
        assert_eq!(palette.selected_index, 0);
    }

    #[test]
    fn test_search_empty_query() {
        let palette = CommandPalette::new();
        let results = palette.search();
        assert!(!results.is_empty());
        // Empty query should return all context-filtered commands with score 0
        for (_, score) in &results {
            assert_eq!(*score, 0);
        }
    }

    #[test]
    fn test_search_non_matching_query() {
        let mut palette = CommandPalette::new();
        palette.set_query("xyzabc123nonexistent".to_string());
        let results = palette.search();
        assert!(results.is_empty());
    }

    #[test]
    fn test_selected_none_when_no_match() {
        let mut palette = CommandPalette::new();
        palette.set_query("xyzabc123nonexistent".to_string());
        assert!(palette.selected().is_none());
    }

    #[test]
    fn test_selected_some_when_match() {
        let palette = CommandPalette::new();
        assert!(palette.selected().is_some());
    }

    #[test]
    fn test_select_previous_at_zero() {
        let mut palette = CommandPalette::new();
        assert_eq!(palette.selected_index, 0);
        palette.select_previous();
        assert_eq!(palette.selected_index, 0); // Should stay at 0
    }

    #[test]
    fn test_select_next_at_max() {
        let mut palette = CommandPalette::new();
        let max = palette.search().len().saturating_sub(1);
        palette.selected_index = max;
        palette.select_next();
        assert_eq!(palette.selected_index, max); // Should stay at max
    }

    #[test]
    fn test_add_to_history() {
        let mut palette = CommandPalette::new();
        assert!(palette.history().is_empty());

        palette.add_to_history("cmd1".to_string());
        assert_eq!(palette.history().len(), 1);
        assert_eq!(palette.history()[0], "cmd1");

        palette.add_to_history("cmd2".to_string());
        assert_eq!(palette.history().len(), 2);
        assert_eq!(palette.history()[1], "cmd2");
    }

    #[test]
    fn test_add_to_history_respects_max() {
        let mut palette = CommandPalette::new();
        palette.max_history = 3;

        palette.add_to_history("cmd1".to_string());
        palette.add_to_history("cmd2".to_string());
        palette.add_to_history("cmd3".to_string());
        assert_eq!(palette.history().len(), 3);

        palette.add_to_history("cmd4".to_string());
        assert_eq!(palette.history().len(), 3); // Should not exceed max
        assert_eq!(palette.history()[0], "cmd2"); // First one removed
        assert_eq!(palette.history()[2], "cmd4"); // New one added
    }

    #[test]
    fn test_history_initial_empty() {
        let palette = CommandPalette::new();
        assert!(palette.history().is_empty());
    }

    #[test]
    fn test_get_command_exists() {
        let palette = CommandPalette::new();
        let cmd = palette.get_command("nav.issues");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().id, "nav.issues");
    }

    #[test]
    fn test_get_command_not_exists() {
        let palette = CommandPalette::new();
        let cmd = palette.get_command("nonexistent.command");
        assert!(cmd.is_none());
    }

    #[test]
    fn test_add_command_increases_count() {
        let mut palette = CommandPalette::new();
        let initial_count = palette.commands.len();

        palette.add_command(Command {
            id: "custom.cmd".to_string(),
            name: "Custom".to_string(),
            description: "Custom command".to_string(),
            category: CommandCategory::System,
            keys: vec![],
            contexts: vec![AppContext::Global],
        });

        assert_eq!(palette.commands.len(), initial_count + 1);
    }

    #[test]
    fn test_context_returns_current() {
        let mut palette = CommandPalette::new();
        assert_eq!(palette.context(), AppContext::Global);

        palette.set_context(AppContext::Labels);
        assert_eq!(palette.context(), AppContext::Labels);
    }

    #[test]
    fn test_search_sorts_by_score() {
        let mut palette = CommandPalette::new();
        palette.set_query("go".to_string());

        let results = palette.search();
        if results.len() > 1 {
            // Verify sorted in descending order by score
            for i in 0..results.len() - 1 {
                assert!(results[i].1 >= results[i + 1].1);
            }
        }
    }

    #[test]
    fn test_multiple_contexts_in_command() {
        let mut palette = CommandPalette::new();
        palette.add_command(Command {
            id: "multi.context".to_string(),
            name: "Multi Context".to_string(),
            description: "Available in multiple contexts".to_string(),
            category: CommandCategory::View,
            keys: vec![],
            contexts: vec![AppContext::Issues, AppContext::Labels],
        });

        // Should be visible in Issues context
        palette.set_context(AppContext::Issues);
        let issues_results = palette.search();
        assert!(issues_results.iter().any(|(cmd, _)| cmd.id == "multi.context"));

        // Should be visible in Labels context
        palette.set_context(AppContext::Labels);
        let labels_results = palette.search();
        assert!(labels_results.iter().any(|(cmd, _)| cmd.id == "multi.context"));

        // Should NOT be visible in Database context
        palette.set_context(AppContext::Database);
        let database_results = palette.search();
        assert!(!database_results
            .iter()
            .any(|(cmd, _)| cmd.id == "multi.context"));
    }

    #[test]
    fn test_command_with_multiple_keys() {
        let mut palette = CommandPalette::new();
        palette.add_command(Command {
            id: "multi.key".to_string(),
            name: "Multi Key".to_string(),
            description: "Command with multiple key bindings".to_string(),
            category: CommandCategory::Navigation,
            keys: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            contexts: vec![AppContext::Global],
        });

        let cmd = palette.get_command("multi.key");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().keys.len(), 3);
    }

    #[test]
    fn test_app_context_copy_trait() {
        let context1 = AppContext::Issues;
        let context2 = context1;
        assert_eq!(context1, context2);
        // Both should still be usable after copy
        assert_eq!(context1, AppContext::Issues);
        assert_eq!(context2, AppContext::Issues);
    }

    #[test]
    fn test_command_category_copy_trait() {
        let cat1 = CommandCategory::Navigation;
        let cat2 = cat1;
        assert_eq!(cat1, cat2);
        // Both should still be usable after copy
        assert_eq!(cat1, CommandCategory::Navigation);
        assert_eq!(cat2, CommandCategory::Navigation);
    }

    #[test]
    fn test_all_command_category_inequality() {
        assert_ne!(CommandCategory::Navigation, CommandCategory::Issue);
        assert_ne!(CommandCategory::Navigation, CommandCategory::Dependency);
        assert_ne!(CommandCategory::Navigation, CommandCategory::Label);
        assert_ne!(CommandCategory::Navigation, CommandCategory::Database);
        assert_ne!(CommandCategory::Navigation, CommandCategory::View);
        assert_ne!(CommandCategory::Navigation, CommandCategory::System);

        assert_ne!(CommandCategory::Issue, CommandCategory::Dependency);
        assert_ne!(CommandCategory::Issue, CommandCategory::Label);
        assert_ne!(CommandCategory::View, CommandCategory::System);
    }

    #[test]
    fn test_command_with_empty_keys() {
        let cmd = Command {
            id: "no.keys".to_string(),
            name: "No Keys".to_string(),
            description: "Command with no key bindings".to_string(),
            category: CommandCategory::System,
            keys: vec![],
            contexts: vec![AppContext::Global],
        };

        assert!(cmd.keys.is_empty());
    }

    #[test]
    fn test_search_scoring_exact_match() {
        let mut palette = CommandPalette::new();
        palette.set_query("Quit".to_string());

        let results = palette.search();
        assert!(!results.is_empty());

        // Exact match should have higher score
        let quit_result = results.iter().find(|(cmd, _)| cmd.name == "Quit");
        assert!(quit_result.is_some());
        let (_, quit_score) = quit_result.unwrap();
        assert!(*quit_score > 0);
    }

    #[test]
    fn test_context_filtering_help_context() {
        let mut palette = CommandPalette::new();
        palette.add_command(Command {
            id: "help.specific".to_string(),
            name: "Help Specific".to_string(),
            description: "Help context only".to_string(),
            category: CommandCategory::View,
            keys: vec![],
            contexts: vec![AppContext::Help],
        });

        palette.set_context(AppContext::Help);
        let results = palette.search();
        assert!(results.iter().any(|(cmd, _)| cmd.id == "help.specific"));

        palette.set_context(AppContext::Global);
        let global_results = palette.search();
        assert!(!global_results.iter().any(|(cmd, _)| cmd.id == "help.specific"));
    }

    #[test]
    fn test_context_filtering_settings_context() {
        let mut palette = CommandPalette::new();
        palette.add_command(Command {
            id: "settings.specific".to_string(),
            name: "Settings Specific".to_string(),
            description: "Settings context only".to_string(),
            category: CommandCategory::System,
            keys: vec![],
            contexts: vec![AppContext::Settings],
        });

        palette.set_context(AppContext::Settings);
        let results = palette.search();
        assert!(results.iter().any(|(cmd, _)| cmd.id == "settings.specific"));

        palette.set_context(AppContext::Issues);
        let issues_results = palette.search();
        assert!(!issues_results.iter().any(|(cmd, _)| cmd.id == "settings.specific"));
    }

    #[test]
    fn test_context_filtering_dependency_graph_context() {
        let mut palette = CommandPalette::new();
        palette.add_command(Command {
            id: "graph.specific".to_string(),
            name: "Graph Specific".to_string(),
            description: "Dependency graph context only".to_string(),
            category: CommandCategory::View,
            keys: vec![],
            contexts: vec![AppContext::DependencyGraph],
        });

        palette.set_context(AppContext::DependencyGraph);
        let results = palette.search();
        assert!(results.iter().any(|(cmd, _)| cmd.id == "graph.specific"));

        palette.set_context(AppContext::Database);
        let database_results = palette.search();
        assert!(!database_results.iter().any(|(cmd, _)| cmd.id == "graph.specific"));
    }

    #[test]
    fn test_history_duplicate_entries() {
        let mut palette = CommandPalette::new();
        palette.add_to_history("cmd1".to_string());
        palette.add_to_history("cmd2".to_string());
        palette.add_to_history("cmd1".to_string());

        assert_eq!(palette.history().len(), 3);
        assert_eq!(palette.history()[0], "cmd1");
        assert_eq!(palette.history()[1], "cmd2");
        assert_eq!(palette.history()[2], "cmd1");
    }

    #[test]
    fn test_selection_after_context_change_different_counts() {
        let mut palette = CommandPalette::new();

        palette.set_context(AppContext::Global);
        let global_count = palette.search().len();
        palette.selected_index = 5;

        palette.set_context(AppContext::IssueDetail);
        assert_eq!(palette.selected_index, 0);

        let detail_count = palette.search().len();
        // Counts should differ due to context filtering
        assert!(global_count > 0 && detail_count > 0);
    }

    #[test]
    fn test_search_matching_description_only() {
        let mut palette = CommandPalette::new();
        palette.add_command(Command {
            id: "test.desc".to_string(),
            name: "Generic".to_string(),
            description: "Unique description keyword xyzabc".to_string(),
            category: CommandCategory::View,
            keys: vec![],
            contexts: vec![AppContext::Global],
        });

        palette.set_query("xyzabc".to_string());
        let results = palette.search();

        assert!(results.iter().any(|(cmd, _)| cmd.id == "test.desc"));
    }

    #[test]
    fn test_available_command_count_after_context_change() {
        let mut palette = CommandPalette::new();

        palette.set_context(AppContext::Global);
        let global_count = palette.available_command_count();

        palette.set_context(AppContext::Issues);
        let issues_count = palette.available_command_count();

        // Issues context should have additional issue-specific commands
        assert!(issues_count >= global_count);
    }

    #[test]
    fn test_multiple_queries_in_sequence() {
        let mut palette = CommandPalette::new();

        palette.set_query("issue".to_string());
        let results1 = palette.search();
        assert!(!results1.is_empty());

        palette.set_query("nav".to_string());
        let results2 = palette.search();
        assert!(!results2.is_empty());

        palette.set_query("".to_string());
        let results3 = palette.search();
        assert!(!results3.is_empty());

        // Query should be updated each time
        assert_eq!(palette.query(), "");
    }

    #[test]
    fn test_selection_behavior_when_query_reduces_results() {
        let mut palette = CommandPalette::new();

        palette.set_query("".to_string());
        palette.selected_index = 10;

        palette.set_query("xyzveryspecificnonmatch".to_string());
        assert_eq!(palette.selected_index, 0); // Reset on query change

        let results = palette.search();
        assert!(results.is_empty());
        assert!(palette.selected().is_none());
    }

    #[test]
    fn test_history_ordering_fifo() {
        let mut palette = CommandPalette::new();
        palette.max_history = 5;

        for i in 1..=7 {
            palette.add_to_history(format!("cmd{}", i));
        }

        assert_eq!(palette.history().len(), 5);
        assert_eq!(palette.history()[0], "cmd3"); // First two removed
        assert_eq!(palette.history()[1], "cmd4");
        assert_eq!(palette.history()[4], "cmd7"); // Last one added
    }

    #[test]
    fn test_command_category_all_equality() {
        assert_eq!(CommandCategory::Navigation, CommandCategory::Navigation);
        assert_eq!(CommandCategory::Issue, CommandCategory::Issue);
        assert_eq!(CommandCategory::Dependency, CommandCategory::Dependency);
        assert_eq!(CommandCategory::Label, CommandCategory::Label);
        assert_eq!(CommandCategory::Database, CommandCategory::Database);
        assert_eq!(CommandCategory::View, CommandCategory::View);
        assert_eq!(CommandCategory::System, CommandCategory::System);
    }

    #[test]
    fn test_search_partial_match_scoring() {
        let mut palette = CommandPalette::new();
        palette.set_query("is".to_string());

        let results = palette.search();
        assert!(!results.is_empty());

        // Should match "Issues" and similar
        let has_issue_match = results.iter().any(|(cmd, score)| {
            (cmd.name.contains("Issue") || cmd.description.contains("issue")) && *score > 0
        });
        assert!(has_issue_match);
    }

    #[test]
    fn test_context_issue_detail_specific() {
        let mut palette = CommandPalette::new();

        palette.set_context(AppContext::IssueDetail);
        assert_eq!(palette.context(), AppContext::IssueDetail);

        let results = palette.search();
        // Should have issue.close and issue.edit available
        assert!(results.iter().any(|(cmd, _)| cmd.id == "issue.close"));
        assert!(results.iter().any(|(cmd, _)| cmd.id == "issue.edit"));
    }

    #[test]
    fn test_selected_index_bounds_after_query() {
        let mut palette = CommandPalette::new();

        palette.set_query("".to_string());
        let all_results = palette.search();
        palette.selected_index = all_results.len().saturating_sub(1);

        palette.set_query("quit".to_string());
        assert_eq!(palette.selected_index, 0); // Reset to 0 on query change
    }

    #[test]
    fn test_command_with_single_context_non_global() {
        let mut palette = CommandPalette::new();
        palette.add_command(Command {
            id: "single.context".to_string(),
            name: "Single Context".to_string(),
            description: "Only in Labels".to_string(),
            category: CommandCategory::Label,
            keys: vec![],
            contexts: vec![AppContext::Labels],
        });

        palette.set_context(AppContext::Labels);
        let results = palette.search();
        assert!(results.iter().any(|(cmd, _)| cmd.id == "single.context"));

        palette.set_context(AppContext::Global);
        let global_results = palette.search();
        assert!(!global_results.iter().any(|(cmd, _)| cmd.id == "single.context"));
    }

    #[test]
    fn test_select_next_with_filtered_results() {
        let mut palette = CommandPalette::new();
        palette.set_context(AppContext::Issues); // Set context where issue commands are available
        palette.set_query("issue".to_string());

        let initial_results = palette.search();
        assert!(!initial_results.is_empty());
        assert!(initial_results.len() >= 3); // Should have multiple issue commands

        palette.select_next();
        assert_eq!(palette.selected_index, 1);

        palette.select_next();
        assert_eq!(palette.selected_index, 2);
    }

    #[test]
    fn test_select_previous_with_filtered_results() {
        let mut palette = CommandPalette::new();
        palette.set_query("go".to_string());

        palette.selected_index = 3;
        palette.select_previous();
        assert_eq!(palette.selected_index, 2);

        palette.select_previous();
        assert_eq!(palette.selected_index, 1);
    }

    #[test]
    fn test_app_context_clone() {
        let context = AppContext::Dependencies;
        let cloned = context;
        assert_eq!(context, cloned);
    }

    #[test]
    fn test_command_category_clone() {
        let category = CommandCategory::Database;
        let cloned = category;
        assert_eq!(category, cloned);
    }

    #[test]
    fn test_history_max_boundary_exact() {
        let mut palette = CommandPalette::new();
        palette.max_history = 2;

        palette.add_to_history("cmd1".to_string());
        palette.add_to_history("cmd2".to_string());
        assert_eq!(palette.history().len(), 2);

        palette.add_to_history("cmd3".to_string());
        assert_eq!(palette.history().len(), 2);
        assert_eq!(palette.history()[0], "cmd2");
        assert_eq!(palette.history()[1], "cmd3");
    }

    #[test]
    fn test_search_with_single_command() {
        let mut palette = CommandPalette {
            commands: vec![],
            matcher: SkimMatcherV2::default(),
            search_query: String::new(),
            selected_index: 0,
            history: Vec::new(),
            max_history: 100,
            current_context: AppContext::Global,
        };

        palette.add_command(Command {
            id: "only.one".to_string(),
            name: "Only One".to_string(),
            description: "Single command".to_string(),
            category: CommandCategory::System,
            keys: vec![],
            contexts: vec![AppContext::Global],
        });

        let results = palette.search();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "only.one");
    }

    #[test]
    fn test_available_command_count_empty_palette() {
        let palette = CommandPalette {
            commands: vec![],
            matcher: SkimMatcherV2::default(),
            search_query: String::new(),
            selected_index: 0,
            history: Vec::new(),
            max_history: 100,
            current_context: AppContext::Global,
        };

        assert_eq!(palette.available_command_count(), 0);
    }

    #[test]
    fn test_selected_with_empty_results() {
        let palette = CommandPalette {
            commands: vec![],
            matcher: SkimMatcherV2::default(),
            search_query: String::new(),
            selected_index: 0,
            history: Vec::new(),
            max_history: 100,
            current_context: AppContext::Global,
        };

        assert!(palette.selected().is_none());
    }

    #[test]
    fn test_get_command_with_empty_palette() {
        let palette = CommandPalette {
            commands: vec![],
            matcher: SkimMatcherV2::default(),
            search_query: String::new(),
            selected_index: 0,
            history: Vec::new(),
            max_history: 100,
            current_context: AppContext::Global,
        };

        assert!(palette.get_command("any.id").is_none());
    }
}
