use super::{DeleteStrategy, RiskLevel, ScanResult, Scanner};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct DevScanner;

impl Scanner for DevScanner {
    fn name(&self) -> &str {
        "DevScanner"
    }

    fn scan(&self, roots: &[PathBuf]) -> Vec<ScanResult> {
        let mut results = Vec::new();

        for root in roots {
            let expanded_root = super::guard::expand_tilde(root);
            if !expanded_root.exists() {
                continue;
            }

            let mut it = WalkDir::new(expanded_root).into_iter();
            loop {
                let entry = match it.next() {
                    None => break,
                    Some(Err(_)) => continue,
                    Some(Ok(entry)) => entry,
                };

                let path = entry.path();
                if super::guard::is_forbidden(path) {
                    if path.is_dir() {
                        it.skip_current_dir();
                    }
                    continue;
                }

                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(res) = check_dev_cache(path, file_name) {
                        results.push(res);
                        if path.is_dir() {
                            it.skip_current_dir();
                        }
                    }
                }
            }
        }

        results
    }
}

fn check_dev_cache(path: &Path, file_name: &str) -> Option<ScanResult> {
    if !path.is_dir() {
        return None;
    }

    match file_name {
        "node_modules" => Some(ScanResult {
            id: format!("node_modules:{}", path.display()),
            category: "dev-dependency".to_string(),
            name: "node_modules".to_string(),
            path: path.to_path_buf(),
            size_bytes: get_size(path),
            risk: RiskLevel::Confirm,
            action: "delete-directory".to_string(),
            delete_strategy: DeleteStrategy::Trash,
            reason: "Node.js dependencies. Can be recreated.".to_string(),
            last_modified: None,
            selected_by_default: false,
        }),
        ".next" | ".nuxt" | ".turbo" | ".vite" | "dist" | "build" | "out" | "coverage" | ".nyc_output" => Some(ScanResult {
            id: format!("frontend-build:{}", path.display()),
            category: "dev-cache".to_string(),
            name: file_name.to_string(),
            path: path.to_path_buf(),
            size_bytes: get_size(path),
            risk: RiskLevel::Safe,
            action: "delete-directory".to_string(),
            delete_strategy: DeleteStrategy::Trash,
            reason: "Frontend build artifact/cache.".to_string(),
            last_modified: None,
            selected_by_default: true,
        }),
        "__pycache__" | ".pytest_cache" | ".ruff_cache" | ".mypy_cache" | ".tox" | ".nox" => Some(ScanResult {
            id: format!("pycache:{}", path.display()),
            category: "dev-cache".to_string(),
            name: file_name.to_string(),
            path: path.to_path_buf(),
            size_bytes: get_size(path),
            risk: RiskLevel::Safe,
            action: "delete-directory".to_string(),
            delete_strategy: DeleteStrategy::Trash,
            reason: "Python cache files will be regenerated automatically.".to_string(),
            last_modified: None,
            selected_by_default: true,
        }),
        "venv" | ".venv" | "env" => Some(ScanResult {
            id: format!("pyvenv:{}", path.display()),
            category: "dev-dependency".to_string(),
            name: file_name.to_string(),
            path: path.to_path_buf(),
            size_bytes: get_size(path),
            risk: RiskLevel::Confirm,
            action: "delete-directory".to_string(),
            delete_strategy: DeleteStrategy::Trash,
            reason: "Python virtual environment. Deleting will require reinstalling packages.".to_string(),
            last_modified: None,
            selected_by_default: false,
        }),
        ".gradle" => Some(ScanResult {
            id: format!("gradle:{}", path.display()),
            category: "dev-cache".to_string(),
            name: file_name.to_string(),
            path: path.to_path_buf(),
            size_bytes: get_size(path),
            risk: RiskLevel::Confirm,
            action: "delete-directory".to_string(),
            delete_strategy: DeleteStrategy::Trash,
            reason: "Gradle cache.".to_string(),
            last_modified: None,
            selected_by_default: false,
        }),
        ".dart_tool" | ".pub-cache" => Some(ScanResult {
            id: format!("flutter:{}", path.display()),
            category: "dev-cache".to_string(),
            name: file_name.to_string(),
            path: path.to_path_buf(),
            size_bytes: get_size(path),
            risk: RiskLevel::Confirm,
            action: "delete-directory".to_string(),
            delete_strategy: DeleteStrategy::Trash,
            reason: "Flutter/Dart build tools and cache.".to_string(),
            last_modified: None,
            selected_by_default: false,
        }),
        "target" => {
            if let Some(parent) = path.parent() {
                let is_rust = parent.join("Cargo.toml").exists();
                let is_maven = parent.join("pom.xml").exists();
                if is_rust || is_maven {
                    let name = if is_rust {
                        "target (Rust)".to_string()
                    } else {
                        "target (Maven)".to_string()
                    };
                    let reason = if is_rust {
                        "Rust build artifacts can be recreated by cargo build.".to_string()
                    } else {
                        "Maven build artifacts can be recreated by mvn clean build.".to_string()
                    };
                    let id = if is_rust {
                        format!("rust-target:{}", path.display())
                    } else {
                        format!("maven-target:{}", path.display())
                    };

                    return Some(ScanResult {
                        id,
                        category: "dev-cache".to_string(),
                        name,
                        path: path.to_path_buf(),
                        size_bytes: get_size(path),
                        risk: RiskLevel::Confirm,
                        action: "delete-directory".to_string(),
                        delete_strategy: DeleteStrategy::Trash,
                        reason,
                        last_modified: None,
                        selected_by_default: false,
                    });
                }
            }
            None
        }
        _ => None,
    }
}

fn get_size(path: &Path) -> u64 {
    let mut total_size = 0;
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            total_size += metadata.len();
        }
    }
    total_size
}
