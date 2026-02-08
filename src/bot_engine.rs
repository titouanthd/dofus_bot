use crate::input_manager::InputManager;
use crate::vision_engine::VisionEngine;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use chrono::Local;

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

pub struct LogMessage {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

pub struct BotEngine {
    pub vision: VisionEngine,
    pub input: InputManager,
    log_tx: Sender<LogMessage>,
}

impl BotEngine {
    pub fn new(log_tx: Sender<LogMessage>) -> Self {
        Self {
            vision: VisionEngine::new(),
            input: InputManager::new(),
            log_tx,
        }
    }

    pub fn log(&self, message: &str, level: LogLevel) {
        let timestamp = Local::now().format("%H:%M:%S").to_string();
        let _ = self.log_tx.send(LogMessage {
            timestamp,
            level,
            message: message.to_string(),
        });
    }

    pub fn scan_for_window(&mut self) {
        self.log("Scanning for Dofus window...", LogLevel::Info);
        match self.vision.find_dofus_window() {
            Ok(msg) => self.log(&msg, LogLevel::Success),
            Err(err) => self.log(&err, LogLevel::Warning),
        }
    }

    pub fn focus_dofus(&self) {
        if let Some(pid) = self.vision.target_window_pid {
            self.log(&format!("Focusing Dofus window (PID: {}) natively...", pid), LogLevel::Info);
            self.input.focus_window(pid);
        } else {
            self.log("Cannot focus Dofus: Window not found.", LogLevel::Warning);
        }
    }

    pub fn focus_bot(&self) {
        self.log("Focusing Bot window natively...", LogLevel::Info);
        self.input.focus_window(std::process::id() as i32);
    }

    pub fn run_test_sequence(&self) {
        let tx = self.log_tx.clone();
        let vision_pid = self.vision.target_window_pid;
        let bot_pid = std::process::id() as i32;
        let input = InputManager::new();

        thread::spawn(move || {
            let log = |msg: &str, level: LogLevel| {
                let timestamp = Local::now().format("%H:%M:%S").to_string();
                let _ = tx.send(LogMessage {
                    timestamp,
                    level,
                    message: msg.to_string(),
                });
            };

            if let Some(p) = vision_pid {
                log("Test: Focusing Dofus in 5s...", LogLevel::Info);
                thread::sleep(Duration::from_secs(5));
                input.focus_window(p);
                log("Test: Dofus focused. Focusing Bot in 5s...", LogLevel::Info);
                thread::sleep(Duration::from_secs(5));
                input.focus_window(bot_pid);
                log("Test: Bot focused. Sequence complete.", LogLevel::Success);
            } else {
                log("Test: Dofus PID not found. Scan first.", LogLevel::Warning);
            }
        });
    }
}
