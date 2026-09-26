use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub status: DownloadStatus,
    pub progress: u8,
    pub message: String,
    pub log: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAcquisitionProgress {
    pub model_id: String,
    pub filename: String,
    pub status: String,
    pub progress: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Idle,
    Downloading,
    Complete,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadState {
    pub model_id: String,
    pub filename: String,
    pub file_size: u64,
    pub downloaded: u64,
    pub progress: u8,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInformation {
    pub platform: String,
    pub command: String,
    pub estimated_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalLine {
    pub line: String,
    pub stream: String,
    pub is_progress: bool,
}
