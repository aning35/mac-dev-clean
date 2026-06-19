use crate::scanner::ScanResult;
use std::path::PathBuf;
use std::fs;
use chrono::Local;

pub fn generate_report(freed_bytes: u64, cleaned: &[&ScanResult], dry_run: bool) {
    if dry_run || cleaned.is_empty() { return; }
    
    let report_dir = crate::scanner::guard::expand_tilde(&PathBuf::from("~/.macdevclean/reports"));
    let _ = fs::create_dir_all(&report_dir);
    
    let timestamp = Local::now().format("%Y-%m-%d-%H%M%S").to_string();
    
    // JSON
    let json_path = report_dir.join(format!("{}.json", timestamp));
    let json_data = serde_json::json!({
        "timestamp": timestamp,
        "freed_bytes": freed_bytes,
        "items": cleaned
    });
    let _ = fs::write(&json_path, serde_json::to_string_pretty(&json_data).unwrap_or_default());
    
    // Markdown
    let md_path = report_dir.join(format!("{}.md", timestamp));
    let mut md_content = format!("# MacDevClean Report ({})\n\n", timestamp);
    md_content.push_str(&format!("**Total Freed**: {}\n\n", crate::utils::format_size(freed_bytes)));
    md_content.push_str("## Cleaned Items\n\n");
    for item in cleaned {
        md_content.push_str(&format!("- **{}** (`{}`): {}\n", item.name, item.path.display(), crate::utils::format_size(item.size_bytes)));
    }
    let _ = fs::write(&md_path, md_content);

    println!("Report generated at: {}", md_path.display());
}
