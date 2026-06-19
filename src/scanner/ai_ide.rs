use super::{DeleteStrategy, RiskLevel, ScanResult, Scanner};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct AiIdeScanner;

impl Scanner for AiIdeScanner {
    fn name(&self) -> &str {
        "AiIdeScanner"
    }

    fn scan(&self, _roots: &[PathBuf]) -> Vec<ScanResult> {
        let mut results = Vec::new();

        let paths_to_check = vec![
            // Cursor
            ("~/Library/Caches/Cursor", "Cursor Cache", RiskLevel::Confirm),
            ("~/Library/Application Support/Cursor/Cache", "Cursor App Cache", RiskLevel::Confirm),
            ("~/Library/Application Support/Cursor/CachedData", "Cursor CachedData", RiskLevel::Confirm),
            ("~/.cursor/extensions", "Cursor Extensions", RiskLevel::Dangerous),
            // Windsurf
            ("~/Library/Caches/com.exafunction.windsurf", "Windsurf Cache", RiskLevel::Confirm),
            ("~/Library/Application Support/Windsurf/Cache", "Windsurf App Cache", RiskLevel::Confirm),
            ("~/.codeium/windsurf/cascade", "Windsurf Cascade History", RiskLevel::Dangerous),
            ("~/.windsurf", "Windsurf Extensions", RiskLevel::Dangerous),
            // Claude Code
            ("~/.claude/projects", "Claude Code Project History", RiskLevel::Dangerous),
            // Cline / Roo Code
            ("~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev", "Cline Storage & History", RiskLevel::Dangerous),
            ("~/Library/Application Support/Code/User/globalStorage/rooveterinaryinc.roo-cline", "Roo Code Storage", RiskLevel::Dangerous),
            // GitHub Copilot
            ("~/Library/Application Support/github-copilot", "GitHub Copilot Data", RiskLevel::Confirm),
            // Gemini / Antigravity
            ("~/.gemini/antigravity-ide/brain", "Antigravity Brain & History", RiskLevel::Dangerous),
            ("~/.gemini/antigravity-ide/logs", "Antigravity Logs", RiskLevel::Confirm),
            // Zed
            ("~/Library/Caches/Zed", "Zed Cache", RiskLevel::Confirm),
            ("~/Library/Application Support/Zed/cache", "Zed App Cache", RiskLevel::Confirm),
            // Goose / Trae
            ("~/.local/share/goose", "Goose Data", RiskLevel::Dangerous),
            ("~/.config/goose", "Goose Config", RiskLevel::Dangerous),
            ("~/.trae/cache", "Trae Cache", RiskLevel::Confirm),
            ("~/Library/Application Support/Trae/Cache", "Trae App Cache", RiskLevel::Confirm),
            // Workbuddy
            ("~/.workbuddy/projects", "Workbuddy Projects & History", RiskLevel::Dangerous),
            ("~/Library/Application Support/Workbuddy", "Workbuddy App Cache", RiskLevel::Confirm),
            // Qoder
            ("~/Library/Application Support/Qoder/SharedClientCache", "Qoder Shared Cache", RiskLevel::Confirm),
            ("~/.qoder/commands", "Qoder Custom Commands", RiskLevel::Dangerous),
            ("~/Library/Application Support/Qoder", "Qoder App Data", RiskLevel::Confirm),
        ];

        for (p, name, risk) in paths_to_check {
            let expanded = super::guard::expand_tilde(&PathBuf::from(p));
            if expanded.exists() {
                results.push(ScanResult {
                    id: format!("ai-ide:{}", p),
                    category: "ai-ide-cache".to_string(),
                    name: name.to_string(),
                    path: expanded.clone(),
                    size_bytes: get_size(&expanded),
                    risk,
                    action: "delete-directory".to_string(),
                    delete_strategy: DeleteStrategy::Trash,
                    reason: format!("{} large cache, logs, or history data.", name),
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
