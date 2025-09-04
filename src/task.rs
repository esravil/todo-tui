use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    Pending,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub priority: i8,
    pub status: Status,
    pub created_at: i64, // unix seconds
}

impl Task {
    pub fn new(title: impl Into<String>, priority: i8, notes: Option<String>) -> Self {
        let title = title.into();
        let id = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            id,
            title,
            notes,
            priority,
            status: Status::Pending,
            created_at: now,
        }
    }

    pub fn toggle_done(&mut self) {
        self.status = match self.status {
            Status::Pending => Status::Done,
            Status::Done => Status::Pending,
        };
    }

    pub fn is_done(&self) -> bool {
        self.status == Status::Done
    }
}
