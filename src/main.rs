mod cli;
mod config;
mod executor;
mod scanner;
mod report;
mod gui;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use colored::*;
use dialoguer::{MultiSelect, theme::ColorfulTheme};

fn main() {
    let cli = Cli::parse();
    let config = Config::load();

    let scanners: Vec<Box<dyn scanner::Scanner>> = vec![
        Box::new(scanner::dev::DevScanner),
        Box::new(scanner::pkgmgr::PkgMgrScanner),
        Box::new(scanner::docker::DockerScanner),
        Box::new(scanner::ai::AiScanner),
        Box::new(scanner::ide::IdeScanner),
        Box::new(scanner::chat::ChatScanner),
        Box::new(scanner::downloads::DownloadsScanner),
        Box::new(scanner::ai_ide::AiIdeScanner),
    ];

    let command = cli.command.unwrap_or(Commands::Ui);

    match &command {
        Commands::Scan { json } => {
            println!("{}", "Scanning system for development caches...".cyan().bold());
            let mut all_results = Vec::new();
            for scanner in &scanners {
                all_results.extend(scanner.scan(&config.project_roots));
            }

            all_results.sort_by(|a, b| a.category.cmp(&b.category).then_with(|| b.size_bytes.cmp(&a.size_bytes)));

            if *json {
                let j = serde_json::to_string_pretty(&all_results).unwrap_or_default();
                println!("{}", j);
            } else {
                let mut total_size = 0;
                for res in &all_results {
                    let path_str = res.path.display().to_string();
                    let display_path = if path_str.ends_with(&res.name) {
                        res.path.parent().map(|p| p.display().to_string()).unwrap_or(path_str)
                    } else {
                        path_str
                    };
                    println!("- {:<15} | {:<50} | {:>10}  [{:?}]", res.name, display_path, crate::utils::format_size(res.size_bytes), res.risk);
                    total_size += res.size_bytes;
                }
                println!("\n{}", format!("Total cache size found: {}", crate::utils::format_size(total_size)).green().bold());
            }
        }
        Commands::Clean { safe, deep, dry_run } => {
            println!("{}", "Starting MacDevClean...".cyan().bold());
            
            let mut all_results = Vec::new();
            for scanner in &scanners {
                all_results.extend(scanner.scan(&config.project_roots));
            }

            all_results.sort_by(|a, b| a.category.cmp(&b.category).then_with(|| b.size_bytes.cmp(&a.size_bytes)));

            if all_results.is_empty() {
                println!("{}", "No caches found to clean!".green());
                return;
            }

            let candidates: Vec<_> = all_results.into_iter().filter(|r| {
                match r.risk {
                    scanner::RiskLevel::Safe => true,
                    scanner::RiskLevel::Confirm => *deep || !*safe,
                    scanner::RiskLevel::Dangerous => *deep,
                    scanner::RiskLevel::Forbidden => false,
                }
            }).collect();

            if candidates.is_empty() {
                println!("{}", "No eligible items to clean based on current flags (--safe / --deep).".yellow());
                return;
            }

            let items: Vec<String> = candidates.iter().map(|r| {
                let path_str = r.path.display().to_string();
                let display_path = if path_str.ends_with(&r.name) {
                    r.path.parent().map(|p| p.display().to_string()).unwrap_or(path_str)
                } else {
                    path_str
                };
                format!("{:<15} | {:<50} | {:>10} [{:?}]", r.name, display_path, crate::utils::format_size(r.size_bytes), r.risk)
            }).collect();

            let defaults: Vec<bool> = candidates.iter().map(|r| {
                matches!(r.risk, scanner::RiskLevel::Safe) || r.selected_by_default
            }).collect();

            println!("\nSelect items to clean (Space to select, Enter to confirm):");
            let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                .items(&items)
                .defaults(&defaults)
                .max_length(15)
                .interact()
                .unwrap();

            if selections.is_empty() {
                println!("No items selected. Exiting.");
                return;
            }

            let mut freed_bytes = 0;
            let mut cleaned_items = Vec::new();
            for idx in selections {
                let target = &candidates[idx];
                println!("Cleaning {} ({}) ...", target.name, target.path.display());
                if let Err(e) = executor::execute_action(target, *dry_run) {
                    eprintln!("{}", format!("Failed to clean {}: {}", target.name, e).red());
                } else {
                    freed_bytes += target.size_bytes;
                    cleaned_items.push(target);
                }
            }

            if *dry_run {
                println!("\n{}", format!("[DRY RUN] Would have freed {}", crate::utils::format_size(freed_bytes)).yellow().bold());
            } else {
                println!("\n{}", format!("Successfully freed {}!", crate::utils::format_size(freed_bytes)).green().bold());
                report::generate_report(freed_bytes, &cleaned_items, *dry_run);
            }
        }
        Commands::Config { command: _ } => {
            println!("Config commands not fully implemented in MVP.");
        }
        Commands::History => {
            println!("History tracking not fully implemented in MVP.");
        }
        Commands::Ai { command: _ } => {
            println!("Advanced AI commands not fully implemented in MVP.");
        }
        Commands::Ui => {
            println!("{}", "Starting MacDevClean Native UI...".cyan().bold());
            let arc_scanners: std::sync::Arc<Vec<Box<dyn scanner::Scanner + Send + Sync>>> = std::sync::Arc::new(vec![
                Box::new(scanner::dev::DevScanner),
                Box::new(scanner::pkgmgr::PkgMgrScanner),
                Box::new(scanner::docker::DockerScanner),
                Box::new(scanner::ai::AiScanner),
                Box::new(scanner::ide::IdeScanner),
                Box::new(scanner::chat::ChatScanner),
                Box::new(scanner::downloads::DownloadsScanner),
                Box::new(scanner::ai_ide::AiIdeScanner),
            ]);
            
            gui::start_gui(config, arc_scanners);
        }
    }
}
