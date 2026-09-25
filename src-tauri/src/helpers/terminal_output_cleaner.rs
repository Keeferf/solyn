use tauri::WebviewWindow;
use tauri::Emitter;
use crate::data::download_state::TerminalLine;

pub fn sanitize_output_line(line: &str) -> String {
    let trimmed = line.trim();
    
    let cleaned = trimmed
        .trim_start_matches("> ")
        .trim_start_matches(">>> ")
        .trim_start_matches(">> ")
        .trim_start_matches("$ ")
        .trim_start_matches("# ")
        .trim_start_matches("VERBOSE: ");

    if cleaned.is_empty() 
        || cleaned == ">" 
        || cleaned == ">>>" 
        || cleaned == ">>" 
        || cleaned.contains("Install complete. Run 'ollama' from the command line.")
        || cleaned.contains("Run 'ollama' from the command line.")
        || cleaned.contains("Install complete.")
        || (cleaned.contains("GET with") && cleaned.contains("payload"))
        || (cleaned.contains("received") && cleaned.contains("response of content type"))
    {
        return String::new();
    }
    
    cleaned.to_string()
}

pub fn remove_ansi_escape_codes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_escape = false;
    let mut in_bracket = false;
    
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            in_escape = true;
            in_bracket = false;
            continue;
        }
        
        if in_escape {
            if c == '[' {
                in_bracket = true;
                continue;
            }
            
            if in_bracket {
                if c.is_ascii_alphabetic() || c == '@' {
                    in_escape = false;
                    in_bracket = false;
                    continue;
                }
                continue;
            } else {
                if c.is_ascii_alphabetic() {
                    in_escape = false;
                    continue;
                }
            }
        } else {
            result.push(c);
        }
    }
    
    result
}

pub fn parse_and_emit_terminal_output(window: &WebviewWindow, text: &str, stream_type: &str) {
    let mut current_line = String::new();
    
    for ch in text.chars() {
        match ch {
            '\r' => {
                if !current_line.is_empty() {
                    let stripped = remove_ansi_escape_codes(&current_line);
                    let cleaned = sanitize_output_line(&stripped);
                    if !cleaned.is_empty() {
                        broadcast_terminal_line(window, &cleaned, stream_type, true);
                    }
                }
                current_line.clear();
            }
            '\n' => {
                if !current_line.is_empty() {
                    let stripped = remove_ansi_escape_codes(&current_line);
                    let cleaned = sanitize_output_line(&stripped);
                    if !cleaned.is_empty() {
                        broadcast_terminal_line(window, &cleaned, stream_type, false);
                    }
                }
                current_line.clear();
            }
            _ => {
                current_line.push(ch);
            }
        }
    }
    
    if !current_line.is_empty() {
        let stripped = remove_ansi_escape_codes(&current_line);
        let cleaned = sanitize_output_line(&stripped);
        if !cleaned.is_empty() {
            broadcast_terminal_line(window, &cleaned, stream_type, true);
        }
    }
}

pub fn broadcast_terminal_line(window: &WebviewWindow, line: &str, stream_type: &str, is_progress: bool) {
    let cleaned_line = sanitize_output_line(line);
    if cleaned_line.is_empty() {
        return;
    }
    
    let _ = window.emit("terminal-output", TerminalLine {
        line: cleaned_line,
        stream: stream_type.to_string(),
        is_progress,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_shell_prompts() {
        assert_eq!(sanitize_output_line("> hello"), "hello");
        assert_eq!(sanitize_output_line(">>> hello"), "hello");
        assert_eq!(sanitize_output_line(">> hello"), "hello");
        assert_eq!(sanitize_output_line("$ ls"), "ls");
        assert_eq!(sanitize_output_line("# root"), "root");
        assert_eq!(sanitize_output_line("VERBOSE: msg"), "msg");
    }

    #[test]
    fn drops_noise_and_empty_lines() {
        assert_eq!(sanitize_output_line("   "), "");
        assert_eq!(sanitize_output_line(">"), "");
        assert_eq!(sanitize_output_line("Install complete."), "");
        assert_eq!(
            sanitize_output_line("Install complete. Run 'ollama' from the command line."),
            ""
        );
        assert_eq!(sanitize_output_line("GET with some payload here"), "");
        assert_eq!(
            sanitize_output_line("received a response of content type json"),
            ""
        );
    }

    #[test]
    fn keeps_meaningful_lines() {
        assert_eq!(sanitize_output_line("Downloading model"), "Downloading model");
    }

    #[test]
    fn removes_ansi_escape_codes() {
        assert_eq!(remove_ansi_escape_codes("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(remove_ansi_escape_codes("\x1b[2Khello"), "hello");
        assert_eq!(remove_ansi_escape_codes("plain"), "plain");
        assert_eq!(remove_ansi_escape_codes("a\x1b[0mb"), "ab");
    }
}