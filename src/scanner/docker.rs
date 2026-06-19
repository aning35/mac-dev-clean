use super::{DeleteStrategy, RiskLevel, ScanResult, Scanner};
use std::path::PathBuf;
use std::process::Command;

pub struct DockerScanner;

impl Scanner for DockerScanner {
    fn name(&self) -> &str {
        "DockerScanner"
    }

    fn scan(&self, _roots: &[PathBuf]) -> Vec<ScanResult> {
        let mut results = Vec::new();

        // Check if docker is available
        if Command::new("docker").arg("--version").output().is_ok() {
            // Docker builder cache
            results.push(ScanResult {
                id: "docker:builder-cache".to_string(),
                category: "docker".to_string(),
                name: "Docker builder cache".to_string(),
                path: PathBuf::from("docker"),
                size_bytes: 0, // Mock size for now, could parse docker system df
                risk: RiskLevel::Confirm,
                action: "docker builder prune -f".to_string(),
                delete_strategy: DeleteStrategy::Permanent,
                reason: "Docker builder cache. Can be recreated.".to_string(),
                last_modified: None,
                selected_by_default: false,
            });

            // Docker volumes (Dangerous)
            results.push(ScanResult {
                id: "docker:volumes".to_string(),
                category: "docker".to_string(),
                name: "Docker volumes".to_string(),
                path: PathBuf::from("docker"),
                size_bytes: 0, 
                risk: RiskLevel::Dangerous,
                action: "docker volume prune -f".to_string(),
                delete_strategy: DeleteStrategy::Permanent,
                reason: "Docker volumes may contain database data.".to_string(),
                last_modified: None,
                selected_by_default: false,
            });
        }

        results
    }
}
