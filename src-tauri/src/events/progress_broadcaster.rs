use crate::data::download_state::{DownloadProgress, DownloadStatus, ModelAcquisitionProgress};
use tauri::Emitter;
use tauri::WebviewWindow;

pub fn broadcast_download_progress(
    window: &WebviewWindow,
    status: DownloadStatus,
    progress: u8,
    message: String,
    log: Option<&str>,
) {
    let progress_data = DownloadProgress {
        status,
        progress,
        message: if message.is_empty() {
            "Processing...".to_string()
        } else {
            message
        },
        log: log.map(|s| s.to_string()),
    };

    let _ = window.emit("download-progress", progress_data);
}

pub fn broadcast_model_acquisition_progress(
    window: &WebviewWindow,
    model_id: &str,
    status: &str,
    progress: u8,
    message: &str,
) {
    let progress_data = ModelAcquisitionProgress {
        model_id: model_id.to_string(),
        filename: "".to_string(),
        status: status.to_string(),
        progress: progress as f64,
        message: message.to_string(),
    };

    let _ = window.emit("model-download-progress", progress_data);
}
