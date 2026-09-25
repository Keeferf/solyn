use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufWriter};
use futures_util::StreamExt;

use super::paths::ModelPaths;

const WRITE_BUFFER_SIZE: usize = 8 * 1024 * 1024; // 8MB

/// Download a single chunk with resume support
pub async fn download_chunk(
    url: &str,
    paths: &ModelPaths,
    filename: &str,
    chunk_index: usize,
    start_byte: u64,
    end_byte: u64,
    cancel_token: &Arc<std::sync::atomic::AtomicBool>,
    client: &reqwest::Client,
) -> Result<(), String> {
    let chunk_path = paths.chunk_path(filename, chunk_index);

    // Check if chunk already exists and get its size
    let existing_size = if chunk_path.exists() {
        if let Ok(metadata) = fs::metadata(&chunk_path).await {
            metadata.len()
        } else {
            0
        }
    } else {
        0
    };

    let expected_size = end_byte - start_byte + 1;

    if existing_size == expected_size {
        return Ok(());
    }

    let resume_from = existing_size;
    let resume_start = start_byte + resume_from;

    let file = if resume_from > 0 && chunk_path.exists() {
        tokio::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&chunk_path)
            .await
            .map_err(|e| format!("Failed to open chunk {} for resume: {}", chunk_index, e))?
    } else {
        fs::File::create(&chunk_path)
            .await
            .map_err(|e| format!("Failed to create chunk {}: {}", chunk_index, e))?
    };

    let mut buffered_writer = BufWriter::with_capacity(WRITE_BUFFER_SIZE, file);
    let mut downloaded = resume_from;

    let mut request_builder = client
        .get(url)
        .header("User-Agent", "SolynApp/1.0")
        .timeout(Duration::from_secs(3600));

    if resume_from > 0 {
        request_builder = request_builder.header("Range", format!("bytes={}-{}", resume_start, end_byte));
    } else {
        request_builder = request_builder.header("Range", format!("bytes={}-{}", start_byte, end_byte));
    }

    let response = request_builder
        .send()
        .await
        .map_err(|e| format!("Failed to download chunk {}: {}", chunk_index, e))?;

    if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(format!("Chunk {} download failed with status: {}", chunk_index, response.status()));
    }

    let mut stream = response.bytes_stream();
    let mut buffer_bytes_written: u64 = 0;
    let buffer_capacity = WRITE_BUFFER_SIZE as u64;

    while let Some(chunk_result) = stream.next().await {
        if cancel_token.load(Ordering::SeqCst) {
            let _ = buffered_writer.flush().await;
            return Err("Download cancelled".to_string());
        }

        let chunk = chunk_result
            .map_err(|e| format!("Chunk {} download error: {}", chunk_index, e))?;

        buffered_writer.write_all(&chunk)
            .await
            .map_err(|e| format!("Chunk {} write error: {}", chunk_index, e))?;

        downloaded += chunk.len() as u64;
        buffer_bytes_written += chunk.len() as u64;

        if buffer_bytes_written >= buffer_capacity {
            buffered_writer.flush()
                .await
                .map_err(|e| format!("Failed to flush chunk {}: {}", chunk_index, e))?;
            buffer_bytes_written = 0;
        }
    }

    buffered_writer.flush()
        .await
        .map_err(|e| format!("Failed to flush chunk {}: {}", chunk_index, e))?;

    if let Err(e) = buffered_writer.into_inner().sync_all().await {
        log::warn!("Failed to sync chunk {}: {}", chunk_index, e);
    }

    if downloaded != expected_size {
        return Err(format!(
            "Chunk {} incomplete: {}/{} bytes",
            chunk_index, downloaded, expected_size
        ));
    }

    Ok(())
}

/// Get total downloaded size from all chunks
pub async fn get_total_downloaded_size(paths: &ModelPaths, filename: &str, num_chunks: usize) -> u64 {
    let mut total = 0;
    for i in 0..num_chunks {
        let chunk_path = paths.chunk_path(filename, i);
        if let Ok(metadata) = fs::metadata(&chunk_path).await {
            total += metadata.len();
        }
    }
    total
}

/// Check if all chunks are complete
pub async fn are_all_chunks_complete(paths: &ModelPaths, filename: &str, num_chunks: usize, total_size: u64) -> bool {
    for i in 0..num_chunks {
        let chunk_path = paths.chunk_path(filename, i);
        if !chunk_path.exists() {
            return false;
        }

        if let Ok(metadata) = fs::metadata(&chunk_path).await {
            let expected_size = if i == num_chunks - 1 {
                let remainder = total_size % (total_size / num_chunks as u64);
                if remainder > 0 {
                    total_size / num_chunks as u64 + remainder
                } else {
                    total_size / num_chunks as u64
                }
            } else {
                total_size / num_chunks as u64
            };

            if metadata.len() != expected_size {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

/// Combine all chunks into the final file
pub async fn combine_chunks(
    paths: &ModelPaths,
    filename: &str,
    num_chunks: usize,
) -> Result<(), String> {
    let final_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&paths.file_path)
        .await
        .map_err(|e| format!("Failed to create final file: {}", e))?;

    let mut buffered_writer = BufWriter::with_capacity(WRITE_BUFFER_SIZE, final_file);

    for i in 0..num_chunks {
        let chunk_path = paths.chunk_path(filename, i);
        let mut chunk_file = fs::File::open(&chunk_path)
            .await
            .map_err(|e| format!("Failed to open chunk {}: {}", i, e))?;

        let mut buffer = vec![0u8; WRITE_BUFFER_SIZE];
        loop {
            let bytes_read = tokio::io::AsyncReadExt::read(&mut chunk_file, &mut buffer)
                .await
                .map_err(|e| format!("Failed to read chunk {}: {}", i, e))?;

            if bytes_read == 0 {
                break;
            }

            buffered_writer.write_all(&buffer[..bytes_read])
                .await
                .map_err(|e| format!("Failed to write chunk {}: {}", i, e))?;
        }
    }

    buffered_writer.flush()
        .await
        .map_err(|e| format!("Failed to flush final file: {}", e))?;

    if let Err(e) = buffered_writer.into_inner().sync_all().await {
        log::warn!("Failed to sync final file: {}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model_paths(dir: &std::path::Path, filename: &str) -> ModelPaths {
        ModelPaths {
            model_dir: dir.to_path_buf(),
            file_path: dir.join(filename),
            part_path: dir.join(format!("{}.part", filename)),
        }
    }

    #[tokio::test]
    async fn totals_downloaded_chunk_sizes() {
        let dir = tempfile::tempdir().unwrap();
        let paths = model_paths(dir.path(), "m.gguf");
        tokio::fs::write(paths.chunk_path("m.gguf", 0), vec![0u8; 10]).await.unwrap();
        tokio::fs::write(paths.chunk_path("m.gguf", 1), vec![0u8; 5]).await.unwrap();

        assert_eq!(get_total_downloaded_size(&paths, "m.gguf", 3).await, 15);
    }

    #[tokio::test]
    async fn all_chunks_complete_when_sizes_match() {
        let dir = tempfile::tempdir().unwrap();
        let paths = model_paths(dir.path(), "m.gguf");
        tokio::fs::write(paths.chunk_path("m.gguf", 0), vec![0u8; 10]).await.unwrap();
        tokio::fs::write(paths.chunk_path("m.gguf", 1), vec![0u8; 10]).await.unwrap();

        assert!(are_all_chunks_complete(&paths, "m.gguf", 2, 20).await);
    }

    #[tokio::test]
    async fn all_chunks_incomplete_when_a_chunk_is_short() {
        let dir = tempfile::tempdir().unwrap();
        let paths = model_paths(dir.path(), "m.gguf");
        tokio::fs::write(paths.chunk_path("m.gguf", 0), vec![0u8; 10]).await.unwrap();
        tokio::fs::write(paths.chunk_path("m.gguf", 1), vec![0u8; 9]).await.unwrap();

        assert!(!are_all_chunks_complete(&paths, "m.gguf", 2, 20).await);
    }

    #[tokio::test]
    async fn last_chunk_expected_size_includes_remainder() {
        let dir = tempfile::tempdir().unwrap();
        let paths = model_paths(dir.path(), "m.gguf");
        // total 25 / 2 chunks -> chunk_size 12, remainder 1 -> last chunk 13.
        tokio::fs::write(paths.chunk_path("m.gguf", 0), vec![0u8; 12]).await.unwrap();
        tokio::fs::write(paths.chunk_path("m.gguf", 1), vec![0u8; 13]).await.unwrap();

        assert!(are_all_chunks_complete(&paths, "m.gguf", 2, 25).await);

        tokio::fs::write(paths.chunk_path("m.gguf", 1), vec![0u8; 12]).await.unwrap();
        assert!(!are_all_chunks_complete(&paths, "m.gguf", 2, 25).await);
    }

    #[tokio::test]
    async fn missing_chunk_is_incomplete() {
        let dir = tempfile::tempdir().unwrap();
        let paths = model_paths(dir.path(), "m.gguf");
        tokio::fs::write(paths.chunk_path("m.gguf", 0), vec![0u8; 10]).await.unwrap();

        assert!(!are_all_chunks_complete(&paths, "m.gguf", 2, 20).await);
    }

    #[tokio::test]
    async fn combine_chunks_reassembles_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let paths = model_paths(dir.path(), "m.gguf");
        tokio::fs::write(paths.chunk_path("m.gguf", 0), [1u8, 2, 3]).await.unwrap();
        tokio::fs::write(paths.chunk_path("m.gguf", 1), [4u8, 5]).await.unwrap();

        combine_chunks(&paths, "m.gguf", 2).await.unwrap();

        assert_eq!(
            tokio::fs::read(&paths.file_path).await.unwrap(),
            vec![1, 2, 3, 4, 5]
        );
    }
}