#![allow(dead_code)]
use crate::config::ProjectConfig;
use crate::docker::compose;
use crossbeam_channel::{Receiver, Sender};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::thread;

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub ports: String,
    pub state: String,
}

#[derive(Debug, Clone)]
pub enum DockerEvent {
    Log(String),
    StatusChange(String, ServiceStatus),
    ContainerList(Vec<ContainerInfo>),
    Error(String),
    DockerAvailable(bool),
}

pub struct DockerManager {
    pub event_tx: Sender<DockerEvent>,
    pub event_rx: Receiver<DockerEvent>,
    pub status: Arc<Mutex<ServiceStatus>>,
    pub logs: Arc<Mutex<VecDeque<String>>>,
    pub containers: Arc<Mutex<Vec<ContainerInfo>>>,
    pub docker_available: Arc<Mutex<bool>>,
    pub use_compose_plugin: Arc<Mutex<bool>>,
}

impl DockerManager {
    pub fn new() -> Self {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        Self {
            event_tx,
            event_rx,
            status: Arc::new(Mutex::new(ServiceStatus::Stopped)),
            logs: Arc::new(Mutex::new(VecDeque::new())),
            containers: Arc::new(Mutex::new(Vec::new())),
            docker_available: Arc::new(Mutex::new(false)),
            use_compose_plugin: Arc::new(Mutex::new(false)),
        }
    }

    pub fn check_docker(&self) {
        let tx = self.event_tx.clone();
        let available = self.docker_available.clone();
        let plugin = self.use_compose_plugin.clone();
        
        thread::spawn(move || {
            let result = Command::new("docker").arg("info").output();
            let is_available = result.map(|o| o.status.success()).unwrap_or(false);
            *available.lock().unwrap() = is_available;
            
            let mut has_compose = false;
            if let Ok(output) = std::process::Command::new("docker")
                .arg("compose")
                .arg("version")
                .output()
            {
                if output.status.success() {
                    has_compose = true;
                }
            }
            *plugin.lock().unwrap() = has_compose;

            tx.send(DockerEvent::DockerAvailable(is_available)).ok();
        });
    }

    pub fn start_services(&self, project: &ProjectConfig) {
        let enabled_count = project.services.values().filter(|s| s.enabled).count();
        if enabled_count == 0 {
            let msg = "No services enabled! Please enable at least one service in the Services tab.".to_string();
            *self.status.lock().unwrap() = ServiceStatus::Error(msg.clone());
            let tx = self.event_tx.clone();
            tx.send(DockerEvent::Error(msg)).ok();
            return;
        }

        let project = project.clone();
        let tx = self.event_tx.clone();
        let status = self.status.clone();
        let logs = self.logs.clone();

        *status.lock().unwrap() = ServiceStatus::Starting;
        tx.send(DockerEvent::StatusChange(
            "all".to_string(),
            ServiceStatus::Starting,
        ))
        .ok();

        let use_compose_plugin = self.use_compose_plugin.clone();

        thread::spawn(move || {
            // Generate and write compose file
            match compose::write_compose_file(&project) {
                Ok(compose_path) => {
                    let msg = format!("[DockStack] Compose file written: {}", compose_path);
                    logs.lock().unwrap().push_back(msg.clone());
                    tx.send(DockerEvent::Log(msg)).ok();
                }
                Err(e) => {
                    let msg = format!("[DockStack] Error writing compose file: {}", e);
                    *status.lock().unwrap() = ServiceStatus::Error(e.to_string());
                    tx.send(DockerEvent::Error(msg)).ok();
                    return;
                }
            }

            let msg = "[DockStack] Starting services...".to_string();
            logs.lock().unwrap().push_back(msg.clone());
            tx.send(DockerEvent::Log(msg)).ok();

            // Determine compose command
            let use_plugin = *use_compose_plugin.lock().unwrap();
            let (program, args) = if use_plugin {
                ("docker", vec!["compose", "up", "-d", "--remove-orphans"])
            } else {
                ("docker-compose", vec!["up", "-d", "--remove-orphans"])
            };
            
            let mut cmd = Command::new(program);
            cmd.args(&args)
                .current_dir(&project.directory)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    let mut stderr_content = String::new();
                    
                    // Read stderr
                    if let Some(stderr) = child.stderr.take() {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines().map_while(Result::ok) {
                            stderr_content.push_str(&line);
                            stderr_content.push('\n');
                            logs.lock().unwrap().push_back(line.clone());
                            tx.send(DockerEvent::Log(line)).ok();
                        }
                    }

                    match child.wait() {
                        Ok(exit) => {
                            if exit.success() {
                                *status.lock().unwrap() = ServiceStatus::Running;
                                let msg = "[DockStack] Services started successfully".to_string();
                                logs.lock().unwrap().push_back(msg.clone());
                                tx.send(DockerEvent::Log(msg)).ok();
                                tx.send(DockerEvent::StatusChange(
                                    "all".to_string(),
                                    ServiceStatus::Running,
                                ))
                                .ok();
                            } else {
                                let error_detail = if !stderr_content.trim().is_empty() {
                                    stderr_content.trim().to_string()
                                } else {
                                    format!("Exit code: {}", exit)
                                };
                                
                                let combined_log = format!(
                                    "[DockStack] Failed to start services: {}\nCommand tried: {} {:?}",
                                    error_detail, program, args
                                );
                                
                                log::error!("{}", combined_log);
                                logs.lock().unwrap().push_back(combined_log.clone());
                                tx.send(DockerEvent::Log(combined_log)).ok(); // Send to logs tab

                                let short_msg = "Failed to start. Check Logs tab.".to_string();
                                *status.lock().unwrap() = ServiceStatus::Error(short_msg.clone());
                                tx.send(DockerEvent::Error(short_msg)).ok(); // Status update
                            }
                        }
                        Err(e) => {
                            let msg = format!("[DockStack] Failed to wait for docker process: {}", e);
                            log::error!("{}", msg);
                             logs.lock().unwrap().push_back(msg.clone());
                            *status.lock().unwrap() = ServiceStatus::Error("Process error. Check Logs.".to_string());
                             tx.send(DockerEvent::Error(msg)).ok();
                        }
                    }
                }
                Err(e) => {
                    let msg = format!(
                        "[DockStack] Failed to execute docker compose command ({}): {}", 
                        program, e
                    );
                    log::error!("{}", msg);
                    logs.lock().unwrap().push_back(msg.clone());
                    *status.lock().unwrap() = ServiceStatus::Error("Exec error. Check Logs.".to_string());
                    tx.send(DockerEvent::Error(msg)).ok();
                }
            }
        });
    }

    pub fn stop_services(&self, project: &ProjectConfig) {
        let project = project.clone();
        let tx = self.event_tx.clone();
        let status = self.status.clone();
        let logs = self.logs.clone();

        *status.lock().unwrap() = ServiceStatus::Stopping;
        tx.send(DockerEvent::StatusChange(
            "all".to_string(),
            ServiceStatus::Stopping,
        ))
        .ok();

        let use_compose_plugin = self.use_compose_plugin.clone();

        thread::spawn(move || {
            let msg = "[DockStack] Stopping services...".to_string();
            logs.lock().unwrap().push_back(msg.clone());
            tx.send(DockerEvent::Log(msg)).ok();

            // Detect compose
            let use_plugin = *use_compose_plugin.lock().unwrap();
            let (prog, args) = if use_plugin {
                ("docker", vec!["compose", "down"])
            } else {
                ("docker-compose", vec!["down"])
            };

            let mut cmd = Command::new(prog);
            cmd.args(&args)
                .current_dir(&project.directory)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    if let Some(stderr) = child.stderr.take() {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines().map_while(Result::ok) {
                            logs.lock().unwrap().push_back(line.clone());
                            tx.send(DockerEvent::Log(line)).ok();
                        }
                    }

                    match child.wait() {
                        Ok(exit) => {
                            if exit.success() {
                                *status.lock().unwrap() = ServiceStatus::Stopped;
                                let msg = "[DockStack] Services stopped".to_string();
                                logs.lock().unwrap().push_back(msg.clone());
                                tx.send(DockerEvent::Log(msg)).ok();
                                tx.send(DockerEvent::StatusChange(
                                    "all".to_string(),
                                    ServiceStatus::Stopped,
                                ))
                                .ok();
                            } else {
                                let msg = format!("[DockStack] docker compose down failed: {}", exit);
                                *status.lock().unwrap() = ServiceStatus::Error(msg.clone());
                                tx.send(DockerEvent::Error(msg)).ok();
                            }
                        }
                        Err(e) => {
                            let msg = format!("[DockStack] Wait error: {}", e);
                            *status.lock().unwrap() = ServiceStatus::Error(msg.clone());
                            tx.send(DockerEvent::Error(msg)).ok();
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("[DockStack] Failed to stop docker compose: {}", e);
                    *status.lock().unwrap() = ServiceStatus::Error(msg.clone());
                    tx.send(DockerEvent::Error(msg)).ok();
                }
            }
        });
    }

    pub fn stop_services_sync(&self, project: &ProjectConfig) {
        let msg = "[DockStack] Stopping services before exit...".to_string();
        self.logs.lock().unwrap().push_back(msg.clone());
        self.event_tx.send(DockerEvent::Log(msg)).ok();
        
        let use_plugin = *self.use_compose_plugin.lock().unwrap();
        let (prog, args) = if use_plugin {
            ("docker", vec!["compose", "down"])
        } else {
            ("docker-compose", vec!["down"])
        };

        let mut cmd = Command::new(prog);
        cmd.args(&args)
            .current_dir(&project.directory)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
            
        let _ = cmd.status();
    }

    pub fn restart_services(&self, project: &ProjectConfig) {
        let project = project.clone();
        let tx = self.event_tx.clone();
        let status = self.status.clone();
        let logs = self.logs.clone();

        *status.lock().unwrap() = ServiceStatus::Stopping;

        let use_compose_plugin = self.use_compose_plugin.clone();

        thread::spawn(move || {
            let msg = "[DockStack] Restarting services...".to_string();
            logs.lock().unwrap().push_back(msg.clone());
            tx.send(DockerEvent::Log(msg)).ok();

            // Detect compose
            let use_plugin = *use_compose_plugin.lock().unwrap();
            // Stop
            let (prog_down, args_down) = if use_plugin {
                 ("docker", vec!["compose", "down"])
            } else {
                 ("docker-compose", vec!["down"])
            };

            let stop = Command::new(prog_down)
                .args(&args_down)
                .current_dir(&project.directory)
                .output();

            if let Err(e) = stop {
                let msg = format!("[DockStack] Stop failed during restart: {}", e);
                tx.send(DockerEvent::Error(msg)).ok();
                return;
            }

            // Regenerate compose
            if let Err(e) = compose::write_compose_file(&project) {
                let msg = format!("[DockStack] Error writing compose file: {}", e);
                tx.send(DockerEvent::Error(msg)).ok();
                return;
            }

            // Start
            *status.lock().unwrap() = ServiceStatus::Starting;
            
            let (prog_up, args_up) = if use_plugin {
                 ("docker", vec!["compose", "up", "-d", "--remove-orphans"])
            } else {
                 ("docker-compose", vec!["up", "-d", "--remove-orphans"])
            };

            let start = Command::new(prog_up)
                .args(&args_up)
                .current_dir(&project.directory)
                .output();

            match start {
                Ok(output) => {
                    if output.status.success() {
                        *status.lock().unwrap() = ServiceStatus::Running;
                        let msg = "[DockStack] Services restarted successfully".to_string();
                        logs.lock().unwrap().push_back(msg.clone());
                        tx.send(DockerEvent::Log(msg)).ok();
                        tx.send(DockerEvent::StatusChange(
                            "all".to_string(),
                            ServiceStatus::Running,
                        ))
                        .ok();
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let msg = format!("[DockStack] Restart failed: {}", stderr);
                        *status.lock().unwrap() = ServiceStatus::Error(msg.clone());
                        tx.send(DockerEvent::Error(msg)).ok();
                    }
                }
                Err(e) => {
                    let msg = format!("[DockStack] Restart failed: {}", e);
                    *status.lock().unwrap() = ServiceStatus::Error(msg.clone());
                    tx.send(DockerEvent::Error(msg)).ok();
                }
            }
        });
    }

    pub fn refresh_containers(&self, project: &ProjectConfig) {
        let project_id = project.id.clone();
        let tx = self.event_tx.clone();
        let containers = self.containers.clone();

        thread::spawn(move || {
            // Using docker ps with filter is more reliable than docker compose ps
            // across different versions and environments.
            let output = Command::new("docker")
                .arg("ps")
                .arg("-a")
                .arg("--filter")
                .arg(format!("label=com.docker.compose.project={}", project_id))
                .arg("--format")
                .arg("{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}|{{.State}}")
                .output();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let list: Vec<ContainerInfo> = stdout
                        .lines()
                        .filter(|l| !l.is_empty())
                        .map(|line| {
                            let parts: Vec<&str> = line.split('|').collect();
                            ContainerInfo {
                                id: parts.first().unwrap_or(&"").to_string(),
                                name: parts.get(1).unwrap_or(&"").to_string(),
                                image: parts.get(2).unwrap_or(&"").to_string(),
                                status: parts.get(3).unwrap_or(&"").to_string(),
                                ports: parts.get(4).unwrap_or(&"").to_string(),
                                state: parts.get(5).unwrap_or(&"").to_string(),
                            }
                        })
                        .collect();

                    *containers.lock().unwrap() = list.clone();
                    tx.send(DockerEvent::ContainerList(list)).ok();
                }
                Err(e) => {
                    tx.send(DockerEvent::Error(format!("Failed to list containers: {}", e)))
                        .ok();
                }
            }
        });
    }

    pub fn stream_logs(&self, project: &ProjectConfig) {
        let project = project.clone();
        let tx = self.event_tx.clone();
        let logs = self.logs.clone();

        let use_compose_plugin = self.use_compose_plugin.clone();

        thread::spawn(move || {
            // Detect compose
            let use_plugin = *use_compose_plugin.lock().unwrap();
            let (prog, args) = if use_plugin {
                ("docker", vec!["compose", "logs", "-f", "--tail", "100"])
            } else {
                ("docker-compose", vec!["logs", "-f", "--tail", "100"])
            };

            let mut cmd = Command::new(prog);
            cmd.args(&args)
                .current_dir(&project.directory)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().map_while(Result::ok) {
                            logs.lock().unwrap().push_back(line.clone());
                            // Keep log buffer limited
                            {
                                let mut l = logs.lock().unwrap();
                                if l.len() > 5000 {
                                    let drain_count = l.len() - 3000;
                                    l.drain(0..drain_count);
                                }
                            }
                            tx.send(DockerEvent::Log(line)).ok();
                        }
                    }
                    child.wait().ok(); // Avoid zombie process
                }
                Err(e) => {
                    tx.send(DockerEvent::Error(format!("Failed to stream logs: {}", e)))
                        .ok();
                }
            }
        });
    }

    pub fn clear_logs(&self) {
        self.logs.lock().unwrap().clear();
    }
}
