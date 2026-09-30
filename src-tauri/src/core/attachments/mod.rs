//! Attachment ingestion: turn user-picked files into bounded context blocks.
//!
//! Files are read by path at send time and only their metadata is persisted (on
//! the session settings JSON), so a session survives reloads while still
//! reflecting the current file on disk. Rescoping to RAG (Phase 5): this reader
//! is what retrieval should reuse.

use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::context::ContextBlock;

/// Per-file cap. Larger files are truncated, not rejected.
pub const MAX_FILE_BYTES: usize = 256 * 1024;
/// Total cap across all attached files in a single request.
pub const MAX_TOTAL_BYTES: usize = 1024 * 1024;

/// Persisted per-attachment metadata. Never stores file contents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttachmentMeta {
    pub path: String,
    pub name: String,
    #[serde(default)]
    pub bytes: u64,
    #[serde(default)]
    pub chars: usize,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default)]
    pub error: Option<String>,
}

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

/// Read at most `max` bytes, reporting whether the file was longer.
fn read_capped(path: &str, max: usize) -> std::io::Result<(Vec<u8>, bool)> {
    let mut handle = std::fs::File::open(path)?.take(max as u64 + 1);
    let mut buf = Vec::new();
    handle.read_to_end(&mut buf)?;
    let truncated = buf.len() > max;
    buf.truncate(max);
    Ok((buf, truncated))
}

/// Inspect a file for display. Captures read errors instead of failing.
pub fn probe_attachment(path: &str) -> AttachmentMeta {
    let mut meta = AttachmentMeta {
        path: path.to_string(),
        name: file_name(path),
        ..Default::default()
    };

    match std::fs::metadata(path) {
        Ok(data) => meta.bytes = data.len(),
        Err(error) => {
            meta.error = Some(error.to_string());
            return meta;
        }
    }

    match read_capped(path, MAX_FILE_BYTES) {
        Ok((bytes, truncated)) => {
            if bytes.contains(&0) {
                meta.error = Some("binary file skipped (contains NUL bytes)".to_string());
                return meta;
            }
            meta.chars = String::from_utf8_lossy(&bytes).chars().count();
            meta.truncated = truncated;
        }
        Err(error) => meta.error = Some(error.to_string()),
    }

    meta
}

/// Build a context block for a file, or `None` if it is unreadable or binary.
pub fn to_context_block(path: &str, max_bytes: usize) -> Option<ContextBlock> {
    let (bytes, truncated) = read_capped(path, max_bytes).ok()?;
    if bytes.contains(&0) {
        return None;
    }

    let mut content = String::from_utf8_lossy(&bytes).to_string();
    if truncated {
        content.push_str("\n[... truncated ...]");
    }

    Some(ContextBlock {
        label: file_name(path),
        content,
    })
}

/// Read the requested files. Unreadable, binary, or over-total-cap files are
/// skipped: a session outlives the files it points at, so a stale path must not
/// block the send.
pub fn read_attachment_blocks(attachments: &[AttachmentMeta]) -> Vec<ContextBlock> {
    let mut blocks = Vec::new();
    let mut total = 0usize;

    for attachment in attachments {
        let Some(block) = to_context_block(&attachment.path, MAX_FILE_BYTES) else {
            continue;
        };
        let size = block.content.len();
        if total + size > MAX_TOTAL_BYTES {
            continue;
        }
        total += size;
        blocks.push(block);
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(name: &str, bytes: &[u8]) -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(bytes).unwrap();
        (dir, path.to_string_lossy().to_string())
    }

    #[test]
    fn reads_text_and_reports_size() {
        let (_dir, path) = temp_file("a.txt", b"hello world");
        let meta = probe_attachment(&path);
        assert_eq!(meta.name, "a.txt");
        assert_eq!(meta.bytes, 11);
        assert_eq!(meta.chars, 11);
        assert!(!meta.truncated);
        assert!(meta.error.is_none());
    }

    #[test]
    fn skips_binary_files() {
        let (_dir, path) = temp_file("bin", b"abc\0def");
        assert!(probe_attachment(&path).error.is_some());
        assert!(to_context_block(&path, MAX_FILE_BYTES).is_none());
    }

    #[test]
    fn truncates_oversized_files() {
        let data = vec![b'x'; MAX_FILE_BYTES + 10];
        let (_dir, path) = temp_file("big.txt", &data);
        assert!(probe_attachment(&path).truncated);

        let block = to_context_block(&path, MAX_FILE_BYTES).unwrap();
        assert!(block.content.ends_with("[... truncated ...]"));
        assert!(block.content.chars().count() > MAX_FILE_BYTES);
    }

    #[test]
    fn missing_file_is_skipped_not_fatal() {
        let blocks = read_attachment_blocks(&[AttachmentMeta {
            path: "/does/not/exist.txt".into(),
            name: "exist.txt".into(),
            ..Default::default()
        }]);
        assert!(blocks.is_empty());
    }

    #[test]
    fn missing_file_still_reports_metadata() {
        let meta = probe_attachment("/does/not/exist.txt");
        assert_eq!(meta.name, "exist.txt");
        assert!(meta.error.is_some());
    }
}
