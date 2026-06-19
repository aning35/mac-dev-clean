use super::{DeleteStrategy, RiskLevel, ScanResult, Scanner};
use std::path::PathBuf;

use walkdir::WalkDir;

pub struct PkgMgrScanner;

fn get_size(path: &std::path::Path) -> u64 {
    let mut total_size = 0;
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            total_size += metadata.len();
        }
    }
    total_size
}

impl Scanner for PkgMgrScanner {
    fn name(&self) -> &str {
        "PkgMgrScanner"
    }

    fn scan(&self, _roots: &[PathBuf]) -> Vec<ScanResult> {
        let mut results = Vec::new();

        // Npm (approximate size via ~/.npm)
        let npm_path = super::guard::expand_tilde(&PathBuf::from("~/.npm"));
        if npm_path.exists() {
            results.push(ScanResult {
                id: "pkgmgr:npm".to_string(),
                category: "package-manager".to_string(),
                name: "npm cache".to_string(),
                path: npm_path.clone(),
                size_bytes: get_size(&npm_path), 
                risk: RiskLevel::Confirm,
                action: "npm cache clean --force".to_string(),
                delete_strategy: DeleteStrategy::Permanent,
                reason: "NPM global cache".to_string(),
                last_modified: None,
                selected_by_default: false,
            });
        }

        // Cargo
        let cargo_path = super::guard::expand_tilde(&PathBuf::from("~/.cargo/registry"));
        if cargo_path.exists() {
            results.push(ScanResult {
                id: "pkgmgr:cargo".to_string(),
                category: "package-manager".to_string(),
                name: "cargo cache".to_string(),
                path: cargo_path.clone(),
                size_bytes: get_size(&cargo_path), 
                risk: RiskLevel::Confirm,
                action: "delete-directory".to_string(),
                delete_strategy: DeleteStrategy::Trash,
                reason: "Cargo registry cache".to_string(),
                last_modified: None,
                selected_by_default: false,
            });
        }

        // Maven (.m2/repository)
        let maven_path = super::guard::expand_tilde(&PathBuf::from("~/.m2/repository"));
        if maven_path.exists() {
            results.push(ScanResult {
                id: "pkgmgr:maven".to_string(),
                category: "package-manager".to_string(),
                name: "Maven cache".to_string(),
                path: maven_path.clone(),
                size_bytes: get_size(&maven_path), 
                risk: RiskLevel::Confirm,
                action: "delete-directory".to_string(),
                delete_strategy: DeleteStrategy::Trash,
                reason: "Maven dependencies (.jar files). Re-downloading takes time.".to_string(),
                last_modified: None,
                selected_by_default: false,
            });
        }

        // Gradle (~/.gradle/caches)
        let gradle_path = super::guard::expand_tilde(&PathBuf::from("~/.gradle/caches"));
        if gradle_path.exists() {
            results.push(ScanResult {
                id: "pkgmgr:gradle".to_string(),
                category: "package-manager".to_string(),
                name: "Gradle cache".to_string(),
                path: gradle_path.clone(),
                size_bytes: get_size(&gradle_path), 
                risk: RiskLevel::Confirm,
                action: "delete-directory".to_string(),
                delete_strategy: DeleteStrategy::Trash,
                reason: "Gradle dependencies and build caches. Re-downloading takes time.".to_string(),
                last_modified: None,
                selected_by_default: false,
            });
        }

        // Go Modules (~/go/pkg/mod)
        let go_mod_path = super::guard::expand_tilde(&PathBuf::from("~/go/pkg/mod"));
        if go_mod_path.exists() {
            results.push(ScanResult {
                id: "pkgmgr:go_mod".to_string(),
                category: "package-manager".to_string(),
                name: "Go mod cache".to_string(),
                path: go_mod_path.clone(),
                size_bytes: get_size(&go_mod_path), 
                risk: RiskLevel::Confirm,
                action: "go clean -modcache".to_string(),
                delete_strategy: DeleteStrategy::Permanent,
                reason: "Go module cache. Go will redownload them via GOPROXY.".to_string(),
                last_modified: None,
                selected_by_default: false,
            });
        }

        // Go Build Cache (~/Library/Caches/go-build)
        let go_build_path = super::guard::expand_tilde(&PathBuf::from("~/Library/Caches/go-build"));
        if go_build_path.exists() {
            results.push(ScanResult {
                id: "pkgmgr:go_build".to_string(),
                category: "dev-cache".to_string(),
                name: "Go build cache".to_string(),
                path: go_build_path.clone(),
                size_bytes: get_size(&go_build_path), 
                risk: RiskLevel::Safe,
                action: "go clean -cache".to_string(),
                delete_strategy: DeleteStrategy::Permanent,
                reason: "Go compiled object files. Safe to delete, local builds will just take slightly longer.".to_string(),
                last_modified: None,
                selected_by_default: true,
            });
        }

        results
    }
}
