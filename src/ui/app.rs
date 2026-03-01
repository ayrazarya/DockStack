use eframe::egui::{self, RichText, ScrollArea, Vec2};
use std::time::Instant;

use crate::config::AppConfig;
use crate::docker::manager::{DockerEvent, DockerManager, ServiceStatus};
use crate::monitor::{ContainerStats, MonitorEvent, ResourceMonitor, SystemStats};
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
    cpu_history: std::collections::VecDeque<f32>,
    mem_history: std::collections::VecDeque<f32>,

    // Flags
    docker_available: bool,
    tray_initialized: bool,
    _last_refresh: Instant,
    last_container_refresh: Instant,
}

impl DockStackApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
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
            cpu_history: std::collections::VecDeque::from(vec![0.0; 60]),
            mem_history: std::collections::VecDeque::from(vec![0.0; 60]),
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
                DockerEvent::ContainerList(_list) => {
                    // Update our monitor stats and analytic history
                    // The main container list is already updated via Mutex in DockerManager,
                    // but we sync it here to trigger UI updates if necessary.
                }
                DockerEvent::Error(e) => {
                    log::error!("Docker error: {}", e);
                }
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

    fn process_tray_events(&mut self, ctx: &egui::Context) {
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
                    log::info!("Quit requested from system tray, initiating graceful shutdown...");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Title based on active tab
            let (icon, title) = match self.active_tab {
                Tab::Dashboard => ("🏠", "System Overview"),
                Tab::Services => ("📦", "Service Stack"),
                Tab::Containers => ("🐳", "Docker Containers"),
                Tab::Logs => ("📋", "System Logs"),
                Tab::Terminal => ("💻", "Interactive Console"),
                Tab::Ports => ("🔌", "Port Checker"),
                Tab::Monitor => ("📊", "Live Analytics"),
                Tab::Settings => ("⚙️", "Settings"),
            };
            ui.horizontal(|ui| {
                ui.add(
                    egui::Image::new(egui::include_image!("../../assets/images/icon.png"))
                        .max_size(Vec2::new(32.0, 32.0))
                        .corner_radius(8.0),
                );
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(icon).size(20.0));
                        ui.label(
                            egui::RichText::new(title)
                                .size(24.0)
                                .strong()
                                .color(theme::COLOR_TEXT),
                        );
                    });
                    ui.label(
                        RichText::new("Manage your containerized dev environment with ease")
                            .size(12.0)
                            .color(theme::COLOR_TEXT_DIM),
                    );
                });
            });

            // Global Actions (Right aligned)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let status = self.docker.status.lock().unwrap().clone();
                let can_start = matches!(status, ServiceStatus::Stopped | ServiceStatus::Error(_));
                let can_stop = matches!(status, ServiceStatus::Running);

                // Start All Button - More Prominent
                ui.add_enabled_ui(can_start, |ui| {
                    let btn = egui::Button::new(
                        egui::RichText::new("▶  Power Up Stack")
                            .color(theme::COLOR_BG_APP)
                            .strong(),
                    )
                    .fill(theme::COLOR_SUCCESS)
                    .corner_radius(egui::CornerRadius::same(10))
                    .min_size(Vec2::new(140.0, 42.0));

                    if ui.add(btn).clicked() {
                        if let Some(project) = self.config.active_project() {
                            self.docker.start_services(project);
                        }
                    }
                });

                ui.add_space(12.0);

                // Restart/Stop Buttons - Ghost style
                ui.add_enabled_ui(can_stop, |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("🔄 Restart").color(theme::COLOR_WARNING),
                            )
                            .frame(true)
                            .stroke(egui::Stroke::new(1.0, theme::COLOR_BORDER))
                            .min_size(Vec2::new(100.0, 42.0)),
                        )
                        .clicked()
                    {
                        if let Some(project) = self.config.active_project() {
                            self.docker.restart_services(project);
                        }
                    }
                });

                ui.add_space(8.0);

                ui.add_enabled_ui(can_stop, |ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("⏹ Stop").color(theme::COLOR_ERROR))
                                .frame(true)
                                .stroke(egui::Stroke::new(1.0, theme::COLOR_BORDER))
                                .min_size(Vec2::new(80.0, 42.0)),
                        )
                        .clicked()
                    {
                        if let Some(project) = self.config.active_project() {
                            self.docker.stop_services(project);
                        }
                    }
                });
            });
        });
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);
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
        self.process_tray_events(ctx);

        // Init tray (only once)
        if !self.tray_initialized {
            if let Err(e) = self.tray.setup() {
                log::error!("Failed to initialize system tray: {}", e);
            }
            self.tray_initialized = true;
        }

        // Periodic container refresh
        if self.last_container_refresh.elapsed().as_secs() >= 3 {
            if let Some(project) = self.config.active_project() {
                self.docker.refresh_containers(project);
            }
            self.last_container_refresh = Instant::now();
        }

        // Bottom status bar (integrated with background)
        egui::TopBottomPanel::bottom("status_bar")
            .max_height(32.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::COLOR_BG_APP)
                    .inner_margin(egui::Margin::symmetric(16, 4)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("DockStack Native")
                            .size(11.0)
                            .color(theme::COLOR_TEXT_MUTED),
                    );
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("Docker Engine: Online")
                            .size(11.0)
                            .color(theme::COLOR_SUCCESS),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "MEM: {:.1} GB",
                                self.sys_stats.memory_used as f32 / 1024.0 / 1024.0 / 1024.0
                            ))
                            .size(11.0)
                            .color(theme::COLOR_TEXT_DIM),
                        );
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new(format!("CPU: {:.1}%", self.sys_stats.cpu_usage))
                                .size(11.0)
                                .color(theme::COLOR_TEXT_DIM),
                        );
                    });
                });
            });

        // Permanent Slim Sidebar
        egui::SidePanel::left("sidebar")
            .exact_width(220.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::COLOR_BG_PANEL)
                    .stroke(egui::Stroke::NONE) // Remove stroke
                    .inner_margin(egui::Margin::symmetric(12, 0)),
            )
            .show(ctx, |ui| {
                let status = self.docker.status.lock().unwrap().clone();
                panels::render_sidebar(ui, &mut self.active_tab, &mut self.config, &status);
            });

        // Modern Central Panel
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::COLOR_BG_APP))
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        egui::Frame::new()
                            .inner_margin(egui::Margin::symmetric(32, 24)) // Keep inner margin for content
                            .stroke(egui::Stroke::NONE) // Remove stroke from this frame
                            .show(ui, |ui| {
                                // Integrated Header
                                self.render_header(ui);

                                match self.active_tab {
                                    Tab::Dashboard => {
                                        let status = self.docker.status.lock().unwrap().clone();
                                        panels::render_dashboard(
                                            ui,
                                            &mut self.config,
                                            &status,
                                            &self.sys_stats,
                                            &self.docker.containers.lock().unwrap(),
                                            self.docker_available,
                                        );
                                    }

                                    Tab::Services => {
                                        panels::render_services(
                                            ui,
                                            &mut self.config,
                                            &self.docker.containers.lock().unwrap(),
                                        );
                                    }
                                    Tab::Containers => {
                                        panels::render_containers(
                                            ui,
                                            &self.docker.containers.lock().unwrap(),
                                        );
                                    }
                                    Tab::Logs => {
                                        let mut clear = false;
                                        let mut logs_guard = self.docker.logs.lock().unwrap();
                                        panels::render_logs(
                                            ui,
                                            logs_guard.make_contiguous(),
                                            &mut clear,
                                        );
                                        if clear {
                                            logs_guard.clear();
                                        }
                                    }
                                    Tab::Terminal => {
                                        let mut term_lines_guard =
                                            self.terminal.output_lines.lock().unwrap();
                                        let term_lines = term_lines_guard.make_contiguous();
                                        let mut send = false;
                                        let mut clear = false;
                                        let mut start = false;
                                        let term_running = self.terminal.is_running();

                                        panels::render_terminal(
                                            ui,
                                            term_lines,
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
                                            term_lines_guard.clear();
                                        }
                                    }
                                    Tab::Ports => {
                                        let mut scan = false;
                                        panels::render_ports(ui, &self.port_infos, &mut scan);
                                        if scan {
                                            if let Some(project) = self.config.active_project() {
                                                self.port_infos = PortScanner::scan_project_ports(
                                                    &project.services,
                                                );
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
                                            self.cpu_history.make_contiguous(),
                                            self.mem_history.make_contiguous(),
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
                                                match SslManager::generate_self_signed(
                                                    &project.directory,
                                                ) {
                                                    Ok((cert, key)) => {
                                                        log::info!(
                                                            "SSL cert generated: {}, {}",
                                                            cert,
                                                            key
                                                        );
                                                    }
                                                    Err(e) => {
                                                        log::error!("SSL generation failed: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                        if rem_ssl {
                                            if let Some(project) = self.config.active_project() {
                                                if let Err(e) =
                                                    SslManager::remove_certs(&project.directory)
                                                {
                                                    log::error!("SSL removal failed: {}", e);
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                    });
            });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        log::info!("DockStack shutting down gracefully...");

        self.monitor.stop();
        self.terminal.stop();
        self.docker.wait_all();

        // Save current configuration to disk
        log::info!("Saving configuration...");
        self.config.save();

        // Stop running Docker containers if services are active
        let status = self.docker.status.lock().unwrap().clone();
        if matches!(status, ServiceStatus::Running | ServiceStatus::Starting) {
            log::info!("Stopping running Docker containers...");
            if let Some(project) = self.config.active_project() {
                self.docker.stop_services_sync(project);
            }
        }

        log::info!("DockStack shutdown complete.");
    }
}
