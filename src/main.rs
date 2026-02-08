mod bot_engine;
mod input_manager;
mod vision_engine;

use std::sync::mpsc::{self, Receiver, Sender};
use eframe::egui;
use bot_engine::{BotEngine, LogLevel, LogMessage};

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
    engine: BotEngine,
    logs: Vec<LogMessage>,
    log_receiver: Receiver<LogMessage>,
    current_tab: Tab,
}

impl MyBotApp {
    fn new(tx: Sender<LogMessage>, rx: Receiver<LogMessage>) -> Self {
        Self {
            engine: BotEngine::new(tx),
            logs: Vec::new(),
            log_receiver: rx,
            current_tab: Tab::Vision,
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
                    ui.colored_label(egui::Color32::LIGHT_BLUE, &self.engine.vision.target_window_name);

                    ui.label("Logical Resolution:");
                    ui.colored_label(egui::Color32::LIGHT_GREEN, &self.engine.vision.window_resolution);

                    ui.add_space(20.0);

                    if ui.button("Scan for Dofus").clicked() {
                        self.engine.scan_for_window();
                    }

                    ui.add_space(10.0);

                    if ui.button("Test Focus Sequence (5s delay)").clicked() {
                        self.engine.run_test_sequence();
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
