use screencapturekit::{
    cm_sample_buffer::CMSampleBuffer,
    content_filter::{InitParams, SCContentFilter},
    shareable_content::SCShareableContent,
    stream::{SCStream, SCStreamConfiguration, SCStreamOutput, SCStreamOutputType},
};
use std::sync::{Arc, Mutex};
use image::RgbaImage;

pub struct VisionEngine {
    pub target_window_name: String,
    pub target_window_pid: Option<i32>,
    pub target_window_id: Option<u32>,
    pub window_resolution: String,
    pub latest_frame: Arc<Mutex<Vec<u8>>>,
    pub frame_size: Arc<Mutex<(u32, u32)>>,
    pub stream: Option<SCStream>,
}

struct StreamHandler {
    latest_frame: Arc<Mutex<Vec<u8>>>,
    frame_size: Arc<Mutex<(u32, u32)>>,
}

impl SCStreamOutput for StreamHandler {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType) {
        if let SCStreamOutputType::Screen = of_type {
            if let Ok(pixel_buffer) = sample_buffer.image_buffer() {
                let width = pixel_buffer.width() as u32;
                let height = pixel_buffer.height() as u32;

                let data = pixel_buffer.as_slice();
                if data.is_empty() { return; }

                // High-performance BGRA to RGBA conversion
                let mut rgba = Vec::with_capacity(data.len());
                for bgra in data.chunks_exact(4) {
                    rgba.push(bgra[2]); // R
                    rgba.push(bgra[1]); // G
                    rgba.push(bgra[0]); // B
                    rgba.push(bgra[3]); // A
                }

                if let Ok(mut latest) = self.latest_frame.lock() {
                    *latest = rgba;
                }
                if let Ok(mut size) = self.frame_size.lock() {
                    *size = (width, height);
                }
            }
        }
    }
}

impl VisionEngine {
    pub fn new() -> Self {
        Self {
            target_window_name: "Not Scanned".to_owned(),
            target_window_pid: None,
            target_window_id: None,
            window_resolution: "0x0".to_owned(),
            latest_frame: Arc::new(Mutex::new(Vec::new())),
            frame_size: Arc::new(Mutex::new((0, 0))),
            stream: None,
        }
    }

    pub fn find_dofus_window(&mut self) -> Result<String, String> {
        if let Ok(content) = SCShareableContent::get() {
            let dofus_window = content.windows().into_iter().find(|w| {
                w.title().contains("Dofus")
            });

            if let Some(window) = dofus_window {
                self.target_window_name = window.title().to_string();
                self.target_window_pid = Some(window.owning_application().process_id());
                self.target_window_id = Some(window.window_id());
                self.window_resolution = format!("{}x{}", window.width(), window.height());

                let window_id = window.window_id();
                let w = window.width() as u32;
                let h = window.height() as u32;

                self.start_streaming(window_id, w, h);

                Ok(format!("Found window: {} (PID: {})", self.target_window_name, self.target_window_pid.unwrap()))
            } else {
                self.target_window_name = "Dofus not found".to_string();
                self.target_window_pid = None;
                self.target_window_id = None;
                Err("Dofus window not found.".to_string())
            }
        } else {
            Err("Failed to get shareable content.".to_string())
        }
    }

    pub fn start_streaming(&mut self, window_id: u32, width: u32, height: u32) {
        if let Some(mut s) = self.stream.take() {
            let _ = s.stop_capture();
        }

        if let Ok(content) = SCShareableContent::get() {
            if let Some(window) = content.windows().into_iter().find(|w| w.window_id() == window_id) {
                let filter = SCContentFilter::new(InitParams::Window(window));
                let mut config = SCStreamConfiguration::default();
                config.width = width;
                config.height = height;
                config.shows_cursor = false;

                let handler = StreamHandler {
                    latest_frame: Arc::clone(&self.latest_frame),
                    frame_size: Arc::clone(&self.frame_size),
                };

                let mut stream = SCStream::new(filter, config, handler);
                let _ = stream.start_capture();
                self.stream = Some(stream);
            }
        }
    }

    pub fn capture_frame(&self) -> Option<RgbaImage> {
        let data = self.latest_frame.lock().ok()?.clone();
        let size = self.frame_size.lock().ok()?;

        if data.is_empty() || size.0 == 0 || size.1 == 0 {
            return None;
        }

        RgbaImage::from_raw(size.0, size.1, data)
    }
}

impl Drop for VisionEngine {
    fn drop(&mut self) {
        if let Some(mut s) = self.stream.take() {
            let _ = s.stop_capture();
        }
    }
}
