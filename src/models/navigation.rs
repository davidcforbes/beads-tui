/// Navigation state and history management

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    IssueList,
    IssueDetail(String), // Issue ID
    Dependencies,
    Labels,
    Database,
    Help,
}

#[derive(Debug)]
pub struct NavigationStack {
    history: Vec<View>,
    current: usize,
}

impl NavigationStack {
    pub fn new() -> Self {
        Self {
            history: vec![View::IssueList],
            current: 0,
        }
    }

    pub fn push(&mut self, view: View) {
        // Remove any forward history if we're not at the end
        self.history.truncate(self.current + 1);
        self.history.push(view);
        self.current += 1;
    }

    pub fn back(&mut self) -> Option<&View> {
        if self.current > 0 {
            self.current -= 1;
            Some(&self.history[self.current])
        } else {
            None
        }
    }

    pub fn forward(&mut self) -> Option<&View> {
        if self.current + 1 < self.history.len() {
            self.current += 1;
            Some(&self.history[self.current])
        } else {
            None
        }
    }

    pub fn current(&self) -> &View {
        &self.history[self.current]
    }

    pub fn can_go_back(&self) -> bool {
        self.current > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.current + 1 < self.history.len()
    }
}

impl Default for NavigationStack {
    fn default() -> Self {
        Self::new()
    }
}
