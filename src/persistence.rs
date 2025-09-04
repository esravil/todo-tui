use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

use crate::todolist::TodoList;

pub fn default_path() -> Result<PathBuf> {
    let proj = ProjectDirs::from("dev", "esravil", "todo-tui")
        .ok_or_else(|| anyhow!("Cannot determine data directory"))?;
    let dir = proj.data_dir().to_path_buf();
    Ok(dir.join("todos.json"))
}

pub fn load(path: &Path) -> Result<TodoList> {
    if !path.exists() {
        return Ok(TodoList::default());
    }
    let bytes = fs::read(path)?;
    let list = serde_json::from_slice::<TodoList>(&bytes)?;
    Ok(list)
}

pub fn save(path: &Path, list: &TodoList) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(list)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&tmp, &bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
