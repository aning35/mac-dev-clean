use anyhow::{Context, Result};
use std::path::Path;

pub fn move_to_trash(path: &Path) -> Result<()> {
    // We use the `trash` crate to safely move files to the macOS Trash
    trash::delete(path).with_context(|| format!("Failed to move {} to trash", path.display()))
}
