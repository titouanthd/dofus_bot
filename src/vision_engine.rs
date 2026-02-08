use screencapturekit::shareable_content::SCShareableContent;
use image::RgbaImage;

pub struct VisionEngine {
    pub target_window_name: String,
    pub target_window_pid: Option<i32>,
    pub window_resolution: String,
}

impl VisionEngine {
    pub fn new() -> Self {
        Self {
            target_window_name: "Not Scanned".to_owned(),
            target_window_pid: None,
            window_resolution: "0x0".to_owned(),
        }
    }

    /// Scans for the Dofus window and updates internal state.
    pub fn find_dofus_window(&mut self) -> Result<String, String> {
        if let Ok(content) = SCShareableContent::get() {
            let dofus_window = content.windows().into_iter().find(|w| {
                w.title().contains("Dofus")
            });

            if let Some(window) = dofus_window {
                self.target_window_name = window.title();
                self.target_window_pid = Some(window.owning_application().process_id);
                self.window_resolution = format!("ID: {}", window.window_id());
                Ok(format!("Found window: {} (PID: {})", self.target_window_name, self.target_window_pid.unwrap()))
            } else {
                self.target_window_name = "Dofus not found".to_string();
                self.target_window_pid = None;
                Err("Dofus window not found.".to_string())
            }
        } else {
            Err("Failed to get shareable content.".to_string())
        }
    }

    /// Captures a frame from the target window.
    /// Phase C: Placeholder for future image processing.
    pub fn capture_frame(&self) -> Option<RgbaImage> {
        // Implementation will go here using SCStream
        None
    }
}
