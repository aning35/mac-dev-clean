use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::Config;
use crate::scanner::{Scanner, ScanResult};
use crate::executor;
use crate::report;

pub struct MacDevCleanApp {
    config: Arc<Config>,
    scanners: Arc<Vec<Box<dyn Scanner + Send + Sync>>>,
    results: Arc<Mutex<Vec<ScanResult>>>,
    selected: std::collections::HashSet<String>,
    is_scanning: Arc<Mutex<bool>>,
    message: String,
    search_query: String,
    show_about: bool,
    show_confirm_clean: bool,
}

impl MacDevCleanApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config, scanners: Arc<Vec<Box<dyn Scanner + Send + Sync>>>) -> Self {
        // Load CJK font for Chinese character support
        let mut fonts = egui::FontDefinitions::default();
        
        // Try loading macOS system CJK font
        let cjk_font_paths = [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
        ];
        
        for font_path in &cjk_font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "cjk_font".to_owned(),
                    Arc::new(egui::FontData::from_owned(font_data)),
                );
                // Add CJK font as fallback for proportional text
                fonts.families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push("cjk_font".to_owned());
                // Also for monospace
                fonts.families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("cjk_font".to_owned());
                break;
            }
        }
        
        cc.egui_ctx.set_fonts(fonts);

        // Set a larger default font size
        let mut style = (*cc.egui_ctx.global_style()).clone();
        for font_id in style.text_styles.values_mut() {
            font_id.size *= 1.2;
        }
        cc.egui_ctx.set_global_style(style);

        Self {
            config: Arc::new(config),
            scanners,
            results: Arc::new(Mutex::new(Vec::new())),
            selected: std::collections::HashSet::new(),
            is_scanning: Arc::new(Mutex::new(false)),
            message: "Ready to scan. Click 'Scan System' to begin.".to_string(),
            search_query: String::new(),
            show_about: false,
            show_confirm_clean: false,
        }
    }

    fn trigger_scan(&mut self) {
        let is_scanning_clone = self.is_scanning.clone();
        let mut scanning_guard = is_scanning_clone.lock().unwrap();
        if *scanning_guard {
            return;
        }
        *scanning_guard = true;
        
        let config = self.config.clone();
        let scanners = self.scanners.clone();
        let results = self.results.clone();
        let is_scanning_thread = self.is_scanning.clone();

        thread::spawn(move || {
            let mut all = Vec::new();
            for scanner in scanners.iter() {
                all.extend(scanner.scan(&config.project_roots));
            }
            all.sort_by(|a, b| a.category.cmp(&b.category).then_with(|| b.size_bytes.cmp(&a.size_bytes)));
            
            let mut res_guard = results.lock().unwrap();
            *res_guard = all;
            
            let mut scanning_guard2 = is_scanning_thread.lock().unwrap();
            *scanning_guard2 = false;
        });
        self.message = "Scanning system... This may take a minute depending on your storage.".to_string();
        self.selected.clear();
    }

    fn execute_cleaning(&mut self) {
        let mut results = self.results.lock().unwrap();
        let mut to_clean = Vec::new();
        
        for r in results.iter() {
            if self.selected.contains(&r.id) {
                to_clean.push(r.clone());
            }
        }
        
        let mut freed = 0;
        let mut refs = Vec::new();
        for target in &to_clean {
            if executor::execute_action(target, false).is_ok() {
                freed += target.size_bytes;
                refs.push(target);
            }
        }
        if freed > 0 {
            report::generate_report(freed, &refs, false);
            self.message = format!("Success! Freed {} of space.", crate::utils::format_size(freed));
            
            // Remove cleaned items from UI
            results.retain(|r| !self.selected.contains(&r.id));
            self.selected.clear();
        } else {
            self.message = "Failed to clean or nothing selected.".to_string();
        }
    }
}

impl eframe::App for MacDevCleanApp {
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Not used, we override update directly.
    }

    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_scanning = *self.is_scanning.lock().unwrap();
        if is_scanning {
            ctx.request_repaint(); // Keep repainting to show progress
        }

        // --- Top Panel ---
        egui::Panel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("💻 MacDevClean");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("ℹ️ About").clicked() {
                        self.show_about = true;
                    }
                    
                    let clean_enabled = !is_scanning && !self.selected.is_empty();
                    if ui.add_enabled(clean_enabled, egui::Button::new("🗑 Clean Selected")).clicked() {
                        self.show_confirm_clean = true;
                    }

                    if ui.button("✅ Select All Safe").clicked() {
                        let results = self.results.lock().unwrap();
                        for r in results.iter() {
                            if matches!(r.risk, crate::scanner::RiskLevel::Safe) {
                                self.selected.insert(r.id.clone());
                            }
                        }
                        self.message = "All Safe items selected.".to_string();
                    }
                    
                    if ui.add_enabled(!is_scanning, egui::Button::new("🔍 Scan System")).clicked() {
                        self.trigger_scan();
                    }
                    
                    if is_scanning {
                        ui.spinner();
                    }
                });
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(&self.message);
            });
            ui.add_space(8.0);
        });

        // --- Central Panel (List) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.search_query);
                if ui.button("Clear").clicked() {
                    self.search_query.clear();
                }
            });
            ui.separator();

            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                let results = self.results.lock().unwrap();
                
                if results.is_empty() {
                    if is_scanning {
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.spinner();
                            ui.label("Scanning your system...");
                        });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.label("No caches found. Click 'Scan System' to begin.");
                        });
                    }
                } else {
                    let mut grouped: std::collections::HashMap<String, Vec<&crate::scanner::ScanResult>> = std::collections::HashMap::new();
                    
                    for res in results.iter() {
                        let matches_search = self.search_query.is_empty() || 
                            res.name.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                            res.path.to_string_lossy().to_lowercase().contains(&self.search_query.to_lowercase());
                            
                        if matches_search {
                            grouped.entry(res.category.clone()).or_default().push(res);
                        }
                    }

                    let mut groups: Vec<(String, Vec<&crate::scanner::ScanResult>, u64)> = grouped
                        .into_iter()
                        .map(|(category, items)| {
                            let total_size: u64 = items.iter().map(|r| r.size_bytes).sum();
                            (category, items, total_size)
                        })
                        .collect();

                    groups.sort_by(|a, b| b.2.cmp(&a.2));

                    for (category, items, total_size) in groups {
                        let all_selected = items.iter().all(|r| self.selected.contains(&r.id));
                        
                        let header_text = format!("📂 {} ({} items, {})", category.to_uppercase(), items.len(), crate::utils::format_size(total_size));
                        
                        egui::CollapsingHeader::new(egui::RichText::new(header_text).strong().size(16.0))
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let mut check = all_selected;
                                    if ui.checkbox(&mut check, "Select All in Category").changed() {
                                        for r in &items {
                                            if matches!(r.risk, crate::scanner::RiskLevel::Forbidden) {
                                                continue;
                                            }
                                            if check {
                                                self.selected.insert(r.id.clone());
                                            } else {
                                                self.selected.remove(&r.id);
                                            }
                                        }
                                    }
                                });
                                ui.separator();
                                
                                for res in items {
                                    let is_forbidden = matches!(res.risk, crate::scanner::RiskLevel::Forbidden);
                                    let mut is_checked = self.selected.contains(&res.id);
                                    
                                    let bg_color = if is_checked {
                                        egui::Color32::from_rgba_unmultiplied(10, 120, 250, 30)
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };

                                    egui::Frame::none()
                                        .fill(bg_color)
                                        .rounding(4.0)
                                        .inner_margin(egui::Margin::symmetric(6, 3))
                                        .show(ui, |ui| {
                                            ui.set_min_width(ui.available_width());
                                            
                                            ui.horizontal(|ui| {
                                                let mut cb_changed = false;
                                                if is_forbidden {
                                                    ui.add_enabled(false, egui::Checkbox::new(&mut false, ""));
                                                } else {
                                                    if ui.checkbox(&mut is_checked, "").changed() {
                                                        cb_changed = true;
                                                    }
                                                }

                                                let rest_response = ui.horizontal(|ui| {
                                                    let (risk_str, risk_color) = match res.risk {
                                                        crate::scanner::RiskLevel::Safe => ("Safe", egui::Color32::from_rgb(46, 204, 113)),      // High-contrast Emerald Green
                                                        crate::scanner::RiskLevel::Confirm => ("Confirm", egui::Color32::from_rgb(241, 196, 15)),  // High-contrast Gold/Yellow
                                                        crate::scanner::RiskLevel::Dangerous => ("Dangerous", egui::Color32::from_rgb(231, 76, 60)), // High-contrast Alizarin Red
                                                        crate::scanner::RiskLevel::Forbidden => ("Forbidden", egui::Color32::from_rgb(189, 195, 199)), // High-contrast Silver Gray
                                                    };
                                                    
                                                    ui.colored_label(risk_color, format!("[{}]", risk_str));
                                                    
                                                    ui.label(egui::RichText::new(format!("{:>10}", crate::utils::format_size(res.size_bytes))).monospace().strong());
                                                    
                                                    ui.label(egui::RichText::new(&res.name).strong());
                                                    
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        let path_str = res.path.display().to_string();
                                                        let display_path = if path_str.ends_with(&res.name) {
                                                            res.path.parent().map(|p| p.display().to_string()).unwrap_or(path_str)
                                                        } else {
                                                            path_str
                                                        };
                                                        
                                                        ui.label(egui::RichText::new(display_path).color(egui::Color32::from_rgb(170, 185, 200)))
                                                            .on_hover_text(&res.reason);
                                                    });
                                                }).response;

                                                let mut clickable_rect = rest_response.rect;
                                                clickable_rect.max.x = ui.max_rect().max.x;
                                                
                                                let rest_interact = ui.interact(
                                                    clickable_rect,
                                                    rest_response.id.with("rest_click"),
                                                    egui::Sense::click()
                                                );

                                                if rest_interact.clicked() && !is_forbidden {
                                                    is_checked = !is_checked;
                                                    cb_changed = true;
                                                }

                                                if cb_changed {
                                                    if is_checked {
                                                        self.selected.insert(res.id.clone());
                                                    } else {
                                                        self.selected.remove(&res.id);
                                                    }
                                                }

                                                rest_interact.context_menu(|ui| {
                                                    let path_str = res.path.to_string_lossy().to_string();
                                                    if ui.button("📋 复制完整路径 (Copy Full Path)").clicked() {
                                                        ui.ctx().copy_text(path_str);
                                                        ui.close_menu();
                                                    }
                                                    if ui.button("📂 在访达中显示 (Reveal in Finder)").clicked() {
                                                        reveal_in_finder(&res.path);
                                                        ui.close_menu();
                                                    }
                                                });
                                            });
                                        });
                                }
                                ui.add_space(10.0);
                            });
                    }
                }
            });
        });
        
        // --- About Window ---
        if self.show_about {
            egui::Window::new("About MacDevClean")
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("MacDevClean");
                        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                        ui.add_space(10.0);
                        ui.label("The safest and fastest cache cleaner for macOS developers.");
                        ui.add_space(5.0);
                        ui.label("Features:");
                        ui.label("✅ Trash-First Security Policy");
                        ui.label("✅ Fine-Grained Guard Lists");
                        ui.label("✅ Detects HuggingFace, Docker, npm, Node, Python, etc.");
                        ui.label("✅ Safe Chat App Cleanup (WeChat, WeCom, DingTalk, Feishu)");
                        ui.add_space(10.0);
                        ui.hyperlink_to("⭐️ Star on GitHub", "https://github.com/aning35/mac-dev-clean");
                        ui.add_space(10.0);
                    });
                });
        }

        // --- Confirm Clean Window ---
        if self.show_confirm_clean {
            let mut results_len = 0;
            let mut total_bytes = 0;
            if let Ok(results) = self.results.lock() {
                for r in results.iter() {
                    if self.selected.contains(&r.id) {
                        results_len += 1;
                        total_bytes += r.size_bytes;
                    }
                }
            }

            egui::Window::new("Confirm Deletion (确认清理)")
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("⚠️ Warning (警告)").strong().color(egui::Color32::from_rgb(231, 76, 60)).size(18.0));
                        ui.add_space(10.0);
                        ui.label(format!("Are you sure you want to clean the {} selected caches?", results_len));
                        ui.label(format!("This will move {} of files to the Trash.", crate::utils::format_size(total_bytes)));
                        ui.add_space(15.0);
                        
                        ui.horizontal(|ui| {
                            let button_width = 120.0;
                            let spacing = 12.0;
                            let total_width = (button_width * 2.0) + spacing;
                            let padding = (ui.available_width() - total_width) / 2.0;
                            if padding > 0.0 {
                                ui.add_space(padding);
                            }

                            if ui.add_sized([button_width, 24.0], egui::Button::new("❌ Cancel (取消)")).clicked() {
                                self.show_confirm_clean = false;
                            }
                            if ui.add_sized([button_width, 24.0], egui::Button::new(egui::RichText::new("🗑 Yes, Clean (确认删除)").color(egui::Color32::from_rgb(231, 76, 60)))).clicked() {
                                self.show_confirm_clean = false;
                                self.execute_cleaning();
                            }
                        });
                        ui.add_space(8.0);
                    });
                });
        }
    }
}

pub fn start_gui(config: Config, scanners: Arc<Vec<Box<dyn Scanner + Send + Sync>>>) {
    let icon_data = if let Ok(image) = image::load_from_memory(include_bytes!("../docs/logo_512.png")) {
        let rgba = image.into_rgba8();
        let (width, height) = rgba.dimensions();
        egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }
    } else {
        egui::IconData::default()
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(icon_data),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "MacDevClean",
        options,
        Box::new(|cc| Ok(Box::new(MacDevCleanApp::new(cc, config, scanners)))),
    ) {
        eprintln!("eframe error: {:?}", e);
    }
}

fn reveal_in_finder(path: &std::path::Path) {
    let _ = std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn();
}

