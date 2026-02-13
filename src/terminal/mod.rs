#![allow(dead_code)]
use crossbeam_channel::{Receiver, Sender};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub enum TerminalEvent {
    Output(String),
    Error(String),
    Exited(i32),
}

pub struct EmbeddedTerminal {
    pub output_lines: Arc<Mutex<Vec<String>>>,
    pub event_tx: Sender<TerminalEvent>,
    pub event_rx: Receiver<TerminalEvent>,
    child_stdin: Arc<Mutex<Option<std::process::ChildStdin>>>,
    running: Arc<Mutex<bool>>,
}

impl EmbeddedTerminal {
    pub fn new() -> Self {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        Self {
            output_lines: Arc::new(Mutex::new(Vec::new())),
            event_tx,
            event_rx,
            child_stdin: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&self) {
        let tx = self.event_tx.clone();
        let output_lines = self.output_lines.clone();
        let child_stdin = self.child_stdin.clone();
        let running = self.running.clone();

        *running.lock().unwrap() = true;

        thread::spawn(move || {
            let shell = if cfg!(target_os = "windows") {
                "cmd"
            } else {
                "/bin/bash"
            };

            let mut cmd = Command::new(shell);
            if !cfg!(target_os = "windows") {
                cmd.arg("-i");
            }
            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            // Set env for non-interactive mode to still get output
            cmd.env("TERM", "dumb");

            match cmd.spawn() {
                Ok(mut child) => {
                    // Store stdin handle
                    if let Some(stdin) = child.stdin.take() {
                        *child_stdin.lock().unwrap() = Some(stdin);
                    }

                    // Read stdout
                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        let tx_out = tx.clone();
                        let lines_out = output_lines.clone();
                        let running_out = running.clone();

                        thread::spawn(move || {
                            for line in reader.lines() {
                                if !*running_out.lock().unwrap() {
                                    break;
                                }
                                if let Ok(line) = line {
                                    lines_out.lock().unwrap().push(line.clone());
                                    // Keep buffer limited
                                    {
                                        let mut l = lines_out.lock().unwrap();
                                        if l.len() > 2000 {
                                            let drain = l.len() - 1500;
                                            l.drain(0..drain);
                                        }
                                    }
                                    tx_out.send(TerminalEvent::Output(line)).ok();
                                }
                            }
                        });
                    }

                    // Read stderr
                    if let Some(stderr) = child.stderr.take() {
                        let reader = BufReader::new(stderr);
                        let tx_err = tx.clone();
                        let lines_err = output_lines.clone();
                        let running_err = running.clone();

                        thread::spawn(move || {
                            for line in reader.lines() {
                                if !*running_err.lock().unwrap() {
                                    break;
                                }
                                if let Ok(line) = line {
                                    lines_err.lock().unwrap().push(line.clone());
                                    tx_err.send(TerminalEvent::Output(line)).ok();
                                }
                            }
                        });
                    }

                    // Wait for child
                    match child.wait() {
                        Ok(status) => {
                            *running.lock().unwrap() = false;
                            tx.send(TerminalEvent::Exited(
                                status.code().unwrap_or(-1),
                            ))
                            .ok();
                        }
                        Err(e) => {
                            *running.lock().unwrap() = false;
                            tx.send(TerminalEvent::Error(format!(
                                "Shell process error: {}",
                                e
                            )))
                            .ok();
                        }
                    }
                }
                Err(e) => {
                    *running.lock().unwrap() = false;
                    tx.send(TerminalEvent::Error(format!(
                        "Failed to start shell: {}",
                        e
                    )))
                    .ok();
                }
            }
        });
    }

    pub fn send_input(&self, input: &str) {
        if let Some(ref mut stdin) = *self.child_stdin.lock().unwrap() {
            let input_with_newline = if input.ends_with('\n') {
                input.to_string()
            } else {
                format!("{}\n", input)
            };
            stdin.write_all(input_with_newline.as_bytes()).ok();
            stdin.flush().ok();

            // Echo input to output
            self.output_lines
                .lock()
                .unwrap()
                .push(format!("$ {}", input.trim()));
        }
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    pub fn clear(&self) {
        self.output_lines.lock().unwrap().clear();
    }
}
