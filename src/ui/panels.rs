use egui::{self, Color32, RichText, ScrollArea, Vec2};
use crate::config::AppConfig;
use crate::docker::manager::{ContainerInfo, ServiceStatus};
use crate::monitor::{ContainerStats, SystemStats};
use crate::services::{get_service_registry, ServiceCategory};
use crate::port_scanner::{PortInfo, PortScanner};
use crate::ui::theme::*;
use crate::ui::widgets::*;
use crate::utils;

/// Render the sidebar
pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: &mut Tab,
    config: &AppConfig,
    status: &ServiceStatus,
) {
    let width = 220.0;
    
    // Direct rendering on parent panel (no extra Frame)
    ui.add_space(8.0);

    // Logo / Brand
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("⚡ DockStack")
                .size(24.0)
                .color(COLOR_PRIMARY)
                .strong(),
        );
        ui.label(
            RichText::new("DevStack Manager")
                .size(12.0)
                .color(COLOR_TEXT_DIM),
        );
    });

    ui.add_space(24.0);

    // Status indicator
    ui.vertical_centered(|ui| {
        let (status_text, status_color) = match status {
            ServiceStatus::Running => ("● Running", COLOR_SUCCESS),
            ServiceStatus::Starting => ("◌ Starting...", COLOR_WARNING),
            ServiceStatus::Stopping => ("◌ Stopping...", COLOR_WARNING),
            ServiceStatus::Stopped => ("○ Stopped", COLOR_TEXT_MUTED),
            ServiceStatus::Error(_) => ("✕ Error", COLOR_ERROR),
        };
        
        // Pill status
        egui::Frame::new()
            .fill(status_color.gamma_multiply(0.15))
            .corner_radius(egui::CornerRadius::same(12))
            .stroke(egui::Stroke::new(1.0, status_color.gamma_multiply(0.5)))
            .inner_margin(egui::Margin::symmetric(12, 4))
            .show(ui, |ui| {
                ui.label(RichText::new(status_text).size(12.0).color(status_color).strong());
            });
        
        if let ServiceStatus::Error(msg) = status {
            ui.add_space(8.0);
            ui.label(RichText::new(msg).size(11.0).color(COLOR_ERROR));
        }
    });

    ui.add_space(32.0);

    // Menu
    let tabs = vec![
        (Tab::Dashboard, "🏠", "Dashboard"),
        (Tab::Services, "📦", "Services"),
        (Tab::Containers, "🐳", "Containers"),
        (Tab::Logs, "📋", "Logs"),
        (Tab::Terminal, "💻", "Terminal"),
        (Tab::Ports, "🔌", "Port Scanner"),
        (Tab::Monitor, "📊", "Monitor"),
        (Tab::Settings, "⚙️", "Settings"),
    ];
    
    ui.label(RichText::new("MENU").size(11.0).color(COLOR_TEXT_MUTED).strong());
    ui.add_space(8.0);

    for (tab, icon, label) in tabs {
        let is_active = *active_tab == tab;
        let bg = if is_active {
            COLOR_SIDEBAR_ACTIVE
        } else {
            Color32::TRANSPARENT
        };
        let text_color = if is_active {
            COLOR_PRIMARY
        } else {
            COLOR_TEXT_DIM
        };

        let btn = egui::Button::new(
            RichText::new(format!("  {}  {}", icon, label))
                .size(14.0)
                .color(text_color)
                .strong(),
        )
        .fill(bg)
        .stroke(egui::Stroke::NONE)
        .corner_radius(egui::CornerRadius::same(8))
        .min_size(Vec2::new(width - 24.0, 42.0))
        .frame(true);

        ui.add_space(4.0);
        if ui.add(btn).clicked() {
            *active_tab = tab;
        }
    }

    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(16.0);
            
            // Project info
            if let Some(project) = config.active_project() {
            egui::Frame::new()
                .fill(COLOR_BG_PANEL)
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(8) // Correct integer type i8
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("📁").size(16.0));
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Active Project").size(10.0).color(COLOR_TEXT_MUTED));
                            ui.label(RichText::new(&project.name).size(13.0).color(COLOR_TEXT).strong());
                        });
                    });
                });
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
        ui.add_space(8.0);
        ui.heading(RichText::new("Dashboard").size(24.0).color(COLOR_TEXT).strong());
        ui.add_space(12.0);

        // Docker status
        if !docker_available {
            egui::Frame::new()
                .fill(Color32::from_rgba_premultiplied(239, 68, 68, 25))
                .corner_radius(egui::CornerRadius::same(8))
                .stroke(egui::Stroke::new(1.0, COLOR_ERROR))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚠").size(18.0).color(COLOR_ERROR));
                        ui.label(
                            RichText::new("Docker is not detected. Please install Docker Desktop or Docker Engine.")
                                .size(13.0)
                                .color(COLOR_ERROR),
                        );
                    });
                });
            ui.add_space(12.0);
        }

        // Quick stats row
        ui.horizontal(|ui| {
            stat_card(ui, "CPU", &format!("{:.1}%", sys_stats.cpu_usage), COLOR_PRIMARY);
            stat_card(
                ui,
                "RAM",
                &format!(
                    "{} / {}",
                    utils::format_bytes(sys_stats.memory_used),
                    utils::format_bytes(sys_stats.memory_total)
                ),
                COLOR_SECONDARY,
            );
            stat_card(
                ui,
                "Containers",
                &format!("{}", containers.len()),
                COLOR_SUCCESS,
            );

            if let Some(project) = config.active_project() {
                stat_card(
                    ui,
                    "Services",
                    &format!(
                        "{} enabled",
                        project.services.iter().filter(|(_, v)| v.enabled).count()
                    ),
                    COLOR_INFO,
                );
            }
        });

        ui.add_space(16.0);

        // Active services
        if let Some(project) = config.active_project() {
            card_frame(ui, |ui| {
                section_header(ui, "Active Services");
                ui.add_space(4.0);

                let enabled: Vec<_> = project
                    .services
                    .iter()
                    .filter(|(_, v)| v.enabled)
                    .collect();

                if enabled.is_empty() {
                    ui.label(
                        RichText::new("No services enabled. Go to Services tab to enable them.")
                            .color(COLOR_TEXT_MUTED),
                    );
                } else {
                    egui::Grid::new("active_services_grid")
                        .num_columns(4)
                        .spacing([12.0, 6.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Service").size(12.0).color(COLOR_TEXT_MUTED).strong());
                            ui.label(RichText::new("Port").size(12.0).color(COLOR_TEXT_MUTED).strong());
                            ui.label(RichText::new("Version").size(12.0).color(COLOR_TEXT_MUTED).strong());
                            ui.label(RichText::new("Status").size(12.0).color(COLOR_TEXT_MUTED).strong());
                            ui.end_row();

                            for (name, svc) in &enabled {
                                let info = crate::services::get_service_info(name);
                                let display = info
                                    .as_ref()
                                    .map(|i| format!("{} {}", i.icon, i.display_name))
                                    .unwrap_or_else(|| name.to_string());

                                ui.label(RichText::new(display).size(13.0).color(COLOR_TEXT));
                                ui.label(RichText::new(format!(":{}", svc.port)).size(13.0).color(COLOR_TEXT_DIM));
                                ui.label(RichText::new(&svc.version).size(13.0).color(COLOR_TEXT_DIM));

                                let is_running = containers
                                    .iter()
                                    .any(|c| c.name.contains(name.as_str()) && c.state.contains("running"));
                                status_dot(ui, is_running);
                                ui.end_row();
                            }
                        });
                }
            });
        }

        ui.add_space(12.0);

        // Quick actions
        card_frame(ui, |ui| {
            section_header(ui, "Quick Actions");
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if let Some(project) = config.active_project() {
                    if secondary_button(ui, "🌐 Open localhost").clicked() {
                        let port = project
                            .services
                            .get("nginx")
                            .map(|s| s.port)
                            .or_else(|| project.services.get("apache").map(|s| s.port))
                            .unwrap_or(80);
                        utils::open_url(&format!("http://localhost:{}", port));
                    }
                    if secondary_button(ui, "📂 Open Project Dir").clicked() {
                        utils::open_directory(&project.directory);
                    }
                }
            });
        });
    });
}

/// Stat card widget
fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, accent: Color32) {
    egui::Frame::new()
        .fill(COLOR_BG_CARD)
        .corner_radius(egui::CornerRadius::same(8))
        .stroke(egui::Stroke::new(1.0, COLOR_BORDER))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(140.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(label).size(11.0).color(COLOR_TEXT_MUTED));
                ui.add_space(2.0);
                ui.label(RichText::new(value).size(16.0).color(accent).strong());
            });
        });
}

/// Render the services panel
pub fn render_services(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    containers: &[ContainerInfo],
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);
        ui.heading(RichText::new("Services").size(24.0).color(COLOR_TEXT).strong());
        ui.label(RichText::new("Toggle services ON/OFF and configure them").color(COLOR_TEXT_MUTED));
        ui.add_space(12.0);

        let registry = get_service_registry();
        let categories = vec![
            ServiceCategory::Database,
            ServiceCategory::WebServer,
            ServiceCategory::Runtime,
            ServiceCategory::Cache,
            ServiceCategory::Admin,
            ServiceCategory::Security,
        ];

        for category in categories {
            let services_in_cat: Vec<_> = registry
                .iter()
                .filter(|s| s.category == category)
                .collect();

            if services_in_cat.is_empty() {
                continue;
            }

            card_frame(ui, |ui| {
                section_header(ui, category.label());
                ui.add_space(4.0);

                for svc_info in &services_in_cat {
                    if let Some(project) = config.active_project_mut() {
                        if let Some(svc) = project.services.get_mut(&svc_info.name) {
                            ui.horizontal(|ui| {
                                let mut enabled = svc.enabled;
                                if toggle_switch(ui, &mut enabled).changed() {
                                    svc.enabled = enabled;
                                    if svc_info.name == "ssl" {
                                        project.ssl_enabled = enabled;
                                    }
                                }

                                ui.add_space(8.0);

                                ui.label(
                                    RichText::new(svc_info.icon)
                                        .size(16.0),
                                );
                                ui.label(
                                    RichText::new(&svc_info.display_name)
                                        .size(14.0)
                                        .color(if svc.enabled { COLOR_TEXT } else { COLOR_TEXT_MUTED }),
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let is_running = containers
                                        .iter()
                                        .any(|c| c.name.contains(&svc_info.name) && c.state.contains("running"));
                                    status_dot(ui, is_running);

                                    ui.add_space(8.0);

                                    ui.label(
                                        RichText::new(format!(":{}", svc.port))
                                            .size(12.0)
                                            .color(COLOR_TEXT_MUTED),
                                    );
                                });
                            });

                            ui.horizontal(|ui| {
                                ui.add_space(52.0);
                                ui.label(
                                    RichText::new(&svc_info.description)
                                        .size(11.0)
                                        .color(COLOR_TEXT_MUTED),
                                );
                            });

                            // Ensure defaults exist for known services
                            if svc_info.name == "phpmyadmin" {
                                svc.env_vars.entry("PMA_USER".to_string()).or_insert("root".to_string());
                                svc.env_vars.entry("PMA_PASSWORD".to_string()).or_insert("root".to_string());
                            }
                            if svc_info.name == "pgadmin" {
                                svc.env_vars.entry("PGADMIN_DEFAULT_EMAIL".to_string()).or_insert("admin@admin.com".to_string());
                                svc.env_vars.entry("PGADMIN_DEFAULT_PASSWORD".to_string()).or_insert("admin".to_string());
                            }

                            ui.horizontal(|ui| {
                                ui.add_space(52.0);
                                egui::CollapsingHeader::new("Environment Variables / Credentials")
                                    .id_source(&svc_info.name)
                                    .show(ui, |ui| {
                                        let mut keys: Vec<String> = svc.env_vars.keys().cloned().collect();
                                        keys.sort();
                                        
                                        egui::Grid::new(format!("env_grid_{}", svc_info.name))
                                            .num_columns(2)
                                            .spacing([12.0, 4.0])
                                            .show(ui, |ui| {
                                                for key in keys {
                                                    if let Some(val) = svc.env_vars.get_mut(&key) {
                                                        ui.label(RichText::new(&key).family(egui::FontFamily::Monospace).size(11.0).color(COLOR_TEXT_DIM));
                                                        ui.add(egui::TextEdit::singleline(val).desired_width(200.0).font(egui::FontId::monospace(12.0)));
                                                        ui.end_row();
                                                    }
                                                }
                                            });
                                    });
                            });

                            ui.add_space(4.0);
                        }
                    }
                }
            });

            ui.add_space(8.0);
        }
    });
}

/// Render containers panel (docker ps)
pub fn render_containers(ui: &mut egui::Ui, containers: &[ContainerInfo]) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);
        ui.heading(RichText::new("Containers").size(24.0).color(COLOR_TEXT).strong());
        ui.label(RichText::new("Docker container list (docker ps)").color(COLOR_TEXT_MUTED));
        ui.add_space(12.0);

        if containers.is_empty() {
            card_frame(ui, |ui| {
                ui.label(RichText::new("No containers running").color(COLOR_TEXT_MUTED));
            });
        } else {
            card_frame(ui, |ui| {
                egui::Grid::new("containers_grid")
                    .num_columns(5)
                    .spacing([16.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Name").size(12.0).color(COLOR_TEXT_MUTED).strong());
                        ui.label(RichText::new("Image").size(12.0).color(COLOR_TEXT_MUTED).strong());
                        ui.label(RichText::new("Status").size(12.0).color(COLOR_TEXT_MUTED).strong());
                        ui.label(RichText::new("Ports").size(12.0).color(COLOR_TEXT_MUTED).strong());
                        ui.label(RichText::new("State").size(12.0).color(COLOR_TEXT_MUTED).strong());
                        ui.end_row();

                        for container in containers {
                            let running = container.state.contains("running");
                            ui.horizontal(|ui| {
                                status_dot(ui, running);
                                ui.label(RichText::new(&container.name).size(12.0).color(COLOR_TEXT));
                            });
                            ui.label(RichText::new(&container.image).size(12.0).color(COLOR_TEXT_DIM));
                            ui.label(RichText::new(&container.status).size(12.0).color(COLOR_TEXT_DIM));
                            ui.label(
                                RichText::new(utils::truncate_string(&container.ports, 40))
                                    .size(11.0)
                                    .color(COLOR_TEXT_MUTED),
                            );
                            let state_color = if running { COLOR_SUCCESS } else { COLOR_ERROR };
                            ui.label(RichText::new(&container.state).size(12.0).color(state_color));
                            ui.end_row();
                        }
                    });
            });
        }
    });
}

/// Render logs panel
pub fn render_logs(ui: &mut egui::Ui, logs: &[String], clear_logs: &mut bool) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Logs").size(24.0).color(COLOR_TEXT).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if danger_button(ui, "🗑 Clear").clicked() {
                *clear_logs = true;
            }
        });
    });
    ui.label(RichText::new("Realtime Docker Compose logs").color(COLOR_TEXT_MUTED));
    ui.add_space(8.0);

    card_frame(ui, |ui| {
        ScrollArea::vertical()
            .max_height(ui.available_height() - 20.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                for line in logs {
                    let color = if line.contains("ERROR") || line.contains("error") {
                        COLOR_ERROR
                    } else if line.contains("WARN") || line.contains("warn") {
                        COLOR_WARNING
                    } else if line.starts_with("[DockStack]") {
                        COLOR_PRIMARY
                    } else {
                        COLOR_TEXT_DIM
                    };
                    ui.label(
                        RichText::new(line)
                            .size(11.0)
                            .color(color)
                            .family(egui::FontFamily::Monospace),
                    );
                }
            });
    });
}

/// Render terminal panel
pub fn render_terminal(
    ui: &mut egui::Ui,
    output_lines: &[String],
    input_buffer: &mut String,
    send_input: &mut bool,
    clear_terminal: &mut bool,
    start_terminal: &mut bool,
    terminal_running: bool,
) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Terminal").size(24.0).color(COLOR_TEXT).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !terminal_running {
                if primary_button(ui, "▶ Start Shell").clicked() {
                    *start_terminal = true;
                }
            }
            if danger_button(ui, "🗑 Clear").clicked() {
                *clear_terminal = true;
            }
        });
    });
    ui.add_space(8.0);

    card_frame(ui, |ui| {
        ScrollArea::vertical()
            .max_height(ui.available_height() - 40.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                for line in output_lines {
                    let color = if line.starts_with("$ ") {
                        COLOR_PRIMARY
                    } else {
                        COLOR_TEXT_DIM
                    };
                    ui.label(
                        RichText::new(line)
                            .size(12.0)
                            .color(color)
                            .family(egui::FontFamily::Monospace),
                    );
                }
            });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("$").size(13.0).color(COLOR_PRIMARY).family(egui::FontFamily::Monospace));

            let response = ui.add(
                egui::TextEdit::singleline(input_buffer)
                    .desired_width(ui.available_width() - 80.0)
                    .font(egui::FontId::monospace(12.0))
                    .text_color(COLOR_TEXT),
            );

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                *send_input = true;
                response.request_focus();
            }

            if primary_button(ui, "Run").clicked() {
                *send_input = true;
            }
        });
    });
}

/// Render port scanner panel
pub fn render_ports(ui: &mut egui::Ui, port_infos: &[PortInfo], scan_ports: &mut bool) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Port Scanner").size(24.0).color(COLOR_TEXT).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary_button(ui, "🔄 Scan").clicked() {
                    *scan_ports = true;
                }
            });
        });
        ui.label(RichText::new("Scan ports for conflicts before starting services").color(COLOR_TEXT_MUTED));
        ui.add_space(12.0);

        card_frame(ui, |ui| {
            egui::Grid::new("ports_grid")
                .num_columns(4)
                .spacing([16.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(RichText::new("Port").size(12.0).color(COLOR_TEXT_MUTED).strong());
                    ui.label(RichText::new("Status").size(12.0).color(COLOR_TEXT_MUTED).strong());
                    ui.label(RichText::new("Process").size(12.0).color(COLOR_TEXT_MUTED).strong());
                    ui.label(RichText::new("Suggestion").size(12.0).color(COLOR_TEXT_MUTED).strong());
                    ui.end_row();

                    for info in port_infos {
                        ui.label(RichText::new(format!(":{}", info.port)).size(13.0).color(COLOR_TEXT));

                        if info.in_use {
                            ui.label(RichText::new("● In Use").size(12.0).color(COLOR_ERROR));
                        } else {
                            ui.label(RichText::new("● Available").size(12.0).color(COLOR_SUCCESS));
                        }

                        ui.label(
                            RichText::new(utils::truncate_string(&info.process, 30))
                                .size(11.0)
                                .color(COLOR_TEXT_MUTED),
                        );

                        if info.in_use {
                            let alt = PortScanner::find_available_port(info.port + 1);
                            ui.label(
                                RichText::new(format!("→ Use :{}", alt))
                                    .size(12.0)
                                    .color(COLOR_WARNING),
                            );
                        } else {
                            ui.label(RichText::new("—").size(12.0).color(COLOR_TEXT_MUTED));
                        }
                        ui.end_row();
                    }
                });
        });
    });
}

/// Render resource monitor panel
pub fn render_monitor(
    ui: &mut egui::Ui,
    sys_stats: &SystemStats,
    container_stats: &[ContainerStats],
    cpu_history: &[f32],
    mem_history: &[f32],
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);
        ui.heading(RichText::new("Resource Monitor").size(24.0).color(COLOR_TEXT).strong());
        ui.label(RichText::new("System and container resource usage").color(COLOR_TEXT_MUTED));
        ui.add_space(12.0);

        // System stats
        ui.horizontal(|ui| {
            card_frame(ui, |ui| {
                ui.set_min_width(280.0);
                ui.label(RichText::new("CPU Usage").size(14.0).color(COLOR_TEXT).strong());
                ui.label(
                    RichText::new(format!("{:.1}%", sys_stats.cpu_usage))
                        .size(28.0)
                        .color(COLOR_PRIMARY)
                        .strong(),
                );
                ui.add_space(4.0);
                sparkline(ui, cpu_history, 100.0, COLOR_PRIMARY, Vec2::new(260.0, 60.0));
            });

            card_frame(ui, |ui| {
                ui.set_min_width(280.0);
                ui.label(RichText::new("Memory Usage").size(14.0).color(COLOR_TEXT).strong());
                ui.label(
                    RichText::new(format!(
                        "{} / {} ({:.1}%)",
                        utils::format_bytes(sys_stats.memory_used),
                        utils::format_bytes(sys_stats.memory_total),
                        sys_stats.memory_percent
                    ))
                    .size(16.0)
                    .color(COLOR_SECONDARY)
                    .strong(),
                );
                ui.add_space(4.0);
                sparkline(ui, mem_history, 100.0, COLOR_SECONDARY, Vec2::new(260.0, 60.0));
            });
        });

        ui.add_space(12.0);

        card_frame(ui, |ui| {
            section_header(ui, "Container Resources");

            if container_stats.is_empty() {
                ui.label(RichText::new("No containers running").color(COLOR_TEXT_MUTED));
            } else {
                egui::Grid::new("container_stats_grid")
                    .num_columns(5)
                    .spacing([16.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Container").size(12.0).color(COLOR_TEXT_MUTED).strong());
                        ui.label(RichText::new("CPU").size(12.0).color(COLOR_TEXT_MUTED).strong());
                        ui.label(RichText::new("Memory").size(12.0).color(COLOR_TEXT_MUTED).strong());
                        ui.label(RichText::new("Net I/O").size(12.0).color(COLOR_TEXT_MUTED).strong());
                        ui.label(RichText::new("Block I/O").size(12.0).color(COLOR_TEXT_MUTED).strong());
                        ui.end_row();

                        for stats in container_stats {
                            ui.label(RichText::new(&stats.name).size(12.0).color(COLOR_TEXT));
                            ui.label(RichText::new(&stats.cpu_percent).size(12.0).color(COLOR_PRIMARY));
                            ui.label(RichText::new(&stats.mem_usage).size(12.0).color(COLOR_SECONDARY));
                            ui.label(RichText::new(&stats.net_io).size(11.0).color(COLOR_TEXT_DIM));
                            ui.label(RichText::new(&stats.block_io).size(11.0).color(COLOR_TEXT_DIM));
                            ui.end_row();
                        }
                    });
            }
        });
    });
}

/// Render settings panel
pub fn render_settings(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    new_project_name: &mut String,
    generate_ssl: &mut bool,
    remove_ssl: &mut bool,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);
        ui.heading(RichText::new("Settings").size(24.0).color(COLOR_TEXT).strong());
        ui.add_space(12.0);

        // Project management
        card_frame(ui, |ui| {
            section_header(ui, "📁 Projects");
            ui.add_space(4.0);

            let project_ids: Vec<(String, String)> = config
                .projects
                .iter()
                .map(|p| (p.id.clone(), p.name.clone()))
                .collect();

            for (id, name) in &project_ids {
                ui.horizontal(|ui| {
                    let is_active = config.active_project_id.as_deref() == Some(id.as_str());
                    let radio = ui.radio(is_active, "");
                    if radio.clicked() {
                        config.active_project_id = Some(id.clone());
                        config.save();
                    }
                    ui.label(
                        RichText::new(name)
                            .size(14.0)
                            .color(if is_active { COLOR_TEXT } else { COLOR_TEXT_DIM }),
                    );

                    if project_ids.len() > 1 {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if danger_button(ui, "🗑").clicked() {
                                config.remove_project(id);
                            }
                        });
                    }
                });
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(new_project_name)
                        .hint_text("New project name...")
                        .desired_width(200.0),
                );
                if primary_button(ui, "➕ Add Project").clicked() && !new_project_name.is_empty() {
                    config.add_project(new_project_name.clone());
                    new_project_name.clear();
                }
            });
        });

        ui.add_space(12.0);

        // Docker settings
        card_frame(ui, |ui| {
            section_header(ui, "🐳 Docker");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Docker path:").size(13.0).color(COLOR_TEXT_DIM));
                ui.add(
                    egui::TextEdit::singleline(&mut config.docker_path)
                        .desired_width(300.0),
                );
            });
        });

        ui.add_space(12.0);

        // SSL settings
        card_frame(ui, |ui| {
            section_header(ui, "🔒 SSL / HTTPS");
            ui.add_space(4.0);

            if let Some(project) = config.active_project() {
                let has_certs = crate::ssl::SslManager::certs_exist(&project.directory);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if has_certs {
                            "✅ SSL certificates found"
                        } else {
                            "❌ No SSL certificates"
                        })
                        .size(13.0)
                        .color(if has_certs { COLOR_SUCCESS } else { COLOR_TEXT_MUTED }),
                    );
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if primary_button(ui, "🔑 Generate Self-Signed Cert").clicked() {
                        *generate_ssl = true;
                    }
                    if has_certs {
                        if danger_button(ui, "🗑 Remove Certs").clicked() {
                            *remove_ssl = true;
                        }
                    }
                });
            }
        });

        ui.add_space(12.0);

        // Window settings
        card_frame(ui, |ui| {
            section_header(ui, "🖥 Window");
            ui.add_space(4.0);
            ui.checkbox(
                &mut config.window.minimize_to_tray,
                RichText::new("Minimize to system tray").size(13.0).color(COLOR_TEXT),
            );
        });

        config.save();
    });
}

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
