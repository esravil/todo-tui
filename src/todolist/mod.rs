use serde::{Deserialize, Serialize};

use crate::task::{Status, Task};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TodoList {
    pub items: Vec<Task>,
}

impl TodoList {
    pub fn add(&mut self, title: &str, priority: i8, notes: Option<String>) {
        self.items.push(Task::new(title, priority, notes));
        self.sort();
    }

    pub fn delete_index(&mut self, idx: usize) -> bool {
        if idx < self.items.len() {
            self.items.remove(idx);
            true
        } else {
            false
        }
    }

    pub fn toggle_done_index(&mut self, idx: usize) -> bool {
        if let Some(t) = self.items.get_mut(idx) {
            t.toggle_done();
            true
        } else {
            false
        }
    }

    pub fn find_index_by_id(&self, id: &str) -> Option<usize> {
        self.items.iter().position(|t| t.id == id)
    }

    /// Sort by priority ascending (1 = highest), then newest first
    pub fn sort(&mut self) {
        self.items
            .sort_by(|a, b| a.priority.cmp(&b.priority).then(b.created_at.cmp(&a.created_at)));
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn active_count(&self) -> usize {
        self.items.iter().filter(|t| t.status == Status::Pending).count()
    }

    pub fn done_count(&self) -> usize {
        self.items.iter().filter(|t| t.status == Status::Done).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_toggle() {
        let mut t = TodoList::default();
        t.add("Write tests", 1, None);
        assert_eq!(t.len(), 1);
        assert!(matches!(t.items[0].status, Status::Pending));
        t.toggle_done_index(0);
        assert!(matches!(t.items[0].status, Status::Done));
    }
}
