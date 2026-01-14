use std::collections::VecDeque;
use std::time::SystemTime;

/// Entry in the undo history representing a reversible operation
#[derive(Debug, Clone)]
pub struct UndoEntry {
    /// Human-readable description of the operation
    pub description: String,
    /// bd CLI command arguments that will reverse this operation
    pub reverse_command: Vec<String>,
    /// When this operation was performed
    pub timestamp: SystemTime,
}

/// Ring buffer for storing undo history
/// Maintains a limited number of recent operations that can be undone
#[derive(Debug)]
pub struct UndoHistory {
    entries: VecDeque<UndoEntry>,
    max_size: usize,
}

impl UndoHistory {
    /// Create a new undo history with the specified maximum size
    ///
    /// # Arguments
    /// * `max_size` - Maximum number of operations to remember
    ///
    /// # Example
    /// ```
    /// let history = UndoHistory::new(20);
    /// ```
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Add a new operation to the history
    /// If the history is full, the oldest entry will be removed
    ///
    /// # Arguments
    /// * `entry` - The undo entry to add
    pub fn push(&mut self, entry: UndoEntry) {
        if self.entries.len() >= self.max_size {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Remove and return the most recent operation
    /// Returns None if history is empty
    pub fn pop(&mut self) -> Option<UndoEntry> {
        self.entries.pop_back()
    }

    /// View the most recent operation without removing it
    /// Returns None if history is empty
    pub fn peek(&self) -> Option<&UndoEntry> {
        self.entries.back()
    }

    /// Get the number of operations in the history
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries from the history
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get an iterator over all entries (oldest first)
    pub fn iter(&self) -> impl Iterator<Item = &UndoEntry> {
        self.entries.iter()
    }
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_history_new() {
        let history = UndoHistory::new(5);
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
    }

    #[test]
    fn test_undo_history_push_pop() {
        let mut history = UndoHistory::new(3);

        history.push(UndoEntry {
            description: "Op 1".to_string(),
            reverse_command: vec!["bd".to_string()],
            timestamp: SystemTime::now(),
        });

        assert_eq!(history.len(), 1);
        assert!(!history.is_empty());

        let entry = history.pop();
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().description, "Op 1");
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
    }

    #[test]
    fn test_undo_history_max_size() {
        let mut history = UndoHistory::new(2);

        for i in 0..5 {
            history.push(UndoEntry {
                description: format!("Op {}", i),
                reverse_command: vec![],
                timestamp: SystemTime::now(),
            });
        }

        // Should only keep last 2 operations
        assert_eq!(history.len(), 2);

        let entry1 = history.pop().unwrap();
        assert_eq!(entry1.description, "Op 4");

        let entry2 = history.pop().unwrap();
        assert_eq!(entry2.description, "Op 3");

        assert!(history.pop().is_none());
    }

    #[test]
    fn test_undo_history_peek() {
        let mut history = UndoHistory::new(5);

        assert!(history.peek().is_none());

        history.push(UndoEntry {
            description: "Test".to_string(),
            reverse_command: vec![],
            timestamp: SystemTime::now(),
        });

        assert_eq!(history.peek().unwrap().description, "Test");
        assert_eq!(history.len(), 1); // peek doesn't remove
    }

    #[test]
    fn test_undo_history_clear() {
        let mut history = UndoHistory::new(5);

        for i in 0..3 {
            history.push(UndoEntry {
                description: format!("Op {}", i),
                reverse_command: vec![],
                timestamp: SystemTime::now(),
            });
        }

        assert_eq!(history.len(), 3);
        history.clear();
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
    }

    #[test]
    fn test_undo_history_iter() {
        let mut history = UndoHistory::new(5);

        for i in 0..3 {
            history.push(UndoEntry {
                description: format!("Op {}", i),
                reverse_command: vec![],
                timestamp: SystemTime::now(),
            });
        }

        let descriptions: Vec<String> = history.iter().map(|e| e.description.clone()).collect();

        assert_eq!(descriptions, vec!["Op 0", "Op 1", "Op 2"]);
    }

    #[test]
    fn test_undo_history_default() {
        let history = UndoHistory::default();
        assert_eq!(history.max_size, 20);
        assert!(history.is_empty());
    }
}
