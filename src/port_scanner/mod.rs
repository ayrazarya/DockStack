#![allow(dead_code)]
use std::collections::HashMap;
use std::net::TcpListener;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PortInfo {
    pub port: u16,
    pub in_use: bool,
    pub process: String,
}

pub struct PortScanner;

impl PortScanner {
    /// Check if a specific port is available
    pub fn is_port_available(port: u16) -> bool {
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    /// Scan multiple ports and return their status
    pub fn scan_ports(ports: &[u16]) -> Vec<PortInfo> {
        ports
            .iter()
            .map(|&port| {
                let is_available = Self::is_port_available(port);
                PortInfo {
                    port,
                    in_use: !is_available,
                    process: if !is_available {
                        Self::get_process_on_port(port)
                    } else {
                        String::new()
                    },
                }
            })
            .collect()
    }

    /// Find the next available port starting from the given port
    pub fn find_available_port(start: u16) -> u16 {
        let mut port = start;
        while port < 65535 {
            if Self::is_port_available(port) {
                return port;
            }
            port += 1;
        }
        start
    }

    /// Suggest alternative port if the desired port is in use
    pub fn suggest_port(desired: u16) -> (bool, u16) {
        if Self::is_port_available(desired) {
            (true, desired)
        } else {
            (false, Self::find_available_port(desired + 1))
        }
    }

    /// Scan all service ports for a project
    pub fn scan_project_ports(
        services: &HashMap<String, crate::config::ServiceConfig>,
    ) -> Vec<PortInfo> {
        let ports: Vec<u16> = services
            .iter()
            .filter(|(_, v)| v.enabled)
            .map(|(_, v)| v.port)
            .collect();
        Self::scan_ports(&ports)
    }

    /// Get process name using the specified port
    fn get_process_on_port(port: u16) -> String {
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("ss")
                .args(["-tlnp", &format!("sport = :{}", port)])
                .output();
            if let Ok(out) = output {
                let s = String::from_utf8_lossy(&out.stdout);
                // Extract process info
                if let Some(line) = s.lines().nth(1) {
                    return line.to_string();
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("lsof")
                .args(["-i", &format!(":{}", port), "-sTCP:LISTEN"])
                .output();
            if let Ok(out) = output {
                let s = String::from_utf8_lossy(&out.stdout);
                if let Some(line) = s.lines().nth(1) {
                    return line.split_whitespace().next().unwrap_or("").to_string();
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("netstat").args(["-ano", "-p", "TCP"]).output();
            if let Ok(out) = output {
                let s = String::from_utf8_lossy(&out.stdout);
                for line in s.lines() {
                    if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
                        return line.to_string();
                    }
                }
            }
        }

        String::from("unknown")
    }

    /// Get a list of commonly used ports and their status
    pub fn get_common_ports() -> Vec<PortInfo> {
        let ports = vec![80, 443, 3306, 5432, 6379, 8080, 8081, 8082, 8083, 9000];
        Self::scan_ports(&ports)
    }
}
