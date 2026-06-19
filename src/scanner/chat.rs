use super::{DeleteStrategy, RiskLevel, ScanResult, Scanner};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct ChatScanner;

/// Represents a specific cache subdirectory within a chat app container.
struct ChatCacheTarget {
    /// Base container path (with ~)
    container: &'static str,
    /// Subdirectory relative to the container that is safe to clean
    sub_path: &'static str,
    /// Human-readable name
    name: &'static str,
    /// Risk level
    risk: RiskLevel,
    /// Reason for deletion
    reason: &'static str,
}

impl Scanner for ChatScanner {
    fn name(&self) -> &str {
        "ChatScanner"
    }

    fn scan(&self, _roots: &[PathBuf]) -> Vec<ScanResult> {
        let mut results = Vec::new();

        let targets = vec![
            // ========== 企业微信 (WeCom) ==========
            ChatCacheTarget {
                container: "~/Library/Containers/com.tencent.WeWorkMac",
                sub_path: "",
                name: "企业微信 总占用",
                risk: RiskLevel::Forbidden,
                reason: "WeCom total workspace size. Size report only. Do not delete directly.",
            },

            // ========== 微信 (WeChat) ==========
            ChatCacheTarget {
                container: "~/Library/Containers/com.tencent.xinWeChat",
                sub_path: "",
                name: "微信 总占用",
                risk: RiskLevel::Forbidden,
                reason: "WeChat total workspace size. Size report only. Do not delete directly.",
            },

            // ========== 飞书 (Feishu/Lark) ==========
            ChatCacheTarget {
                container: "~/Library/Containers/com.electron.feishu",
                sub_path: "",
                name: "飞书 总占用",
                risk: RiskLevel::Forbidden,
                reason: "Feishu total workspace size. Size report only.",
            },
            ChatCacheTarget {
                container: "~/Library/Containers/com.electron.lark",
                sub_path: "",
                name: "Lark 总占用",
                risk: RiskLevel::Forbidden,
                reason: "Lark total workspace size. Size report only.",
            },

            // ========== 钉钉 (DingTalk) ==========
            ChatCacheTarget {
                container: "~/Library/Application Support/DingTalkMac",
                sub_path: "",
                name: "钉钉 总占用 (App Support)",
                risk: RiskLevel::Forbidden,
                reason: "DingTalk total workspace size. Size report only.",
            },
            ChatCacheTarget {
                container: "~/Library/Containers/com.alibaba.DingTalkMac",
                sub_path: "",
                name: "钉钉 总占用 (Container)",
                risk: RiskLevel::Forbidden,
                reason: "DingTalk sandbox container size. Size report only.",
            },

            // ========== QQ ==========
            ChatCacheTarget {
                container: "~/Library/Containers/com.tencent.qq",
                sub_path: "",
                name: "QQ 总占用",
                risk: RiskLevel::Forbidden,
                reason: "QQ total workspace size. Size report only.",
            },
            ChatCacheTarget {
                container: "~/Library/Containers/com.tencent.qqmac",
                sub_path: "",
                name: "QQ(NT版) 总占用",
                risk: RiskLevel::Forbidden,
                reason: "QQ NT total workspace size. Size report only.",
            },
        ];

        for target in &targets {
            let base = super::guard::expand_tilde(&PathBuf::from(target.container));
            let full_path = if target.sub_path.is_empty() {
                base.clone()
            } else {
                base.join(target.sub_path)
            };

            if full_path.exists() {
                let size = get_size(&full_path);
                // Only add items that have meaningful size (> 1 MB)
                if size > 1024 * 1024 {
                    results.push(ScanResult {
                        id: format!("chat:{}:{}", target.container, target.sub_path),
                        category: "collaboration-app".to_string(),
                        name: target.name.to_string(),
                        path: full_path,
                        size_bytes: size,
                        risk: target.risk.clone(),
                        action: "delete-directory".to_string(),
                        delete_strategy: DeleteStrategy::Trash,
                        reason: target.reason.to_string(),
                        last_modified: None,
                        selected_by_default: matches!(target.risk, RiskLevel::Safe),
                    });
                }
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
