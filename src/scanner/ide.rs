use super::{DeleteStrategy, RiskLevel, ScanResult, Scanner};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct IdeScanner;

impl Scanner for IdeScanner {
    fn name(&self) -> &str {
        "IdeScanner"
    }

    fn scan(&self, _roots: &[PathBuf]) -> Vec<ScanResult> {
        let mut results = Vec::new();

        let paths_to_check = vec![
            ("~/Library/Developer/Xcode/DerivedData", "Xcode DerivedData", RiskLevel::Confirm),
            ("~/Library/Developer/Xcode/Archives", "Xcode Archives", RiskLevel::Dangerous),
            ("~/Library/Developer/CoreSimulator", "iOS Simulators", RiskLevel::Dangerous),
            ("~/.android/avd", "Android Emulators", RiskLevel::Dangerous),
            ("~/Library/Caches/Google/AndroidStudio", "Android Studio Caches", RiskLevel::Confirm),
        ];

        for (p, name, risk) in paths_to_check {
            let expanded = super::guard::expand_tilde(&PathBuf::from(p));
            if expanded.exists() {
                results.push(ScanResult {
                    id: format!("ide:{}", p),
                    category: "ide-cache".to_string(),
                    name: name.to_string(),
                    path: expanded.clone(),
                    size_bytes: get_size(&expanded),
                    risk,
                    action: "delete-directory".to_string(),
                    delete_strategy: DeleteStrategy::Trash,
                    reason: format!("{} large cache or data.", name),
                    last_modified: None,
                    selected_by_default: false,
                });
            }
        }

        results
    }
}

fn get_size(path: &std::path::Path) -> u64 {
    let mut total_size = 0;
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            total_size += metadata.len();
        }
    }
    total_size
}
