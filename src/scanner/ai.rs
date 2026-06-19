use super::{DeleteStrategy, RiskLevel, ScanResult, Scanner};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct AiScanner;

impl Scanner for AiScanner {
    fn name(&self) -> &str {
        "AiScanner"
    }

    fn scan(&self, _roots: &[PathBuf]) -> Vec<ScanResult> {
        let mut results = Vec::new();

        // Common AI paths
        let paths_to_check = vec![
            ("~/.cache/huggingface", "Hugging Face Cache", RiskLevel::Confirm),
            ("~/.cache/modelscope", "ModelScope Cache", RiskLevel::Confirm),
            ("~/.ollama/models", "Ollama Models", RiskLevel::Dangerous),
            ("~/.cache/torch", "PyTorch Cache", RiskLevel::Confirm),
            ("~/Library/Application Support/LM Studio", "LM Studio Models", RiskLevel::Dangerous),
            ("~/ComfyUI/models", "ComfyUI Models", RiskLevel::Dangerous),
            ("~/ComfyUI/output", "ComfyUI Outputs", RiskLevel::Confirm),
            ("~/ComfyUI/temp", "ComfyUI Temp", RiskLevel::Safe),
            ("~/stable-diffusion-webui/models", "Stable Diffusion Models", RiskLevel::Dangerous),
            ("~/.keras", "Keras Cache", RiskLevel::Confirm),
            ("~/Models", "User Models Directory", RiskLevel::Dangerous),
            ("~/AI", "User AI Directory", RiskLevel::Dangerous),
            ("~/ai-models", "User AI Models Directory", RiskLevel::Dangerous),
        ];

        for (p, name, risk) in paths_to_check {
            let expanded = super::guard::expand_tilde(&PathBuf::from(p));
            if expanded.exists() {
                results.push(ScanResult {
                    id: format!("ai:{}", p),
                    category: "ai-model".to_string(),
                    name: name.to_string(),
                    path: expanded.clone(),
                    size_bytes: get_size(&expanded),
                    risk,
                    action: "delete-directory".to_string(),
                    delete_strategy: DeleteStrategy::Trash,
                    reason: format!("{} cache or downloaded models.", name),
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
