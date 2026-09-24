use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn open_path(app_handle: AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    app_handle
        .opener()
        .open_path(path.as_str(), None::<&str>)
        .map_err(|e| format!("Failed to open path: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn open_url(app_handle: AppHandle, url: String) -> Result<(), String> {
    // Use the opener plugin instead of deprecated shell.open()
    use tauri_plugin_opener::OpenerExt;
    
    // Open the URL using the opener plugin
    app_handle
        .opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn copy_to_clipboard(app_handle: AppHandle, text: String) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window
            .eval(&format!("navigator.clipboard.writeText('{}')", text.replace("'", "\\'")))
            .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
        Ok(())
    } else {
        // Fallback using system clipboard
        #[cfg(not(target_os = "linux"))]
        {
            use arboard::Clipboard;
            let mut clipboard = Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;
            clipboard.set_text(text).map_err(|e| format!("Failed to copy: {}", e))?;
            Ok(())
        }
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("xclip")
                .args(&["-selection", "clipboard"])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    let mut stdin = child.stdin.take().ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::Other, "Failed to get stdin")
                    })?;
                    use std::io::Write;
                    stdin.write_all(text.as_bytes())?;
                    child.wait()?;
                    Ok(())
                })
                .map_err(|e| format!("Failed to copy: {}", e))?;
            Ok(())
        }
    }
}

#[tauri::command]
pub async fn restart_app(_app_handle: AppHandle) -> Result<(), String> {
    std::process::exit(0);
}