use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub project_roots: Vec<PathBuf>,
    pub default_mode: String,
    pub delete_strategy: DeleteStrategy,
    pub downloads: DownloadsConfig,
    pub collaboration_apps: CollaborationAppsConfig,
    pub ai_models: AiModelsConfig,
    pub exclude: Vec<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteStrategy {
    pub default: String,
    pub permanent_for_safe_cache: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadsConfig {
    pub enabled: bool,
    pub older_than_days: u32,
    pub min_file_size_mb: u32,
    pub extensions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollaborationAppsConfig {
    pub enabled: bool,
    pub older_than_days: u32,
    pub min_file_size_mb: u32,
    pub apps: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AiModelsConfig {
    pub enabled: bool,
    pub scan_only_by_default: bool,
    pub detect_duplicates: bool,
    pub custom_dirs: Vec<PathBuf>,
}

impl Config {
    pub fn load() -> Self {
        let config_path = crate::scanner::guard::expand_tilde(&PathBuf::from("~/.macdevclean/config.yaml"));
        if config_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_yaml::from_str(&contents) {
                    return config;
                }
            }
        } else {
            if let Some(parent) = config_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let default_config = Config::default();
            if let Ok(yaml) = serde_yaml::to_string(&default_config) {
                let _ = std::fs::write(&config_path, yaml);
            }
            return default_config;
        }
        Config::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            project_roots: vec![
                PathBuf::from("~/workspace"),
                PathBuf::from("~/Projects"),
                PathBuf::from("~/Code"),
                PathBuf::from("~/Developer"),
            ],
            default_mode: "safe".to_string(),
            delete_strategy: DeleteStrategy {
                default: "trash".to_string(),
                permanent_for_safe_cache: true,
            },
            downloads: DownloadsConfig {
                enabled: true,
                older_than_days: 30,
                min_file_size_mb: 50,
                extensions: vec![
                    ".dmg".to_string(),
                    ".pkg".to_string(),
                    ".zip".to_string(),
                    ".tar.gz".to_string(),
                    ".tgz".to_string(),
                    ".iso".to_string(),
                    ".ipa".to_string(),
                    ".apk".to_string(),
                ],
            },
            collaboration_apps: CollaborationAppsConfig {
                enabled: true,
                older_than_days: 30,
                min_file_size_mb: 10,
                apps: vec![
                    "wecom".to_string(),
                    "feishu".to_string(),
                    "dingtalk".to_string(),
                    "wechat".to_string(),
                ],
            },
            ai_models: AiModelsConfig {
                enabled: true,
                scan_only_by_default: true,
                detect_duplicates: true,
                custom_dirs: vec![
                    PathBuf::from("~/Models"),
                    PathBuf::from("~/AI"),
                    PathBuf::from("~/ai-models"),
                ],
            },
            exclude: vec![
                PathBuf::from("~/.ssh"),
                PathBuf::from("~/.gnupg"),
                PathBuf::from("~/.kube"),
                PathBuf::from("~/Documents"),
            ],
        }
    }
}
