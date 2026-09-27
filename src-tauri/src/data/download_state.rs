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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_status_roundtrips_every_variant() {
        for status in [
            DownloadStatus::Idle,
            DownloadStatus::Downloading,
            DownloadStatus::Complete,
            DownloadStatus::Error,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(serde_json::from_str::<DownloadStatus>(&json).unwrap(), status);
        }
    }

    #[test]
    fn download_progress_keeps_optional_log() {
        let progress = DownloadProgress {
            status: DownloadStatus::Downloading,
            progress: 42,
            message: "downloading".into(),
            log: Some("chunk 3".into()),
        };
        let value = serde_json::to_value(&progress).unwrap();
        assert_eq!(value["status"], "Downloading");
        assert_eq!(value["progress"], 42);
        assert_eq!(value["log"], "chunk 3");

        let parsed: DownloadProgress = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.status, DownloadStatus::Downloading);
        assert_eq!(parsed.log.as_deref(), Some("chunk 3"));
    }

    #[test]
    fn acquisition_progress_roundtrips() {
        let progress = ModelAcquisitionProgress {
            model_id: "author/repo".into(),
            filename: "m.gguf".into(),
            status: "downloading".into(),
            progress: 12.5,
            message: "halfway".into(),
        };
        let parsed: ModelAcquisitionProgress =
            serde_json::from_str(&serde_json::to_string(&progress).unwrap()).unwrap();
        assert_eq!(parsed.model_id, "author/repo");
        assert_eq!(parsed.progress, 12.5);
        assert_eq!(parsed.filename, "m.gguf");
    }

    #[test]
    fn download_state_roundtrips() {
        let state = DownloadState {
            model_id: "author/repo".into(),
            filename: "m.gguf".into(),
            file_size: 1_000,
            downloaded: 250,
            progress: 25,
            status: "downloading".into(),
        };
        let parsed: DownloadState =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert_eq!(parsed.downloaded, 250);
        assert_eq!(parsed.progress, 25);
    }

    #[test]
    fn installation_information_roundtrips() {
        let info = InstallationInformation {
            platform: "linux".into(),
            command: "curl -fsSL https://ollama.com/install.sh | sh".into(),
            estimated_time: "~5 minutes".into(),
        };
        let parsed: InstallationInformation =
            serde_json::from_str(&serde_json::to_string(&info).unwrap()).unwrap();
        assert_eq!(parsed.platform, "linux");
        assert_eq!(parsed.command, info.command);
    }

    #[test]
    fn terminal_line_roundtrips() {
        let line = TerminalLine {
            line: "pulling manifest".into(),
            stream: "stdout".into(),
            is_progress: false,
        };
        let parsed: TerminalLine =
            serde_json::from_str(&serde_json::to_string(&line).unwrap()).unwrap();
        assert_eq!(parsed.line, "pulling manifest");
        assert!(!parsed.is_progress);
    }
}
