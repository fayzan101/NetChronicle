use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForegroundWindow {
    pub app_name: String,
    pub friendly_name: String,
    pub process_path: String,
    pub process_id: u32,
    pub window_title: String,
}

pub fn current_foreground() -> anyhow::Result<ForegroundWindow> {
    let window = x_win::get_active_window().map_err(|e| anyhow::anyhow!("{e}"))?;
    let app_name = if window.info.exec_name.is_empty() {
        window.info.name.clone()
    } else {
        window.info.exec_name.clone()
    };
    let friendly_name = friendly_name_from_process(&app_name, &window.info.path);

    Ok(ForegroundWindow {
        app_name,
        friendly_name,
        process_path: window.info.path,
        process_id: window.info.process_id,
        window_title: window.title,
    })
}

pub fn friendly_name_from_process(exec_name: &str, process_path: &str) -> String {
    let file_name = Path::new(process_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(exec_name);

    let lower = file_name.to_lowercase();
    let mapped = match lower.as_str() {
        "code" => "Visual Studio Code",
        "devenv" => "Visual Studio",
        "msedge" => "Microsoft Edge",
        "chrome" => "Google Chrome",
        "firefox" => "Firefox",
        "brave" => "Brave",
        "opera" => "Opera",
        "vivaldi" => "Vivaldi",
        "explorer" => "File Explorer",
        "windowsterminal" => "Windows Terminal",
        "powershell" => "PowerShell",
        "cmd" => "Command Prompt",
        "discord" => "Discord",
        "slack" => "Slack",
        "teams" => "Microsoft Teams",
        "spotify" => "Spotify",
        _ => file_name,
    };

    mapped.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_processes() {
        assert_eq!(
            friendly_name_from_process("Code.exe", r"C:\Program Files\Code\Code.exe"),
            "Visual Studio Code"
        );
    }
}
