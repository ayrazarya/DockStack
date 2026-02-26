use egui::{self, Color32, RichText, ScrollArea, Vec2, Stroke, Rect, StrokeKind};
use std::collections::HashMap;
use crate::config::{AppConfig, ServiceConfig};
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
    
    // Brand Area
    ui.add_space(32.0);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(40.0, 40.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, egui::CornerRadius::same(10), COLOR_PRIMARY);
        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "⚡", egui::FontId::proportional(24.0), COLOR_BG_APP);
        
        ui.add_space(12.0);
        ui.vertical(|ui| {
            ui.label(RichText::new("DockStack").size(18.0).strong().color(COLOR_TEXT));
            ui.label(RichText::new("v0.1.0-alpha").size(10.0).color(COLOR_TEXT_MUTED));
        });
    });
    ui.add_space(32.0);

    // Project Context
    ui.label(RichText::new("WORKSPACE").size(10.0).color(COLOR_TEXT_MUTED).strong());
    ui.add_space(8.0);
    
    egui::Frame::new()
        .fill(COLOR_BG_CARD.gamma_multiply(0.5))
        .corner_radius(egui::CornerRadius::same(10))
        .stroke(Stroke::new(1.0, COLOR_BORDER))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_width(width);
            let project_name = config.active_project().map(|p| p.name.clone()).unwrap_or("Select Project".to_string());
            
            ui.menu_button(RichText::new(format!("📂 {}", project_name)).strong().color(COLOR_TEXT), |ui| {
                for project in &config.projects {
                    if ui.selectable_label(config.active_project_id.as_ref() == Some(&project.id), &project.name).clicked() {
                        config.active_project_id = Some(project.id.clone());
                        config.save();
                        ui.close_menu();
                    }
                }
                ui.separator();
                if ui.button("➕ Create New Project").clicked() {
                    config.add_project("New Project".to_string());
                    ui.close_menu();
                }
                if ui.button("📥 Import docker-compose.yml").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Docker Compose", &["yml", "yaml"])
                        .pick_file() {
                            let _ = config.import_from_compose(&path);
                    }
                    ui.close_menu();
                }
            });
        });
    
    ui.add_space(32.0);

    // Navigation Menu
    ui.label(RichText::new("NAVIGATION").size(10.0).color(COLOR_TEXT_MUTED).strong());
    ui.add_space(8.0);

    let tabs = vec![
        (Tab::Dashboard, "🏠", "Overview"),
        (Tab::Services, "📦", "Service Stack"),
        (Tab::Containers, "🐳", "Containers"),
        (Tab::Logs, "📋", "System Logs"),
        (Tab::Terminal, "💻", "Terminal"),
        (Tab::Ports, "🔌", "Port Checker"),
        (Tab::Monitor, "📊", "Real-time Metrics"),
        (Tab::Settings, "⚙", "Preferences"),
    ];

    for (tab, icon, label) in tabs {
        let is_active = *active_tab == tab;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width - 12.0, 40.0), egui::Sense::click());
        
        if response.clicked() {
            *active_tab = tab;
        }

        if ui.is_rect_visible(rect) {
            let (bg, text_col) = if is_active {
                (COLOR_SIDEBAR_ACTIVE, COLOR_PRIMARY)
            } else if response.hovered() {
                (COLOR_BG_HOVER, COLOR_TEXT)
            } else {
                (Color32::TRANSPARENT, COLOR_TEXT_DIM)
            };
            
            // Draw background
            ui.painter().rect_filled(rect, egui::CornerRadius::same(8), bg);
            
            if is_active {
                 // Active border and side indicator
                 ui.painter().rect_stroke(rect, egui::CornerRadius::same(8), Stroke::new(1.0, COLOR_SIDEBAR_BORDER), StrokeKind::Inside);
                 ui.painter().rect_filled(
                    Rect::from_min_size(rect.left_center() + Vec2::new(4.0, -8.0), Vec2::new(3.0, 16.0)),
                    egui::CornerRadius::same(1),
                    COLOR_PRIMARY
                );
            }

            // Icon and Label - Tightened spacing and fixed alignment
            let text_pos = rect.left_center() + Vec2::new(14.0, 0.0);
            ui.painter().text(
                text_pos,
                egui::Align2::LEFT_CENTER,
                format!("{}  {}", icon.replace("\u{FE0F}", ""), label),
                egui::FontId::proportional(13.0),
                text_col
            );
        }
        ui.add_space(4.0);
    }

    // Bottom System Health
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.add_space(16.0);
        
        let (status_text, status_col) = match status {
            ServiceStatus::Running => ("STABLE", COLOR_SUCCESS),
            _ => ("OFFLINE", COLOR_TEXT_MUTED),
        };

        ui.horizontal_centered(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 3.5, status_col);
            ui.add_space(8.0);
            ui.label(RichText::new(format!("SYSTEM STATUS: {}", status_text)).size(9.0).strong().color(COLOR_TEXT_MUTED));
        });
    });
}

/// Render the dashboard panel
pub fn render_dashboard(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    _status: &ServiceStatus,
    sys_stats: &SystemStats,
    containers: &[ContainerInfo],
    docker_available: bool,
) {
    let mut something_changed = false;
    if !docker_available {
        ui.add_space(20.0);
        card_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚠").size(40.0).color(COLOR_ERROR));
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.heading(RichText::new("Docker Daemon Unreachable").color(COLOR_ERROR));
                    ui.label("DockStack requires Docker to manage your services. Please ensure Docker is running.");
                });
            });
        });
        return;
    }

    // Unified Top Metrics Bar
    ui.add_space(8.0);
    ui.label(RichText::new("SYSTEM WELLNESS").size(9.0).color(COLOR_TEXT_MUTED).strong().extra_letter_spacing(1.2));
    ui.add_space(12.0);
    
    egui::Grid::new("system_wellness_grid")
        .num_columns(4)
        .spacing(Vec2::new(16.0, 16.0))
        .min_col_width((ui.available_width() - 48.0) / 4.0)
        .show(ui, |ui| {
             stat_card(ui, "CPU Load", &format!("{:.0}%", sys_stats.cpu_usage), "📈", COLOR_PRIMARY);
             stat_card(ui, "Memory", &format!("{:.1}GB", sys_stats.memory_used as f64 / 1024.0 / 1024.0 / 1024.0), "💾", COLOR_SECONDARY);
             stat_card(ui, "Containers", &format!("{}", containers.len()), "🐳", COLOR_SUCCESS);
             stat_card(ui, "Network", "100%", "🛡", COLOR_ACCENT);
             ui.end_row();
        });

    ui.add_space(32.0);

    // Workspace and Domain Configuration
    ui.columns(2, |columns| {
        columns[0].vertical(|ui| {
            ui.label(RichText::new("WORKSPACE CONTEXT").size(9.0).color(COLOR_TEXT_MUTED).strong().extra_letter_spacing(1.2));
            ui.add_space(10.0);
            
            card_frame(ui, |ui| {
                 ui.set_width(ui.available_width());
                 ui.set_height(140.0);
                 ui.horizontal(|ui| {
                     ui.add(egui::Image::new(egui::include_image!("../../assets/images/icon.png"))
                        .max_size(Vec2::new(32.0, 32.0))
                        .corner_radius(8.0));
                     ui.add_space(12.0);
                     ui.vertical(|ui| {
                        if let Some(project) = config.active_project_mut() {
                            ui.label(RichText::new(&project.name).size(20.0).strong().color(COLOR_TEXT));
                            ui.label(RichText::new(&project.directory).size(11.0).color(COLOR_TEXT_DIM));
                        }
                     });
                 });
                 
                 if let Some(project) = config.active_project_mut() {
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🌐 Domain:").size(11.0).color(COLOR_TEXT_DIM));
                        if ui.add(egui::TextEdit::singleline(&mut project.domain).desired_width(120.0)).changed() {
                            something_changed = true;
                        }
                        if ui.button("📋").clicked() {
                            ui.ctx().copy_text(format!("127.0.0.1  {}", project.domain));
                        }
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(RichText::new("🔗  Open").strong()).fill(COLOR_BG_HOVER)).clicked() {
                             let port = project.services.get("nginx").map(|s| s.port).or_else(|| project.services.get("apache").map(|s| s.port)).unwrap_or(80);
                             utils::open_url(&format!("http://localhost:{}", port));
                        }
                        ui.add_space(8.0);
                        if ui.add(egui::Button::new(RichText::new("📂  Explore").strong()).fill(COLOR_BG_HOVER)).clicked() {
                            utils::open_directory(&project.directory);
                        }
                    });
                 }
            });
        });

        columns[1].vertical(|ui| {
            ui.label(RichText::new("DOCKER ENGINE").size(9.0).color(COLOR_TEXT_MUTED).strong().extra_letter_spacing(1.2));
            ui.add_space(10.0);

            card_frame(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(140.0); 
                ui.label(RichText::new("Runtime Connectivity").strong());
                ui.add_space(12.0);
                ui.horizontal_centered(|ui| {
                    status_dot(ui, docker_available);
                    ui.add_space(8.0);
                    ui.label(RichText::new(if docker_available { "Daemon is Online" } else { "Daemon Offline" }).color(COLOR_TEXT).strong());
                });
                ui.add_space(10.0);
                ui.label(RichText::new("API: 1.44  •  v25.0.3").size(11.0).color(COLOR_TEXT_DIM));
            });
        });
    });

    if something_changed {
        config.save();
    }

    ui.add_space(40.0);
    ui.separator();
    ui.add_space(32.0);

    // Services Grid
    ui.horizontal(|ui| {
        ui.add(egui::Image::new(egui::include_image!("../../assets/images/icon.png"))
            .max_size(Vec2::new(20.0, 20.0))
            .corner_radius(5.0));
        ui.add_space(8.0);
        ui.label(RichText::new("SERVICE STACK OVERVIEW").size(9.0).color(COLOR_TEXT_MUTED).strong().extra_letter_spacing(1.2));
    });
    ui.add_space(18.0);

    if let Some(project) = config.active_project() {
        let enabled_services: Vec<_> = project.services.iter().filter(|(_, v)| v.enabled).collect();
        
        if enabled_services.is_empty() {
            ui.label(RichText::new("No services enabled in this stack.").color(COLOR_TEXT_MUTED).italics());
        } else {
            egui::Grid::new("dash_services_grid")
                .num_columns(2)
                .spacing(Vec2::new(16.0, 16.0))
                .min_col_width((ui.available_width() - 16.0) / 2.0)
                .show(ui, |ui| {
                    for (i, (name, svc)) in enabled_services.iter().enumerate() {
                        let info = crate::services::get_service_info(name);
                        let display_name = info.as_ref().map(|i| i.display_name.clone()).unwrap_or(name.to_string());
                        let icon = info.as_ref().map(|i| i.icon).unwrap_or("❓");
                        let is_running = containers.iter().any(|c| c.name.contains(name.as_str()) && c.state.contains("running"));
                        
                        service_card_compact(ui, &display_name, icon, &svc.version, svc.port, is_running);
                        
                        if (i + 1) % 2 == 0 {
                            ui.end_row();
                        }
                    }
                });
        }
    }
}

fn stat_card(ui: &mut egui::Ui, title: &str, value: &str, icon: &str, accent: Color32) {
    egui::Frame::new()
        .fill(COLOR_BG_CARD)
        .corner_radius(egui::CornerRadius::same(12))
        .stroke(Stroke::new(1.0, COLOR_BORDER))
        .inner_margin(16.0)
        .show(ui, |ui| {
             ui.set_width(ui.available_width());
             ui.set_height(86.0); // Strictly fixed height for stat cards

             ui.horizontal_centered(|ui| {
                 // Premium Icon Container with Glow
                 let (rect, _) = ui.allocate_exact_size(Vec2::new(52.0, 52.0), egui::Sense::hover());
                 
                 // Glow effect
                 ui.painter().circle_filled(rect.center(), 24.0, accent.gamma_multiply(0.1));
                 ui.painter().circle_stroke(rect.center(), 20.0, Stroke::new(1.0, accent.gamma_multiply(0.2)));
                 
                 ui.painter().text(
                     rect.center(), 
                     egui::Align2::CENTER_CENTER, 
                     icon, 
                     egui::FontId::proportional(26.0), 
                     accent
                 );
                 
                 ui.add_space(14.0);
                 
                 ui.vertical(|ui| {
                     ui.label(RichText::new(title.to_uppercase()).size(11.0).color(COLOR_TEXT_MUTED).strong());
                     ui.add_space(2.0);
                     ui.label(RichText::new(value).size(26.0).strong().color(COLOR_TEXT));
                 });
             });
             
             // Sleek Bottom Accent Line
             let rect = ui.min_rect();
             ui.painter().rect_filled(
                 Rect::from_min_size(rect.left_bottom() + Vec2::new(12.0, -4.0), Vec2::new(rect.width() - 24.0, 3.0)),
                 egui::CornerRadius::same(1),
                 accent
             );
        });
}

fn service_card_compact(ui: &mut egui::Ui, name: &str, icon: &str, version: &str, port: u16, running: bool) {
    egui::Frame::new()
        .fill(COLOR_BG_CARD)
        .corner_radius(egui::CornerRadius::same(12))
        .stroke(Stroke::new(1.0, if running { COLOR_PRIMARY.gamma_multiply(0.4) } else { COLOR_BORDER }))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(68.0); // Unified height

            ui.horizontal_centered(|ui| {
                // Icon styling in panel-like box
                let (rect, _) = ui.allocate_exact_size(Vec2::new(42.0, 42.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, egui::CornerRadius::same(10), COLOR_BG_PANEL);
                ui.painter().rect_stroke(rect, egui::CornerRadius::same(10), Stroke::new(1.0, COLOR_BORDER), StrokeKind::Inside);
                
                ui.painter().text(
                    rect.center() + Vec2::new(0.0, 1.0), 
                    egui::Align2::CENTER_CENTER, 
                    icon, 
                    egui::FontId::proportional(20.0), 
                    Color32::WHITE
                );

                ui.add_space(14.0);
                
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(name).size(16.0).strong().color(COLOR_TEXT));
                        if running {
                             ui.add_space(8.0);
                             ui.label(RichText::new("●").size(10.0).color(COLOR_SUCCESS));
                        }
                    });
                    ui.add_space(1.0);
                    ui.label(RichText::new(format!("v{} ● Port: {}", version, port)).size(11.0).color(COLOR_TEXT_DIM));
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if running {
                        ui.label(RichText::new("ONLINE").size(9.0).strong().color(COLOR_SUCCESS).extra_letter_spacing(1.0));
                    } else {
                        ui.label(RichText::new("OFFLINE").size(9.0).strong().color(COLOR_TEXT_MUTED).extra_letter_spacing(1.0));
                    }
                });
            });
        });
}


pub fn render_services(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    containers: &[ContainerInfo],
) {
    let mut something_changed = false;
    
    ui.horizontal(|ui| {
        ui.add(egui::Image::new(egui::include_image!("../../assets/images/icon.png"))
            .max_size(Vec2::new(24.0, 24.0))
            .corner_radius(6.0));
        ui.add_space(8.0);
        ui.label(RichText::new("SERVICE STACK CONFIGURATION").size(10.0).color(COLOR_TEXT_MUTED).strong().extra_letter_spacing(1.2));
        
        if let Some(project) = config.active_project() {
            ui.label(RichText::new(format!("({})", project.services.len())).size(10.0).color(COLOR_TEXT_DIM));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Button::new(RichText::new("➕ Add Custom Service").strong().color(COLOR_BG_PANEL)).fill(COLOR_ACCENT)).clicked() {
                if let Some(project) = config.active_project_mut() {
                    let id = uuid::Uuid::new_v4().to_string()[..4].to_string();
                    let name = format!("custom-{}", id);
                    project.services.insert(name.clone(), ServiceConfig {
                        enabled: false,
                        is_custom: true,
                        is_locked: false,
                        display_name: Some(name),
                        image: Some("nginx".to_string()),
                        port: 8080,
                        version: "latest".to_string(),
                        env_vars: HashMap::new(),
                        settings: HashMap::new(),
                    });
                    something_changed = true;
                }
            }
        });
    });
    ui.add_space(16.0);
    
    let mut service_to_remove = None;
    let registry = get_service_registry();
    let categories = vec![
            ServiceCategory::WebServer,
            ServiceCategory::Database,
            ServiceCategory::Runtime,
            ServiceCategory::Cache,
            ServiceCategory::Admin,
            ServiceCategory::Security,
            ServiceCategory::Custom,
        ];

        for category in categories {
            let mut services_to_render = Vec::new();

            if category == ServiceCategory::Custom {
                if let Some(project) = config.active_project() {
                    for (name, svc) in &project.services {
                        if svc.is_custom {
                            services_to_render.push((
                                name.clone(),
                                svc.display_name.clone().unwrap_or_else(|| name.clone()),
                                "User-defined docker service".to_string(), // description
                                "🧩".to_string(), // icon
                            ));
                        }
                    }
                }
            } else {
                for svc_info in registry.iter().filter(|s| s.category == category) {
                    if let Some(project) = config.active_project() {
                        if project.services.contains_key(&svc_info.name) {
                            services_to_render.push((
                                svc_info.name.clone(),
                                svc_info.display_name.clone(),
                                svc_info.description.clone(),
                                svc_info.icon.to_string(),
                            ));
                        }
                    }
                }
            }

            if services_to_render.is_empty() { continue; }

            ui.label(RichText::new(category.label()).size(14.0).strong().color(COLOR_ACCENT));
            ui.add_space(8.0);

            for (id, display_name, description, icon) in services_to_render {
                if let Some(project) = config.active_project_mut() {
                    if let Some(svc) = project.services.get_mut(&id) {
                        ui.push_id(&id, |ui| {
                             let is_running = containers.iter().any(|c| c.name.contains(&id) && c.state.contains("running"));
                             
                             egui::Frame::new()
                                .fill(COLOR_BG_CARD)
                                .corner_radius(egui::CornerRadius::same(12))
                                .stroke(Stroke::new(1.0, COLOR_BORDER))
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.set_min_height(72.0); // Consistent Height

                                ui.horizontal(|ui| {
                                    // Status & Icon container
                                    let (rect, _) = ui.allocate_exact_size(Vec2::new(48.0, 48.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, egui::CornerRadius::same(10), COLOR_BG_PANEL);
                                    ui.painter().text(
                                        rect.center() + Vec2::new(0.0, 1.0), 
                                        egui::Align2::CENTER_CENTER, 
                                        icon.replace("\u{FE0F}", ""), 
                                        egui::FontId::proportional(22.0), 
                                        Color32::WHITE
                                    );

                                    ui.add_space(16.0);
                                    
                                    // Info
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&display_name).size(18.0).strong().color(COLOR_TEXT));
                                            if is_running {
                                                ui.add_space(8.0);
                                                ui.label(RichText::new("● RUNNING").size(10.0).color(COLOR_SUCCESS).strong());
                                            }
                                        });
                                        ui.add_space(4.0);
                                        ui.label(RichText::new(&description).size(13.0).color(COLOR_TEXT_DIM));
                                    });
                                    
                                    // Controls (Right aligned)
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                         // Toggle
                                        let mut enabled = svc.enabled;
                                        if toggle_switch(ui, &mut enabled).changed() {
                                            svc.enabled = enabled;
                                            if id == "ssl" { project.ssl_enabled = enabled; }
                                        }
                                        
                                        ui.add_space(24.0);
                                        
                                        // Config actions
                                        ui.menu_button(RichText::new("⚙ Config").size(13.0).color(COLOR_TEXT), |ui| {
                                             if svc.is_custom {
                                                 if ui.button(RichText::new("🗑 Remove Service").color(COLOR_ERROR)).clicked() {
                                                     service_to_remove = Some(id.clone());
                                                     ui.close_menu();
                                                 }
                                             } else {
                                                 let config_path = match id.as_str() {
                                                    "nginx" => Some(std::path::Path::new(&project.directory).join("nginx/default.conf")),
                                                    "apache" => Some(std::path::Path::new(&project.directory).join("apache/httpd.conf")),
                                                    "php" => Some(std::path::Path::new(&project.directory).join("php/php.ini")),
                                                    "mysql" => Some(std::path::Path::new(&project.directory).join("mysql/my.cnf")),
                                                    "postgresql" => Some(std::path::Path::new(&project.directory).join("postgresql/postgresql.conf")),
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
                                             }
                                        });
                                        
                                        ui.label(RichText::new(format!("Port: {}", svc.port)).size(13.0).color(COLOR_TEXT_MUTED).monospace());
                                    });
                                });
                                
                                // Premium Customization & Environment
                                let adv_toggle_id = ui.id().with(format!("adv_toggle_{}", id));
                                let mut show_advanced = ui.data_mut(|d| d.get_temp::<bool>(adv_toggle_id).unwrap_or(false));
                                
                                ui.add_space(8.0);
                                let btn_text = if show_advanced { "▼  Hide Advanced Settings" } else { "▶🛠  Customization & Environment" };
                                if ui.selectable_label(show_advanced, RichText::new(btn_text).size(12.0).strong().color(COLOR_ACCENT)).clicked() {
                                    show_advanced = !show_advanced;
                                    ui.data_mut(|d| d.insert_temp(adv_toggle_id, show_advanced));
                                }

                                 if show_advanced {
                                      ui.add_space(8.0);
                                      ui.vertical(|ui| {
                                                  egui::Frame::new()
                                                      .fill(COLOR_BG_PANEL)
                                                      .corner_radius(egui::CornerRadius::same(8))
                                                      .inner_margin(16.0)
                                                      .show(ui, |ui| {
                                                          ui.set_width(ui.available_width()); // Stretch to full width
                                                          
                                                          egui::Grid::new(format!("grid_{}", id))
                                                              .spacing(Vec2::new(60.0, 16.0))
                                                              .show(ui, |ui| {
                                                                  // Column 1: Identity
                                                                  ui.vertical(|ui| {
                                                                      if svc.is_custom {
                                                                          ui.label(RichText::new("Display Name").size(11.0).color(COLOR_TEXT_DIM));
                                                                          ui.add_space(4.0);
                                                                          let mut name = svc.display_name.clone().unwrap_or_else(|| id.clone());
                                                                          if ui.add(egui::TextEdit::singleline(&mut name).desired_width(280.0)).changed() {
                                                                              svc.display_name = Some(name);
                                                                              something_changed = true;
                                                                          }
                                                                      } else {
                                                                          ui.label(RichText::new("Service ID").size(11.0).color(COLOR_TEXT_DIM));
                                                                          ui.add_space(4.0);
                                                                          ui.label(RichText::new(&id).strong().color(COLOR_PRIMARY));
                                                                      }
                                                                  });

                                                                  // Column 2: Docker Image
                                                                  ui.vertical(|ui| {
                                                                      ui.label(RichText::new("Docker Image").size(11.0).color(COLOR_TEXT_DIM));
                                                                      ui.add_space(4.0);
                                                                      if svc.is_custom {
                                                                          let mut img = svc.image.clone().unwrap_or_default();
                                                                          if ui.add(egui::TextEdit::singleline(&mut img).desired_width(350.0)).changed() {
                                                                              svc.image = Some(img);
                                                                              something_changed = true;
                                                                          }
                                                                      } else {
                                                                          ui.label(RichText::new(svc.image.as_ref().unwrap_or(&id)).strong().color(COLOR_ACCENT));
                                                                      }
                                                                  });

                                                                  // Column 3: Version
                                                                  ui.vertical(|ui| {
                                                                      ui.label(RichText::new("Version").size(11.0).color(COLOR_TEXT_DIM));
                                                                      ui.add_space(4.0);
                                                                      if ui.add(egui::TextEdit::singleline(&mut svc.version).desired_width(100.0)).changed() {
                                                                          something_changed = true;
                                                                      }
                                                                  });

                                                                  // Column 4: Port
                                                                  ui.vertical(|ui| {
                                                                      let is_available = crate::utils::is_port_available(svc.port);
                                                                      ui.horizontal(|ui| {
                                                                          ui.label(RichText::new("Host Port").size(11.0).color(COLOR_TEXT_DIM));
                                                                          ui.add_space(4.0);
                                                                          ui.label(RichText::new(if is_available { "✔" } else { "✘" }).size(10.0).color(if is_available { COLOR_SUCCESS } else { COLOR_ERROR }));
                                                                      });
                                                                      ui.add_space(4.0);
                                                                      if ui.add(egui::DragValue::new(&mut svc.port).range(1..=65535)).changed() {
                                                                          something_changed = true;
                                                                      }
                                                                  });

                                                                  // Column 5: Lock Configuration
                                                                  ui.vertical(|ui| {
                                                                      ui.label(RichText::new("Lock Config").size(11.0).color(COLOR_TEXT_DIM));
                                                                      ui.add_space(8.0);
                                                                      if ui.checkbox(&mut svc.is_locked, "").on_hover_text("If locked, DockStack won't overwrite your manual changes to config files").changed() {
                                                                          something_changed = true;
                                                                      }
                                                                  });
                                                                  ui.end_row();
                                                              });
                                                      });

                                          ui.add_space(8.0);
                                          ui.separator();
                                          ui.add_space(8.0);

                                         // PHP Configuration
                                         if id == "php" {
                                             ui.label(RichText::new("PHP Version & Extensions").strong().color(COLOR_ACCENT));
                                             ui.horizontal(|ui| {
                                                 ui.label("Memory Limit:");
                                                 let mut limit = svc.settings.get("memory_limit").cloned().unwrap_or_else(|| "256M".to_string());
                                                 if ui.add(egui::TextEdit::singleline(&mut limit).desired_width(80.0)).changed() {
                                                     svc.settings.insert("memory_limit".to_string(), limit);
                                                     something_changed = true;
                                                 }
                                             });
                                             
                                             let extensions = svc.settings.get("extensions").cloned().unwrap_or_else(|| "pdo_mysql,gd,intl".to_string());
                                             let common_exts = vec!["pdo_mysql", "pdo_pgsql", "gd", "intl", "zip", "mbstring", "bcmath", "xml", "curl"];
                                             ui.horizontal_wrapped(|ui| {
                                                 let current = extensions.split(',').collect::<Vec<_>>();
                                                 for ext in common_exts {
                                                     let mut checked = current.contains(&ext);
                                                     if ui.checkbox(&mut checked, ext).changed() {
                                                         let mut list = current.iter().map(|s| s.to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>();
                                                         if checked { if !list.contains(&ext.to_string()) { list.push(ext.to_string()); } }
                                                         else { list.retain(|s| s != ext); }
                                                         svc.settings.insert("extensions".to_string(), list.join(","));
                                                         something_changed = true;
                                                     }
                                                 }
                                             });
                                             ui.add_space(8.0);
                                             ui.separator();
                                             ui.add_space(8.0);
                                         }

                                         // Database Configuration
                                         if id == "mysql" || id == "postgresql" {
                                              let root_key = if id == "mysql" { "MYSQL_ROOT_PASSWORD" } else { "POSTGRES_PASSWORD" };
                                              let db_key = if id == "mysql" { "MYSQL_DATABASE" } else { "POSTGRES_DB" };
                                              
                                              ui.label(RichText::new("Database Settings").strong().color(COLOR_ACCENT));
                                              egui::Grid::new("db_settings").show(ui, |ui| {
                                                  ui.label("Root Password:");
                                                  let mut pass = svc.env_vars.get(root_key).cloned().unwrap_or_default();
                                                  if ui.add(egui::TextEdit::singleline(&mut pass)).changed() {
                                                      svc.env_vars.insert(root_key.to_string(), pass);
                                                      something_changed = true;
                                                  }
                                                  ui.end_row();
                                                  
                                                  ui.label("Database Name:");
                                                  let mut db = svc.env_vars.get(db_key).cloned().unwrap_or_default();
                                                  if ui.add(egui::TextEdit::singleline(&mut db)).changed() {
                                                      svc.env_vars.insert(db_key.to_string(), db);
                                                      something_changed = true;
                                                  }
                                                  ui.end_row();
                                              });
                                              ui.add_space(8.0);
                                              ui.separator();
                                              ui.add_space(8.0);
                                         }

                                         // Environment Variables
                                         ui.label(RichText::new("Environment Variables").strong());
                                         let mut vars: Vec<(String, String)> = svc.env_vars.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                                         let mut env_changed = false;
                                         let mut to_remove = None;
                                         
                                         egui::Grid::new(format!("env_{}", id)).spacing(Vec2::new(12.0, 8.0)).show(ui, |ui| {
                                             for (i, (key, val)) in vars.iter_mut().enumerate() {
                                                 ui.push_id(i, |ui| {
                                                     if ui.add(egui::TextEdit::singleline(key).desired_width(140.0).hint_text("KEY")).changed() { env_changed = true; }
                                                     if ui.add(egui::TextEdit::singleline(val).desired_width(200.0).hint_text("VALUE")).changed() { env_changed = true; }
                                                     if ui.button(RichText::new("🗑").color(COLOR_ERROR)).clicked() { to_remove = Some(i); env_changed = true; }
                                                 });
                                                 ui.end_row();
                                             }
                                         });
                                         
                                         if ui.button(RichText::new("➕ Add Variable").color(COLOR_SUCCESS)).clicked() {
                                             vars.push(("NEW_VAR".to_string(), "VALUE".to_string()));
                                             env_changed = true;
                                         }

                                         if let Some(idx) = to_remove { vars.remove(idx); }
                                         if env_changed {
                                             svc.env_vars = vars.into_iter().collect();
                                             something_changed = true;
                                         }
                                     });
                                }
                            });
                        });
                        ui.add_space(12.0);
                    }
                }
            }
    }

    if let Some(id) = service_to_remove {
        if let Some(project) = config.active_project_mut() {
            project.services.remove(&id);
            something_changed = true;
        }
    }

    if something_changed {
        config.save();
    }
}

pub fn render_containers(ui: &mut egui::Ui, containers: &[ContainerInfo]) {
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
}

pub fn render_logs(ui: &mut egui::Ui, logs: &[String], clear_logs: &mut bool) {
    ui.add_space(10.0);
    ui.horizontal(|ui| {
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
              } else if ui.button(RichText::new("⏹ Reset").color(COLOR_ERROR)).clicked() { /* logic to kill */ }
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
    _config: &mut AppConfig,
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
                 if ui.button("Create").clicked() && !new_project_name.is_empty() {
                     // Logic handled in parent or here
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
