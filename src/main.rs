use std::sync::mpsc::{self, Receiver, Sender};
use chrono::Local;
use eframe::egui;
use screencapturekit::shareable_content::SCShareableContent;

#[derive(Debug, Clone)]
enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

struct LogMessage {
    timestamp: String,
    level: LogLevel,
    message: String,
}

#[derive(PartialEq)]
enum Tab {
    Vision,
    Logs,
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    let (tx, rx) = mpsc::channel();

    eframe::run_native(
        "Dofus Vision Module",
        native_options,
        Box::new(|_cc| Box::new(MyBotApp::new(tx, rx))),
    )
}

struct MyBotApp {
    target_window_name: String,
    window_resolution: String,
    logs: Vec<LogMessage>,
    log_receiver: Receiver<LogMessage>,
    log_tx: Sender<LogMessage>,
    current_tab: Tab,
}

impl MyBotApp {
    fn new(tx: Sender<LogMessage>, rx: Receiver<LogMessage>) -> Self {
        Self {
            target_window_name: "Not Scanned".to_owned(),
            window_resolution: "0x0".to_owned(),
            logs: Vec::new(),
            log_receiver: rx,
            log_tx: tx,
            current_tab: Tab::Vision,
        }
    }

    fn log(&self, message: &str, level: LogLevel) {
        let timestamp = Local::now().format("%H:%M:%S").to_string();
        let _ = self.log_tx.send(LogMessage {
            timestamp,
            level,
            message: message.to_string(),
        });
    }

    fn find_dofus_window(&mut self) {
        self.log("Scanning for Dofus window...", LogLevel::Info);
        if let Ok(content) = SCShareableContent::get() {
            // We find the window that contains Dofus
            let dofus_window = content.windows().into_iter().find(|w| {
                w.title().contains("Dofus")
            });

            if let Some(window) = dofus_window {
                self.target_window_name = window.title();
                self.window_resolution = format!("ID: {}", window.window_id());
                self.log(&format!("Found window: {}", self.target_window_name), LogLevel::Success);
            } else {
                self.target_window_name = "Dofus not found".to_string();
                self.log("Dofus window not found.", LogLevel::Warning);
            }
        } else {
            self.log("Failed to get shareable content.", LogLevel::Error);
        }
    }
}

impl eframe::App for MyBotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain logs
        while let Ok(msg) = self.log_receiver.try_recv() {
            self.logs.push(msg);
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Vision, "Vision");
                ui.selectable_value(&mut self.current_tab, Tab::Logs, "Logs");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Vision => {
                    ui.heading("Dofus Unity Vision");
                    ui.separator();

                    ui.label("Detected Window:");
                    ui.colored_label(egui::Color32::LIGHT_BLUE, &self.target_window_name);

                    ui.label("Logical Resolution:");
                    ui.colored_label(egui::Color32::LIGHT_GREEN, &self.window_resolution);

                    ui.add_space(20.0);

                    if ui.button("Scan for Dofus").clicked() {
                        self.find_dofus_window();
                    }
                }
                Tab::Logs => {
                    ui.heading("System Logs");
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for log in self.logs.iter().rev() {
                            ui.horizontal(|ui| {
                                ui.label(format!("[{}]", log.timestamp));
                                let color = match log.level {
                                    LogLevel::Info => egui::Color32::GRAY,
                                    LogLevel::Success => egui::Color32::GREEN,
                                    LogLevel::Warning => egui::Color32::YELLOW,
                                    LogLevel::Error => egui::Color32::RED,
                                };
                                ui.colored_label(color, &log.message);
                            });
                        }
                    });
                }
            }
        });
    }
}