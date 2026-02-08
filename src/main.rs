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
            .with_inner_size([600.0, 500.0]), // Increased size for vision preview
        ..Default::default()
    };

    let (tx, rx) = mpsc::channel();

    eframe::run_native(
        "Botfus",
        native_options,
        Box::new(|_cc| Box::new(MyBotApp::new(tx, rx))),
    )
}

struct MyBotApp {
    engine: BotEngine,
    logs: Vec<LogMessage>,
    log_receiver: Receiver<LogMessage>,
    current_tab: Tab,
    texture: Option<egui::TextureHandle>,
}

impl MyBotApp {
    fn new(tx: Sender<LogMessage>, rx: Receiver<LogMessage>) -> Self {
        Self {
            engine: BotEngine::new(tx), 
            logs: Vec::new(),
            log_receiver: rx,
            current_tab: Tab::Vision,
            texture: None,
        }
    }
}

impl eframe::App for MyBotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Drain logs from the channel
        while let Ok(msg) = self.log_receiver.try_recv() {
            self.logs.push(msg);
        }

        // 2. Update Live Texture if in Vision Tab
        if self.current_tab == Tab::Vision {
            if let Some(rgba_img) = self.engine.vision.capture_frame() {
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [rgba_img.width() as usize, rgba_img.height() as usize],
                    rgba_img.as_flat_samples().as_slice(),
                );

                if let Some(ref mut tex) = self.texture {
                    tex.set(color_image, egui::TextureOptions::LINEAR);
                } else {
                    self.texture = Some(ctx.load_texture(
                        "live_view",
                        color_image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
                // Request a repaint to keep the stream moving
                ctx.request_repaint();
            }
        }

        // 3. Top Navigation Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Vision, "Vision");
                ui.selectable_value(&mut self.current_tab, Tab::Logs, "Logs");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let stream_active = self.engine.vision.stream.is_some();
                    if stream_active {
                        ui.colored_label(egui::Color32::GREEN, "STREAM ACTIVE");
                    } else {
                        ui.colored_label(egui::Color32::RED, "STREAM INACTIVE");
                    }
                });
            });
        });

        // 4. Main Content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Vision => {
                    ui.horizontal(|ui| {
                        ui.heading("Dofus Unity Vision");
                        if ui.button("Scan for Dofus").clicked() {
                            self.engine.scan_for_window();
                        }
                    });
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Window:");
                        ui.colored_label(egui::Color32::LIGHT_BLUE, &self.engine.vision.target_window_name);
                        ui.label("| Resolution:");
                        ui.colored_label(egui::Color32::LIGHT_GREEN, &self.engine.vision.window_resolution);
                    });

                    ui.add_space(10.0);

                    // Action Buttons
                    ui.horizontal(|ui| {
                        if ui.button("📸 Trigger Mission Proof").clicked() {
                            self.engine.trigger_mission_proof();
                        }
                        if ui.button("📁 Open Mission Folder").clicked() {
                            let _ = std::process::Command::new("open")
                                .arg("./mission_logs")
                                .spawn();
                        }
                    });

                    ui.add_space(10.0);

                    // Live Preview
                    if let Some(ref tex) = self.texture {
                        let size = tex.size_vec2();
                        let max_size = ui.available_size();
                        let scale = (max_size.x / size.x).min(max_size.y / size.y).min(1.0);
                        ui.image(tex, size * scale);
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Waiting for stream... (Scan for Dofus to start)");
                        });
                    }
                }
                Tab::Logs => {
                    ui.heading("System Logs");
                    ui.separator();

                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        for log in self.logs.iter() {
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
