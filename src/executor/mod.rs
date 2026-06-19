pub mod trash_ops;

use crate::scanner::{DeleteStrategy, ScanResult};
use std::process::Command;

pub fn execute_action(result: &ScanResult, dry_run: bool) -> anyhow::Result<()> {
    if dry_run {
        println!("[DRY RUN] Would execute action for: {}", result.name);
        return Ok(());
    }

    match result.delete_strategy {
        DeleteStrategy::Trash => {
            if result.action == "delete-directory" || result.action == "delete-file" {
                trash_ops::move_to_trash(&result.path)?;
            }
        }
        DeleteStrategy::Permanent => {
            if result.action.starts_with("docker ") || result.action.starts_with("npm ") {
                let parts: Vec<&str> = result.action.split_whitespace().collect();
                if !parts.is_empty() {
                    let mut cmd = Command::new(parts[0]);
                    cmd.args(&parts[1..]);
                    let _ = cmd.output()?;
                }
            } else if result.action == "delete-directory" {
                if !crate::scanner::guard::is_forbidden(&result.path) {
                    std::fs::remove_dir_all(&result.path)?;
                }
            } else if result.action == "delete-file" {
                if !crate::scanner::guard::is_forbidden(&result.path) {
                    std::fs::remove_file(&result.path)?;
                }
            }
        }
    }

    Ok(())
}
