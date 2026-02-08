use std::process::Command;
use std::thread;
use std::time::Duration;
use screencapturekit::shareable_content::SCShareableContent;

fn main() {
    println!("Starting Focus Test Program...");

    println!("Scanning for Dofus window...");
    let content = match SCShareableContent::get() {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to get shareable content: {:?}", e);
            return;
        }
    };

    let dofus_window = content.windows().into_iter().find(|w| {
        w.title().contains("Dofus")
    });

    if let Some(window) = dofus_window {
        let pid = window.owning_application().process_id;
        println!("Found Dofus window: {} (PID: {})", window.title(), pid);

        let focus_pid = |p: i32| {
            let script = format!(
                "tell application \"System Events\" to set frontmost of first process whose unix id is {} to true",
                p
            );
            let output = Command::new("osascript")
                .arg("-e")
                .arg(script)
                .output();

            match output {
                Ok(out) => {
                    if !out.status.success() {
                        eprintln!("AppleScript error: {}", String::from_utf8_lossy(&out.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to run osascript: {}", e),
            }
        };

        println!("Focusing Dofus in 5 seconds...");
        thread::sleep(Duration::from_secs(5));
        focus_pid(pid);
        println!("Dofus focused.");

        println!("Focusing this test program in 5 seconds...");
        thread::sleep(Duration::from_secs(5));
        focus_pid(std::process::id() as i32);
        println!("Test program focused.");

        println!("Test complete.");
    } else {
        println!("Dofus window not found. Please make sure Dofus is running.");
    }
}
