use eframe::egui;
use std::time::Instant;

use crate::config::AppConfig;
use crate::docker::manager::{DockerEvent, DockerManager, ServiceStatus};
use crate::monitor::{MonitorEvent, ResourceMonitor, SystemStats, ContainerStats};
use crate::port_scanner::{PortInfo, PortScanner};
use crate::ssl::SslManager;
use crate::terminal::EmbeddedTerminal;
use crate::tray::{SystemTray, TrayCommand};
use crate::ui::panels::{self, Tab};
use crate::ui::theme;

pub struct DockStackApp {
    config: AppConfig,
    docker: DockerManager,
    monitor: ResourceMonitor,
    terminal: EmbeddedTerminal,
    tray: SystemTray,

    // UI State
    active_tab: Tab,
    terminal_input: String,
    new_project_name: String,

    // Cached data
    port_infos: Vec<PortInfo>,
    sys_stats: SystemStats,
    container_stats: Vec<ContainerStats>,
    cpu_history: Vec<f32>,
    mem_history: Vec<f32>,

    // Flags
    docker_available: bool,
    tray_initialized: bool,
    _last_refresh: Instant,
    last_container_refresh: Instant,
}

impl DockStackApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);

        let config = AppConfig::load();
        let docker = DockerManager::new();
        let monitor = ResourceMonitor::new();
        let terminal = EmbeddedTerminal::new();
        let tray = SystemTray::new();

        // Check Docker availability
        docker.check_docker();

        // Start resource monitoring
        monitor.start();

        // Initial port scan
        let port_infos = if let Some(project) = config.active_project() {
            PortScanner::scan_project_ports(&project.services)
        } else {
            PortScanner::get_common_ports()
        };

        Self {
            config,
            docker,
            monitor,
            terminal,
            tray,
            active_tab: Tab::Dashboard,
            terminal_input: String::new(),
            new_project_name: String::new(),
            port_infos,
            sys_stats: SystemStats::default(),
            container_stats: Vec::new(),
            cpu_history: vec![0.0; 60],
            mem_history: vec![0.0; 60],
            docker_available: false,
            tray_initialized: false,
            _last_refresh: Instant::now(),
            last_container_refresh: Instant::now(),
        }
    }

    fn process_docker_events(&mut self) {
        while let Ok(event) = self.docker.event_rx.try_recv() {
            match event {
                DockerEvent::DockerAvailable(available) => {
                    self.docker_available = available;
                }
                DockerEvent::StatusChange(_, _status) => {}
                DockerEvent::Log(_) => {}
                DockerEvent::ContainerList(_) => {}
                DockerEvent::Error(_) => {}
            }
        }
    }

    fn process_monitor_events(&mut self) {
        while let Ok(event) = self.monitor.event_rx.try_recv() {
            match event {
                MonitorEvent::SystemUpdate(stats) => {
                    self.sys_stats = stats;
                    self.cpu_history = self.monitor.cpu_history.lock().unwrap().clone();
                    self.mem_history = self.monitor.mem_history.lock().unwrap().clone();
                }
                MonitorEvent::ContainerUpdate(stats) => {
                    self.container_stats = stats;
                }
            }
        }
    }

    fn process_terminal_events(&mut self) {
        while let Ok(_event) = self.terminal.event_rx.try_recv() {
            // Events are already stored in terminal.output_lines
        }
    }

    fn process_tray_events(&mut self) {
        while let Ok(cmd) = self.tray.command_rx.try_recv() {
            match cmd {
                TrayCommand::Start => {
                    if let Some(project) = self.config.active_project() {
                        self.docker.start_services(project);
                    }
                }
                TrayCommand::Stop => {
                    if let Some(project) = self.config.active_project() {
                        self.docker.stop_services(project);
                    }
                }
                TrayCommand::Restart => {
                    if let Some(project) = self.config.active_project() {
                        self.docker.restart_services(project);
                    }
                }
                TrayCommand::OpenUI => {
                    // Window focus is handled by the framework
                }
                TrayCommand::Quit => {
                    std::process::exit(0);
                }
            }
        }
    }

    fn render_top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(theme::COLOR_BG_PANEL)
            .inner_margin(egui::Margin::symmetric(24, 12))
            .stroke(egui::Stroke::new(1.0, theme::COLOR_BORDER))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Title based on active tab
                    let title = match self.active_tab {
                        Tab::Dashboard => "Dashboard",
                        Tab::Services => "Services Stack",
                        Tab::Containers => "Containers",
                        Tab::Logs => "System Logs",
                        Tab::Terminal => "Terminal",
                        Tab::Ports => "Port Scanner",
                        Tab::Monitor => "Resource Monitor",
                        Tab::Settings => "Settings",
                    };
                    
                    ui.label(
                        egui::RichText::new(title)
                            .size(18.0)
                            .strong()
                            .color(theme::COLOR_TEXT)
                    );

                    // Global Actions (Right aligned)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let status = self.docker.status.lock().unwrap().clone();
                        let can_start = matches!(status, ServiceStatus::Stopped | ServiceStatus::Error(_));
                        let can_stop = matches!(status, ServiceStatus::Running);

                        ui.add_enabled_ui(can_stop, |ui| {
                            if ui.button(egui::RichText::new("⏹ Stop All").color(theme::COLOR_ERROR)).clicked() {
                                if let Some(project) = self.config.active_project() {
                                    self.docker.stop_services(project);
                                }
                            }
                        });
                        
                        ui.add_space(8.0);

                        ui.add_enabled_ui(can_stop, |ui| {
                            if ui.button(egui::RichText::new("🔄 Restart").color(theme::COLOR_WARNING)).clicked() {
                                if let Some(project) = self.config.active_project() {
                                    self.docker.restart_services(project);
                                }
                            }
                        });

                        ui.add_space(8.0);

                        ui.add_enabled_ui(can_start, |ui| {
                            let btn = egui::Button::new(
                                egui::RichText::new("▶ Start All")
                                    .color(theme::COLOR_BG_APP) // Text on primary
                                    .strong()
                            )
                            .fill(theme::COLOR_SUCCESS);
                            
                            if ui.add(btn).clicked() {
                                if let Some(project) = self.config.active_project() {
                                    self.docker.start_services(project);
                                }
                            }
                        });
                    });
                });
            });
    }
}

impl eframe::App for DockStackApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaint for animations and monitoring
        ctx.request_repaint_after(std::time::Duration::from_millis(250));

        // Process events
        self.process_docker_events();
        self.process_monitor_events();
        self.process_terminal_events();
        self.process_tray_events();

        // Init tray (only once)
        if !self.tray_initialized {
            log::warn!("System tray temporarily disabled for debugging");
            self.tray_initialized = true;
        }

        // Periodic container refresh
        if self.last_container_refresh.elapsed().as_secs() >= 3 {
             if let Some(project) = self.config.active_project() {
                self.docker.refresh_containers(project);
            }
            self.last_container_refresh = Instant::now();
        }

        // Top panel with action buttons
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            self.render_top_bar(ui);
        });
        
        // Bottom status bar (Minimal)
        egui::TopBottomPanel::bottom("status_bar")
            .max_height(24.0)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(theme::COLOR_BG_PANEL)
                    .inner_margin(egui::Margin::symmetric(12, 2))
                    .show(ui, |ui| {
                         ui.horizontal(|ui| {
                             ui.label(egui::RichText::new("DockStack v0.1.0").size(10.0).color(theme::COLOR_TEXT_DIM));
                             ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                 ui.label(egui::RichText::new(format!("CPU: {:.0}%", self.sys_stats.cpu_usage)).size(10.0).color(theme::COLOR_TEXT_DIM));
                             });
                         });
                    });
            });

        // Left sidebar
        egui::SidePanel::left("sidebar")
            .exact_width(240.0) // Wider sidebar
            .resizable(false)
            .frame(egui::Frame::new()
                .fill(theme::COLOR_BG_PANEL)
                .inner_margin(egui::Margin::symmetric(0, 0))) // Full width
            .show(ctx, |ui| {
                let status = self.docker.status.lock().unwrap().clone();
                // Pass mutable config now
                panels::render_sidebar(ui, &mut self.active_tab, &mut self.config, &status);
            });


        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::COLOR_BG_APP)
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    let containers = self.docker.containers.lock().unwrap().clone();
                    let logs = self.docker.logs.lock().unwrap().clone();

                    match self.active_tab {
                        Tab::Dashboard => {
                            panels::render_dashboard(
                                ui,
                                &self.config,
                                &self.docker.status.lock().unwrap().clone(),
                                &self.sys_stats,
                                &containers,
                                self.docker_available,
                            );
                        }
                        Tab::Services => {
                            panels::render_services(ui, &mut self.config, &containers);
                        }
                        Tab::Containers => {
                            panels::render_containers(ui, &containers);
                        }
                        Tab::Logs => {
                            let mut clear = false;
                            panels::render_logs(ui, &logs, &mut clear);
                            if clear {
                                self.docker.clear_logs();
                            }
                        }
                        Tab::Terminal => {
                            let term_lines = self.terminal.output_lines.lock().unwrap().clone();
                            let mut send = false;
                            let mut clear = false;
                            let mut start = false;
                            let term_running = self.terminal.is_running();

                            panels::render_terminal(
                                ui,
                                &term_lines,
                                &mut self.terminal_input,
                                &mut send,
                                &mut clear,
                                &mut start,
                                term_running,
                            );

                            if start && !term_running {
                                self.terminal.start();
                            }
                            if send && !self.terminal_input.is_empty() {
                                let input = self.terminal_input.clone();
                                self.terminal.send_input(&input);
                                self.terminal_input.clear();
                            }
                            if clear {
                                self.terminal.clear();
                            }
                        }
                        Tab::Ports => {
                            let mut scan = false;
                            panels::render_ports(ui, &self.port_infos, &mut scan);
                            if scan {
                                if let Some(project) = self.config.active_project() {
                                    self.port_infos =
                                        PortScanner::scan_project_ports(&project.services);
                                } else {
                                    self.port_infos = PortScanner::get_common_ports();
                                }
                            }
                        }
                        Tab::Monitor => {
                            panels::render_monitor(
                                ui,
                                &self.sys_stats,
                                &self.container_stats,
                                &self.cpu_history,
                                &self.mem_history,
                            );
                        }
                        Tab::Settings => {
                            let mut gen_ssl = false;
                            let mut rem_ssl = false;
                            panels::render_settings(
                                ui,
                                &mut self.config,
                                &mut self.new_project_name,
                                &mut gen_ssl,
                                &mut rem_ssl,
                            );

                            if gen_ssl {
                                if let Some(project) = self.config.active_project() {
                                    match SslManager::generate_self_signed(&project.directory) {
                                        Ok((cert, key)) => {
                                            log::info!("SSL cert generated: {}, {}", cert, key);
                                        }
                                        Err(e) => {
                                            log::error!("SSL generation failed: {}", e);
                                        }
                                    }
                                }
                            }
                            if rem_ssl {
                                if let Some(project) = self.config.active_project() {
                                    if let Err(e) = SslManager::remove_certs(&project.directory) {
                                        log::error!("SSL removal failed: {}", e);
                                    }
                                }
                            }
                        }
                    }
                });
        });
    }
}
