#![allow(dead_code)]
use sysinfo::System;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crossbeam_channel::{Sender, Receiver};

#[derive(Debug, Clone, Default)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_percent: f32,
}

#[derive(Debug, Clone, Default)]
pub struct ContainerStats {
    pub name: String,
    pub cpu_percent: String,
    pub mem_usage: String,
    pub mem_percent: String,
    pub net_io: String,
    pub block_io: String,
}

#[derive(Debug, Clone)]
pub enum MonitorEvent {
    SystemUpdate(SystemStats),
    ContainerUpdate(Vec<ContainerStats>),
}

pub struct ResourceMonitor {
    pub system_stats: Arc<Mutex<SystemStats>>,
    pub container_stats: Arc<Mutex<Vec<ContainerStats>>>,
    pub cpu_history: Arc<Mutex<Vec<f32>>>,
    pub mem_history: Arc<Mutex<Vec<f32>>>,
    pub event_tx: Sender<MonitorEvent>,
    pub event_rx: Receiver<MonitorEvent>,
    running: Arc<Mutex<bool>>,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        Self {
            system_stats: Arc::new(Mutex::new(SystemStats::default())),
            container_stats: Arc::new(Mutex::new(Vec::new())),
            cpu_history: Arc::new(Mutex::new(vec![0.0; 60])),
            mem_history: Arc::new(Mutex::new(vec![0.0; 60])),
            event_tx,
            event_rx,
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&self) {
        let running = self.running.clone();
        {
            let mut r = running.lock().unwrap();
            if *r {
                return;
            }
            *r = true;
        }

        // System stats thread
        let sys_stats = self.system_stats.clone();
        let cpu_history = self.cpu_history.clone();
        let mem_history = self.mem_history.clone();
        let tx = self.event_tx.clone();
        let running_sys = self.running.clone();

        thread::spawn(move || {
            let mut sys = System::new_all();
            while *running_sys.lock().unwrap() {
                sys.refresh_cpu_usage();
                sys.refresh_memory();

                let cpu = sys.global_cpu_usage();
                let mem_used = sys.used_memory();
                let mem_total = sys.total_memory();
                let mem_pct = if mem_total > 0 {
                    (mem_used as f32 / mem_total as f32) * 100.0
                } else {
                    0.0
                };

                let stats = SystemStats {
                    cpu_usage: cpu,
                    memory_used: mem_used,
                    memory_total: mem_total,
                    memory_percent: mem_pct,
                };

                *sys_stats.lock().unwrap() = stats.clone();

                {
                    let mut hist = cpu_history.lock().unwrap();
                    hist.push(cpu);
                    if hist.len() > 60 {
                        hist.remove(0);
                    }
                }
                {
                    let mut hist = mem_history.lock().unwrap();
                    hist.push(mem_pct);
                    if hist.len() > 60 {
                        hist.remove(0);
                    }
                }

                tx.send(MonitorEvent::SystemUpdate(stats)).ok();
                thread::sleep(Duration::from_secs(1));
            }
        });

        // Container stats thread
        let container_stats = self.container_stats.clone();
        let tx2 = self.event_tx.clone();
        let running_cont = self.running.clone();

        thread::spawn(move || {
            while *running_cont.lock().unwrap() {
                let output = Command::new("docker")
                    .args(["stats", "--no-stream", "--format",
                        "{{.Name}}|{{.CPUPerc}}|{{.MemUsage}}|{{.MemPerc}}|{{.NetIO}}|{{.BlockIO}}"])
                    .output();

                if let Ok(out) = output {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stats: Vec<ContainerStats> = stdout
                        .lines()
                        .filter(|l| !l.is_empty())
                        .map(|line| {
                            let parts: Vec<&str> = line.splitn(6, '|').collect();
                            ContainerStats {
                                name: parts.first().unwrap_or(&"").to_string(),
                                cpu_percent: parts.get(1).unwrap_or(&"").to_string(),
                                mem_usage: parts.get(2).unwrap_or(&"").to_string(),
                                mem_percent: parts.get(3).unwrap_or(&"").to_string(),
                                net_io: parts.get(4).unwrap_or(&"").to_string(),
                                block_io: parts.get(5).unwrap_or(&"").to_string(),
                            }
                        })
                        .collect();

                    *container_stats.lock().unwrap() = stats.clone();
                    tx2.send(MonitorEvent::ContainerUpdate(stats)).ok();
                }

                thread::sleep(Duration::from_secs(2));
            }
        });
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}
