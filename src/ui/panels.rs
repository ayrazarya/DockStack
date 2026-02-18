use egui::{self, Color32, RichText, ScrollArea, Vec2, Stroke, Rect};
use crate::config::AppConfig;
use crate::docker::manager::{ContainerInfo, ServiceStatus};
use crate::monitor::{ContainerStats, SystemStats};
use crate::services::{get_service_registry, ServiceCategory};
use crate::port_scanner::PortInfo;
use crate::ui::theme::*;
use crate::ui::widgets::*;
use crate::utils;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Dashboard,
    Services,
    Containers,
    Logs,
    Terminal,
    Ports,
    Monitor,
    Settings,
}

/// Render the sidebar
pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: &mut Tab,
    config: &mut AppConfig,
    status: &ServiceStatus,
) {
    let width = ui.available_width();
    
    // Logo Area
    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new("⚡").size(32.0).color(COLOR_PRIMARY));
        ui.add_space(4.0);
        ui.label(RichText::new("DockStack").size(20.0).strong().color(COLOR_TEXT));
        ui.label(RichText::new("Premium Dev Env").size(10.0).color(COLOR_TEXT_DIM));
    });
    ui.add_space(24.0);

    // Project Selector (Modern Pill Style)
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let project_name = config.active_project().map(|p| p.name.clone()).unwrap_or("Select Project".to_string());
        
        egui::ComboBox::from_id_salt("sidebar_project_selector")
            .selected_text(RichText::new(format!("📂  {}", project_name)).strong().color(COLOR_TEXT))
            .width(width - 24.0)
            .show_ui(ui, |ui| {
                let project_names: Vec<String> = config.projects.iter().map(|p| p.name.clone()).collect();
                let current_id = config.active_project_id.clone();
                
                for (_i, name) in project_names.iter().enumerate() {
                    let id = config.projects.iter().find(|p| p.name == *name).map(|p| p.id.clone()).unwrap();
                    let is_selected = Some(&id) == current_id.as_ref();
                    
                    if ui.selectable_label(is_selected, name).clicked() {
                        config.active_project_id = Some(id);
                        config.save();
                    }
                }
            });
    });
    ui.add_space(24.0);

    // Navigation Menu
    let tabs = vec![
        (Tab::Dashboard, "🏠", "Overview"),
        (Tab::Services, "📦", "Stack"),
        (Tab::Containers, "🐳", "Containers"),
        (Tab::Logs, "📋", "Logs"),
        (Tab::Terminal, "💻", "Console"),
        (Tab::Ports, "🔌", "Port Check"),
        (Tab::Monitor, "📊", "Metrics"),
        (Tab::Settings, "⚙️", "Settings"),
    ];
    
    ui.label(RichText::new("MAIN MENU").size(10.0).color(COLOR_TEXT_MUTED).strong());
    ui.add_space(8.0);

    for (tab, icon, label) in tabs {
        let is_active = *active_tab == tab;
        
        let (bg, text_col) = if is_active {
            (COLOR_SIDEBAR_ACTIVE, COLOR_PRIMARY)
        } else {
            (Color32::TRANSPARENT, COLOR_TEXT_DIM)
        };

        let btn = egui::Button::new(
            RichText::new(format!("  {}  {}", icon, label))
                .size(13.0)
                .color(text_col)
                .strong(),
        )
        .fill(bg)
        .stroke(egui::Stroke::NONE)
        .corner_radius(egui::CornerRadius::same(6))
        .min_size(Vec2::new(width - 20.0, 38.0))
        .frame(true);

        if ui.add(btn).clicked() {
            *active_tab = tab;
        }
        ui.add_space(2.0);
    }

    // Bottom Status
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.add_space(16.0);
        
        // Status Pill
        let (status_text, status_col) = match status {
            ServiceStatus::Running => ("RUNNING", COLOR_SUCCESS),
            ServiceStatus::Starting => ("STARTING", COLOR_WARNING),
            ServiceStatus::Stopping => ("STOPPING", COLOR_WARNING),
            ServiceStatus::Stopped => ("STOPPED", COLOR_TEXT_MUTED),
            ServiceStatus::Error(_) => ("ERROR", COLOR_ERROR),
        };

        egui::Frame::new()
            .fill(COLOR_BG_CARD)
            .corner_radius(egui::CornerRadius::same(12))
            .stroke(Stroke::new(1.0, COLOR_BORDER))
            .inner_margin(8)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").size(10.0).color(status_col));
                    ui.label(RichText::new(status_text).size(10.0).strong().color(status_col));
                });
            });
            
        if let ServiceStatus::Error(_) = status {
             if ui.link(RichText::new("View Error").size(10.0).color(COLOR_ERROR)).clicked() {
                 *active_tab = Tab::Logs;
             }
        }
    });
}

/// Render the dashboard panel
pub fn render_dashboard(
    ui: &mut egui::Ui,
    config: &AppConfig,
    _status: &ServiceStatus,
    sys_stats: &SystemStats,
    containers: &[ContainerInfo],
    docker_available: bool,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(10.0);
        
        // Modern Header
        ui.horizontal(|ui| {
             ui.vertical(|ui| {
                 ui.heading(RichText::new("Dashboard").size(28.0).color(COLOR_TEXT).strong());
                 ui.label(RichText::new("Overview of your local environment").size(14.0).color(COLOR_TEXT_DIM));
             });
        });
        ui.add_space(24.0);

        if !docker_available {
            ui.scope(|ui| {
                ui.style_mut().visuals.extreme_bg_color = COLOR_ERROR.gamma_multiply(0.1);
                egui::Frame::group(ui.style())
                    .stroke(Stroke::new(1.0, COLOR_ERROR))
                    .show(ui, |ui| {
                         ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠").size(24.0).color(COLOR_ERROR));
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Docker Not Found").strong().color(COLOR_ERROR));
                                ui.label(RichText::new("Please start Docker Desktop/Engine to use DockStack.").color(COLOR_TEXT));
                            });
                         });
                    });
            });
            return;
        }

        // Stats Grid
        egui::Grid::new("dash_stats")
            .spacing(Vec2::new(16.0, 16.0))
            .min_col_width(180.0)
            .show(ui, |ui| {
                stat_card(ui, "CPU Load", &format!("{:.0}%", sys_stats.cpu_usage), "📈", COLOR_PRIMARY);
                stat_card(ui, "Memory", &format!("{:.1} GB", sys_stats.memory_used as f64 / 1024.0 / 1024.0 / 1024.0), "💾", COLOR_SECONDARY);
                stat_card(ui, "Containers", &format!("{}", containers.len()), "📦", COLOR_ACCENT);
                if let Some(project) = config.active_project() {
                    let enabled = project.services.values().filter(|v| v.enabled).count();
                    stat_card(ui, "Active Services", &format!("{}", enabled), "⚡", COLOR_SUCCESS);
                }
                ui.end_row();
            });

        ui.add_space(32.0);

        // Active Stack Section
        ui.label(RichText::new("Details").size(16.0).strong().color(COLOR_TEXT));
        ui.add_space(12.0);

        if let Some(project) = config.active_project() {
            let enabled_services: Vec<_> = project.services.iter().filter(|(_, v)| v.enabled).collect();
            
            if enabled_services.is_empty() {
                ui.label("No services enabled. Enable them in the Stack tab.");
            } else {
                 egui::Grid::new("dash_services")
                    .num_columns(2)
                    .spacing(Vec2::new(16.0, 16.0))
                    .show(ui, |ui| {
                        for (i, (name, svc)) in enabled_services.iter().enumerate() {
                            let info = crate::services::get_service_info(name);
                            let display_name = info.as_ref().map(|i| i.display_name.clone()).unwrap_or(name.to_string());
                            let icon = info.as_ref().map(|i| i.icon).unwrap_or("❓");
                            
                            let is_running = containers.iter().any(|c| c.name.contains(name.as_str()) && c.state.contains("running"));
                            
                            service_card_compact(ui, &display_name, &icon, &svc.version, svc.port, is_running);
                            
                            if (i + 1) % 2 == 0 {
                                ui.end_row();
                            }
                        }
                        if enabled_services.len() % 2 != 0 {
                            ui.end_row();
                        }
                    });
            }
        }
        
        ui.add_space(32.0);
        
        // Quick Actions
         ui.label(RichText::new("Shortcuts").size(16.0).strong().color(COLOR_TEXT));
         ui.add_space(12.0);
         ui.horizontal(|ui| {
             if let Some(project) = config.active_project() {
                if ui.add(egui::Button::new(RichText::new("🌐 Open Localhost").size(14.0)).frame(true).min_size(Vec2::new(140.0, 40.0))).clicked() {
                     let port = project.services.get("nginx").map(|s| s.port).or_else(|| project.services.get("apache").map(|s| s.port)).unwrap_or(80);
                     utils::open_url(&format!("http://localhost:{}", port));
                }
                if ui.add(egui::Button::new(RichText::new("📂 Open Project Folder").size(14.0)).frame(true).min_size(Vec2::new(160.0, 40.0))).clicked() {
                     utils::open_directory(&project.directory);
                }
             }
         });
    });
}

fn stat_card(ui: &mut egui::Ui, title: &str, value: &str, icon: &str, accent: Color32) {
    egui::Frame::new()
        .fill(COLOR_BG_CARD)
        .corner_radius(egui::CornerRadius::same(12))
        .stroke(Stroke::new(1.0, COLOR_BORDER))
        .inner_margin(16.0)
        .show(ui, |ui| {
             ui.set_min_width(180.0);
             ui.horizontal(|ui| {
                 ui.label(RichText::new(icon).size(24.0));
                 ui.add_space(8.0);
                 ui.vertical(|ui| {
                     ui.label(RichText::new(title).size(12.0).color(COLOR_TEXT_DIM));
                     ui.label(RichText::new(value).size(20.0).strong().color(COLOR_TEXT));
                 });
             });
             // Little accent bar
             let rect = ui.min_rect();
             ui.painter().rect_filled(
                 Rect::from_min_size(rect.left_bottom() + Vec2::new(16.0, -2.0), Vec2::new(rect.width() - 32.0, 2.0)),
                 egui::CornerRadius::same(2),
                 accent
             );
        });
}

fn service_card_compact(ui: &mut egui::Ui, name: &str, icon: &str, version: &str, port: u16, running: bool) {
    egui::Frame::new()
        .fill(COLOR_BG_CARD)
        .corner_radius(egui::CornerRadius::same(12))
        .stroke(Stroke::new(1.0, if running { COLOR_BORDER_LIGHT } else { COLOR_BORDER }))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(24.0));
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(name).size(16.0).strong().color(COLOR_TEXT));
                        if running {
                             ui.label(RichText::new("●").size(8.0).color(COLOR_SUCCESS));
                        }
                    });
                    ui.label(RichText::new(format!("v{} • Port {}", version, port)).size(12.0).color(COLOR_TEXT_DIM));
                });
            });
        });
}


/// Render the services panel (Stack)
pub fn render_services(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    containers: &[ContainerInfo],
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(10.0);
        ui.heading(RichText::new("Service Stack").size(28.0).color(COLOR_TEXT).strong());
        ui.label(RichText::new("Configure your development stack").size(14.0).color(COLOR_TEXT_DIM));
        ui.add_space(24.0);

        let registry = get_service_registry();
        let categories = vec![
            ServiceCategory::WebServer,
            ServiceCategory::Database,
            ServiceCategory::Runtime,
            ServiceCategory::Cache,
            ServiceCategory::Admin,
        ];

        for category in categories {
            let services_in_cat: Vec<_> = registry.iter().filter(|s| s.category == category).collect();
            if services_in_cat.is_empty() { continue; }

            ui.label(RichText::new(category.label()).size(14.0).strong().color(COLOR_ACCENT));
            ui.add_space(8.0);

            for svc_info in services_in_cat {
                if let Some(project) = config.active_project_mut() {
                    if let Some(svc) = project.services.get_mut(&svc_info.name) {
                         let is_running = containers.iter().any(|c| c.name.contains(&svc_info.name) && c.state.contains("running"));
                         
                         egui::Frame::new()
                            .fill(COLOR_BG_CARD)
                            .corner_radius(egui::CornerRadius::same(12))
                            .stroke(Stroke::new(1.0, COLOR_BORDER))
                            .inner_margin(16.0)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    // Status & Icon
                                    ui.label(RichText::new(svc_info.icon).size(28.0));
                                    ui.add_space(12.0);
                                    
                                    // Info
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&svc_info.display_name).size(16.0).strong().color(COLOR_TEXT));
                                            if is_running {
                                                ui.label(RichText::new("● RUNNING").size(9.0).color(COLOR_SUCCESS).strong());
                                            }
                                        });
                                        ui.label(RichText::new(&svc_info.description).size(12.0).color(COLOR_TEXT_DIM));
                                    });
                                    
                                    // Controls (Right aligned)
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                         // Toggle
                                        let mut enabled = svc.enabled;
                                        if toggle_switch(ui, &mut enabled).changed() {
                                            svc.enabled = enabled;
                                            if svc_info.name == "ssl" { project.ssl_enabled = enabled; }
                                        }
                                        
                                        ui.add_space(16.0);
                                        
                                        // Config actions
                                        ui.menu_button("⚙ Config", |ui| {
                                             if ui.button("Edit Environment Vars").clicked() {
                                                 // Todo: Expand logic
                                             }
                                             
                                             let config_path = match svc_info.name.as_str() {
                                                "nginx" => Some(std::path::Path::new(&project.directory).join("nginx/default.conf")),
                                                "apache" => Some(std::path::Path::new(&project.directory).join("apache/httpd.conf")),
                                                "php" => Some(std::path::Path::new(&project.directory).join("php/php.ini")),
                                                "mysql" => Some(std::path::Path::new(&project.directory).join("mysql/my.cnf")),
                                                _ => None,
                                            };
                                            if let Some(path) = config_path {
                                                if ui.button("Open Config File").clicked() {
                                                     if !path.exists() {
                                                        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).ok(); }
                                                        std::fs::write(&path, "# Config file\n").ok();
                                                     }
                                                     crate::utils::open_url(&path.to_string_lossy());
                                                     ui.close_menu();
                                                }
                                            }
                                        });
                                        
                                        ui.label(RichText::new(format!("Port: {}", svc.port)).size(12.0).color(COLOR_TEXT_MUTED));
                                    });
                                });
                                
                                // Expandable Environment Variables (Simplified for now)
                                if svc.enabled {
                                     ui.add_space(8.0);
                                     ui.collapsing("Advanced Settings", |ui| {
                                         // Reuse existing env var grid logic but cleaner
                                          let mut vars: Vec<(String, String)> = svc.env_vars.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                                          vars.sort_by(|a, b| a.0.cmp(&b.0));
                                          let mut changed = false;
                                          
                                          egui::Grid::new(format!("env_{}", svc_info.name)).spacing(Vec2::new(8.0, 8.0)).show(ui, |ui| {
                                              for (i, (key, val)) in vars.iter_mut().enumerate() {
                                                  ui.push_id(i, |ui| {
                                                      if ui.add(egui::TextEdit::singleline(key).desired_width(140.0)).changed() { changed = true; }
                                                  });
                                                  ui.push_id(i+1000, |ui| {
                                                       if ui.add(egui::TextEdit::singleline(val).desired_width(200.0)).changed() { changed = true; }
                                                  });
                                                  ui.end_row();
                                              }
                                          });
                                          
                                          if changed {
                                              svc.env_vars = vars.into_iter().collect();
                                          }
                                     });
                                }
                            });
                         ui.add_space(12.0);
                    }
                }
            }
            ui.add_space(12.0);
        }
    });
}

pub fn render_containers(ui: &mut egui::Ui, containers: &[ContainerInfo]) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(10.0);
        ui.heading(RichText::new("Containers").size(28.0).color(COLOR_TEXT).strong());
        ui.label(RichText::new("Active Docker Containers").size(14.0).color(COLOR_TEXT_DIM));
        ui.add_space(24.0);

        if containers.is_empty() {
             ui.label(RichText::new("No containers found.").color(COLOR_TEXT_MUTED));
        } else {
             egui::Grid::new("container_list")
                .striped(true)
                .spacing(Vec2::new(20.0, 12.0))
                .min_row_height(32.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("NAME").size(12.0).strong().color(COLOR_TEXT_MUTED));
                    ui.label(RichText::new("IMAGE").size(12.0).strong().color(COLOR_TEXT_MUTED));
                    ui.label(RichText::new("STATE").size(12.0).strong().color(COLOR_TEXT_MUTED));
                    ui.label(RichText::new("PORTS").size(12.0).strong().color(COLOR_TEXT_MUTED));
                    ui.end_row();
                    
                    for c in containers {
                        let running = c.state.contains("running");
                        ui.horizontal(|ui| {
                             ui.label(RichText::new(if running { "●" } else { "○" }).size(10.0).color(if running { COLOR_SUCCESS } else { COLOR_TEXT_MUTED }));
                             ui.label(RichText::new(&c.name).size(13.0).color(COLOR_TEXT));
                        });
                        ui.label(RichText::new(&c.image).size(13.0).color(COLOR_ACCENT));
                        ui.label(RichText::new(&c.state).size(13.0).color(if running { COLOR_SUCCESS } else { COLOR_TEXT_DIM }));
                        ui.label(RichText::new(utils::truncate_string(&c.ports, 50)).size(11.0).color(COLOR_TEXT_DIM));
                        ui.end_row();
                    }
                });
        }
    });
}

pub fn render_logs(ui: &mut egui::Ui, logs: &[String], clear_logs: &mut bool) {
    ui.add_space(10.0);
    ui.horizontal(|ui| {
         ui.heading(RichText::new("Logs").size(28.0).color(COLOR_TEXT).strong());
         ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
              if ui.button(RichText::new("🗑 Clear Output").size(12.0)).clicked() {
                  *clear_logs = true;
              }
         });
    });
    ui.add_space(16.0);
    
    egui::Frame::new()
        .fill(COLOR_BG_APP) 
        .stroke(Stroke::new(1.0, COLOR_BORDER))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(12.0)
        .show(ui, |ui| {
             ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                     ui.set_min_width(ui.available_width());
                     for line in logs {
                         let color = if line.contains("ERROR") { COLOR_ERROR } 
                                     else if line.contains("WARN") { COLOR_WARNING }
                                     else if line.starts_with("[DockStack]") { COLOR_PRIMARY }
                                     else { COLOR_TEXT_DIM };
                         
                         ui.label(RichText::new(line).size(12.0).family(egui::FontFamily::Monospace).color(color));
                     }
                });
        });
}

pub fn render_terminal(
    ui: &mut egui::Ui,
    output_lines: &[String],
    input_buffer: &mut String,
    send_input: &mut bool,
    clear_terminal: &mut bool,
    start_terminal: &mut bool,
    terminal_running: bool,
) {
    ui.add_space(10.0);
     ui.horizontal(|ui| {
         ui.heading(RichText::new("Terminal").size(28.0).color(COLOR_TEXT).strong());
         ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
              if !terminal_running {
                  if ui.button(RichText::new("▶ Start Shell").color(COLOR_SUCCESS)).clicked() { *start_terminal = true; }
              } else {
                  if ui.button(RichText::new("⏹ Reset").color(COLOR_ERROR)).clicked() { /* logic to kill */ }
              }
              if ui.button("Clear").clicked() { *clear_terminal = true; }
         });
    });
    ui.add_space(16.0);
    
    egui::Frame::new()
        .fill(COLOR_BG_APP)
        .stroke(Stroke::new(1.0, COLOR_BORDER))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(12.0)
        .show(ui, |ui| {
            // Output area
             ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(ui.available_height() - 40.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                     ui.set_min_width(ui.available_width());
                     for line in output_lines {
                         let col = if line.starts_with("$") { COLOR_PRIMARY } else { COLOR_TEXT_DIM };
                         ui.label(RichText::new(line).size(12.0).family(egui::FontFamily::Monospace).color(col));
                     }
                });
             
             ui.separator();
             
             // Input area
             ui.horizontal(|ui| {
                 ui.label(RichText::new("❯").color(COLOR_SUCCESS).strong());
                 let response = ui.add(egui::TextEdit::singleline(input_buffer)
                    .frame(false)
                    .desired_width(ui.available_width())
                    .font(egui::FontId::monospace(13.0))
                    .text_color(COLOR_TEXT));
                 
                 if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                     *send_input = true;
                     response.request_focus();
                 }
             });
        });
}

pub fn render_ports(ui: &mut egui::Ui, port_infos: &[PortInfo], scan_ports: &mut bool) {
     ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
             ui.heading(RichText::new("Port Check").size(28.0).color(COLOR_TEXT).strong());
             ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                  if ui.button("🔄 Rescan").clicked() { *scan_ports = true; }
             });
        });
        ui.label(RichText::new("Detect conflicts before starting your services").size(14.0).color(COLOR_TEXT_DIM));
        ui.add_space(24.0);

        egui::Grid::new("port_grid").spacing(Vec2::new(32.0, 12.0)).striped(true).show(ui, |ui| {
             ui.label(RichText::new("PORT").strong().color(COLOR_TEXT_MUTED));
             ui.label(RichText::new("STATUS").strong().color(COLOR_TEXT_MUTED));
             ui.label(RichText::new("PROCESS").strong().color(COLOR_TEXT_MUTED));
             ui.label(RichText::new("ACTION").strong().color(COLOR_TEXT_MUTED));
             ui.end_row();

             for info in port_infos {
                 ui.label(RichText::new(format!("{}", info.port)).size(14.0).strong().color(COLOR_TEXT));
                 if info.in_use {
                     ui.label(RichText::new("BUSY").size(12.0).color(COLOR_ERROR));
                     ui.label(RichText::new(&info.process).size(12.0).color(COLOR_TEXT_DIM));
                     ui.label(RichText::new("Kill / Change Port").size(12.0).color(COLOR_WARNING));
                 } else {
                     ui.label(RichText::new("FREE").size(12.0).color(COLOR_SUCCESS));
                     ui.label("-");
                     ui.label("-");
                 }
                 ui.end_row();
             }
        });
     });
}

pub fn render_monitor(
    ui: &mut egui::Ui,
    _sys_stats: &SystemStats,
    container_stats: &[ContainerStats],
    cpu_history: &[f32],
    mem_history: &[f32],
) {
    ScrollArea::vertical().show(ui, |ui| {
         ui.add_space(10.0);
         ui.heading(RichText::new("Live Monitor").size(28.0).color(COLOR_TEXT).strong());
         ui.add_space(24.0);
         
         ui.horizontal(|ui| {
            card_frame(ui, |ui| {
                 ui.set_min_width(300.0);
                 ui.label(RichText::new("CPU History").size(14.0).color(COLOR_TEXT_DIM));
                 sparkline(ui, cpu_history, 120.0, COLOR_PRIMARY, Vec2::new(280.0, 80.0));
            });
            card_frame(ui, |ui| {
                 ui.set_min_width(300.0);
                 ui.label(RichText::new("Memory History").size(14.0).color(COLOR_TEXT_DIM));
                 sparkline(ui, mem_history, 120.0, COLOR_SECONDARY, Vec2::new(280.0, 80.0));
            });
         });
         
         ui.add_space(24.0);
         
         if !container_stats.is_empty() {
             ui.label(RichText::new("Container Live Usage").size(16.0).strong());
             ui.add_space(12.0);
             egui::Grid::new("monitor_grid").striped(true).spacing(Vec2::new(24.0, 12.0)).show(ui, |ui| {
                 ui.label(RichText::new("NAME").strong().color(COLOR_TEXT_MUTED));
                 ui.label(RichText::new("CPU").strong().color(COLOR_TEXT_MUTED));
                 ui.label(RichText::new("MEM").strong().color(COLOR_TEXT_MUTED));
                 ui.end_row();
                 
                 for s in container_stats {
                     ui.label(RichText::new(&s.name).color(COLOR_TEXT));
                     ui.label(RichText::new(&s.cpu_percent).color(COLOR_PRIMARY));
                     ui.label(RichText::new(&s.mem_usage).color(COLOR_SECONDARY));
                     ui.end_row();
                 }
             });
         }
    });
}
pub fn render_settings(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    new_project_name: &mut String,
    gen_ssl: &mut bool,
    rem_ssl: &mut bool,
) {
     ScrollArea::vertical().show(ui, |ui| {
         ui.add_space(10.0);
         ui.heading(RichText::new("Settings").size(28.0).color(COLOR_TEXT).strong());
         ui.add_space(24.0);
         
         card_frame(ui, |ui| {
             ui.label(RichText::new("Projects").size(16.0).strong());
             ui.separator();
             ui.horizontal(|ui| {
                 ui.label("New Project Name:");
                 ui.text_edit_singleline(new_project_name);
                 if ui.button("Create").clicked() {
                     if !new_project_name.is_empty() {
                         // Logic handled in parent or here
                     }
                 }
             });
         });

         ui.add_space(16.0);

         card_frame(ui, |ui| {
             ui.label(RichText::new("SSL / HTTPS").size(16.0).strong());
             ui.separator();
             ui.label(RichText::new("Generate locally trusted certificates for your development domains.").color(COLOR_TEXT_DIM));
             ui.add_space(8.0);
             ui.horizontal(|ui| {
                 if ui.button("Generate Certs").clicked() { *gen_ssl = true; }
                 if ui.button("Remove Certs").clicked() { *rem_ssl = true; }
             });
         });
     });
}
