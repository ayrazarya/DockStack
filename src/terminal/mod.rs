use crossbeam_channel::{Receiver, Sender};
use portable_pty::{native_pty_system, Child, CommandBuilder, PtySize};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TerminalEvent {
    Output(String),
    Error(String),
    Exited(i32),
}

pub struct EmbeddedTerminal {
    pub output_lines: Arc<Mutex<VecDeque<String>>>,
    pub event_tx: Sender<TerminalEvent>,
    pub event_rx: Receiver<TerminalEvent>,
    master_writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    running: Arc<Mutex<bool>>,
}

impl EmbeddedTerminal {
    pub fn new() -> Self {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        Self {
            output_lines: Arc::new(Mutex::new(VecDeque::new())),
            event_tx,
            event_rx,
            master_writer: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&self) {
        let tx = self.event_tx.clone();
        let output_lines = self.output_lines.clone();
        let master_writer = self.master_writer.clone();
        let running = self.running.clone();

        *running.lock().unwrap_or_else(|e| e.into_inner()) = true;

        thread::spawn(move || {
            let pty_system = native_pty_system();

            let pair = match pty_system.openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            }) {
                Ok(p) => p,
                Err(e) => {
                    *running.lock().unwrap_or_else(|e| e.into_inner()) = false;
                    tx.send(TerminalEvent::Error(format!("Failed to open PTY: {}", e)))
                        .ok();
                    return;
                }
            };

            let shell = if cfg!(target_os = "windows") {
                "powershell.exe"
            } else {
                "bash"
            };
            let mut cmd = CommandBuilder::new(shell);
            if !cfg!(target_os = "windows") {
                cmd.arg("-i"); // Interactive
            }

            let mut child: Box<dyn Child + Send> = match pair.slave.spawn_command(cmd) {
                Ok(c) => c,
                Err(e) => {
                    *running.lock().unwrap_or_else(|e| e.into_inner()) = false;
                    tx.send(TerminalEvent::Error(format!(
                        "Failed to spawn shell: {}",
                        e
                    )))
                    .ok();
                    return;
                }
            };

            // Drop slave side in this process as it's now owned by the child
            drop(pair.slave);

            // Set up writer
            let writer = pair.master.take_writer().unwrap();
            *master_writer.lock().unwrap_or_else(|e| e.into_inner()) = Some(writer);

            // Reader thread
            let mut reader = pair.master.try_clone_reader().unwrap();
            let tx_out = tx.clone();
            let lines_out = output_lines.clone();
            let running_out = running.clone();

            thread::spawn(move || {
                let mut buffer = [0u8; 4096];
                let mut line_buffer = String::new();
                loop {
                    if !*running_out.lock().unwrap_or_else(|e| e.into_inner()) {
                        break;
                    }
                    match reader.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            let data = String::from_utf8_lossy(&buffer[..n]);
                            line_buffer.push_str(&data);

                            if line_buffer.contains('\n') {
                                let mut lines: Vec<String> =
                                    line_buffer.split('\n').map(|s| s.to_string()).collect();
                                line_buffer = lines.pop().unwrap_or_default(); // Keep the last partial line

                                let mut l = lines_out.lock().unwrap_or_else(|e| e.into_inner());
                                for line in lines {
                                    let cleaned = line.replace("\r", "");
                                    if !cleaned.trim().is_empty() || line.len() > 2 {
                                        l.push_back(cleaned);
                                    }
                                }

                                if l.len() > 1000 {
                                    let drain = l.len() - 800;
                                    l.drain(0..drain);
                                }
                            }
                            tx_out.send(TerminalEvent::Output(data.to_string())).ok();
                        }
                        Err(_) => break,
                    }
                }
            });

            // Wait for exit
            match child.wait() {
                Ok(_status) => {
                    *running.lock().unwrap_or_else(|e| e.into_inner()) = false;
                    tx.send(TerminalEvent::Exited(0)).ok();
                }
                Err(e) => {
                    *running.lock().unwrap_or_else(|e| e.into_inner()) = false;
                    tx.send(TerminalEvent::Error(format!(
                        "Shell exited with error: {}",
                        e
                    )))
                    .ok();
                }
            }
        });
    }

    pub fn send_input(&self, input: &str) {
        if let Some(ref mut writer) = *self.master_writer.lock().unwrap_or_else(|e| e.into_inner()) {
            let data = if input.ends_with('\n') {
                input.to_string()
            } else {
                format!("{}\n", input)
            };
            let _ = writer.write_all(data.as_bytes());
            let _ = writer.flush();
        }
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap_or_else(|e| e.into_inner())
    }
}
