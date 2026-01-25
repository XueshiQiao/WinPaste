use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

static RUNNING: AtomicBool = AtomicBool::new(true);

pub fn start_clipboard_monitor(app: AppHandle) {
    let app_clone = app.clone();

    std::thread::spawn(move || {
        let mut seen_hashes: HashSet<String> = HashSet::new();
        let mut last_text = String::new();

        loop {
            if !RUNNING.load(Ordering::SeqCst) {
                break;
            }

            std::thread::sleep(Duration::from_millis(500));

            let current_text = get_clipboard_text();

            if current_text != last_text && !current_text.is_empty() {
                last_text = current_text.clone();

                let hash = calculate_hash(current_text.as_bytes());

                if !seen_hashes.contains(&hash) {
                    seen_hashes.insert(hash.clone());

                    let app_for_thread = app_clone.clone();
                    let clip = super::models::Clip {
                        id: 0,
                        uuid: Uuid::new_v4().to_string(),
                        clip_type: "text".to_string(),
                        content: current_text.as_bytes().to_vec(),
                        text_preview: current_text.chars().take(200).collect(),
                        content_hash: hash,
                        folder_id: None,
                        is_pinned: false,
                        is_deleted: false,
                        source_app: None,
                        metadata: None,
                        created_at: chrono::Utc::now(),
                        last_accessed: chrono::Utc::now(),
                    };

                    let _ = app_for_thread.emit("clipboard-change", &clip);
                }
            }
        }
    });
}

fn get_clipboard_text() -> String {
    use std::process::Command;

    let output = Command::new("powershell")
        .args(&["-Command", "Get-Clipboard"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout)
                    .to_string()
                    .trim()
                    .to_string()
            } else {
                String::new()
            }
        }
        Err(_) => String::new(),
    }
}

fn calculate_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn stop_clipboard_monitor() {
    RUNNING.store(false, Ordering::SeqCst);
}
