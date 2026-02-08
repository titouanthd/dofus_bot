use eframe::egui;
use screencapturekit::shareable_content::SCShareableContent;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Dofus Vision Module",
        native_options,
        Box::new(|_cc| Box::new(MyBotApp::default())),
    )
}

struct MyBotApp {
    target_window_name: String,
    window_resolution: String,
}

impl Default for MyBotApp {
    fn default() -> Self {
        Self {
            target_window_name: "Not Scanned".to_owned(),
            window_resolution: "0x0".to_owned(),
        }
    }
}

impl MyBotApp {
    fn find_dofus_window(&mut self) {
        if let Ok(content) = SCShareableContent::get() {
            self.target_window_name = "Dofus not found".to_string();

            // We find the window that contains Dofus
            let dofus_window = content.windows().into_iter().find(|w| {
                w.title().contains("Dofus")
            });

            if let Some(window) = dofus_window {
                self.target_window_name = window.title();
                
                // Since the geometry fields are currently giving us trouble 
                // because they are private in this crate version, 
                // we will display the Window ID for now to prove it's found.
                // The actual resolution is handled by the "Stream" we build next.
                self.window_resolution = format!("ID: {}", window.window_id());
            }
        }
    }
}

impl eframe::App for MyBotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
        });
    }
}