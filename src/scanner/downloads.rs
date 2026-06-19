use super::{DeleteStrategy, RiskLevel, ScanResult, Scanner};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct DownloadsScanner;

impl Scanner for DownloadsScanner {
    fn name(&self) -> &str {
        "DownloadsScanner"
    }

    fn scan(&self, _roots: &[PathBuf]) -> Vec<ScanResult> {
        let mut results = Vec::new();
        let downloads_path = super::guard::expand_tilde(&PathBuf::from("~/Downloads"));

        if !downloads_path.exists() {
            return results;
        }

        let installer_exts = vec!["dmg", "pkg", "iso", "ipa", "apk"];
        let archive_exts = vec!["zip", "gz", "tgz", "tar", "rar", "7z"];

        for entry in WalkDir::new(&downloads_path).max_depth(2).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                
                if size_bytes > 0 {
                    if installer_exts.contains(&ext_lower.as_str()) {
                        results.push(ScanResult {
                            id: format!("download-installer:{}", path.display()),
                            category: "installer".to_string(),
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: path.to_path_buf(),
                            size_bytes,
                            risk: RiskLevel::Safe, // Installers are usually safe to delete
                            action: "delete-file".to_string(),
                            delete_strategy: DeleteStrategy::Trash,
                            reason: "Old installer in Downloads folder.".to_string(),
                            last_modified: None,
                            selected_by_default: true,
                        });
                    } else if archive_exts.contains(&ext_lower.as_str()) {
                        results.push(ScanResult {
                            id: format!("download-archive:{}", path.display()),
                            category: "archive".to_string(),
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: path.to_path_buf(),
                            size_bytes,
                            risk: RiskLevel::Confirm, // General archives might contain important data
                            action: "delete-file".to_string(),
                            delete_strategy: DeleteStrategy::Trash,
                            reason: "Archive file in Downloads folder. Check contents before deleting.".to_string(),
                            last_modified: None,
                            selected_by_default: false,
                        });
                    }
                }
            }
        }

        results
    }
}
