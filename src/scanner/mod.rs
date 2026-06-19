pub mod guard;
pub mod dev;
pub mod docker;
pub mod ai;
pub mod pkgmgr;
pub mod ide;
pub mod chat;
pub mod downloads;

use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub enum RiskLevel {
    Safe,
    Confirm,
    Dangerous,
    Forbidden,
}

#[derive(Debug, Clone, Serialize)]
pub enum DeleteStrategy {
    Trash,
    Permanent,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub id: String,
    pub category: String,
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub risk: RiskLevel,
    pub action: String, // e.g., "delete-directory", "npm-cache-clean"
    pub delete_strategy: DeleteStrategy,
    pub reason: String,
    pub last_modified: Option<String>,
    pub selected_by_default: bool,
}

pub trait Scanner: Send + Sync {
    #[allow(dead_code)]
    fn name(&self) -> &str;
    fn scan(&self, roots: &[PathBuf]) -> Vec<ScanResult>;
}
