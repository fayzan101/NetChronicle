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
    let file_name = process_stem(process_path).unwrap_or_else(|| {
        process_stem(exec_name)
            .unwrap_or_else(|| exec_name.trim().to_string())
    });

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
        "windowsterminal" | "wt" => "Windows Terminal",
        "powershell" => "PowerShell",
        "pwsh" => "PowerShell",
        "cmd" => "Command Prompt",
        "discord" => "Discord",
        "slack" => "Slack",
        "teams" => "Microsoft Teams",
        "spotify" => "Spotify",
        "notion" => "Notion",
        "obsidian" => "Obsidian",
        "figma" => "Figma",
        "postman" => "Postman",
        "docker desktop" | "docker" => "Docker Desktop",
        "cursor" => "Cursor",
        "winword" => "Microsoft Word",
        "excel" => "Microsoft Excel",
        "powerpnt" => "Microsoft PowerPoint",
        "outlook" => "Microsoft Outlook",
        "onenote" => "OneNote",
        "zoom" => "Zoom",
        "telegram" => "Telegram",
        "whatsapp" => "WhatsApp",
        "steam" => "Steam",
        "vlc" => "VLC",
        "itunes" => "Apple Music",
        "music" => "Apple Music",
        "safari" => "Safari",
        "finder" => "Finder",
        "terminal" => "Terminal",
        "iTerm2" | "iterm2" => "iTerm",
        "idea64" | "idea" => "IntelliJ IDEA",
        "pycharm64" | "pycharm" => "PyCharm",
        "webstorm64" | "webstorm" => "WebStorm",
        "goland64" | "goland" => "GoLand",
        "clion64" | "clion" => "CLion",
        "rider64" | "rider" => "Rider",
        "datagrip64" | "datagrip" => "DataGrip",
        "sublime_text" => "Sublime Text",
        "notepad++" => "Notepad++",
        "notepad" => "Notepad",
        _ => file_name.as_str(),
    };

    mapped.to_string()
}

/// Basename stem that works for both Windows (`\`) and Unix (`/`) paths.
fn process_stem(path: &str) -> Option<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let basename = trimmed
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(trimmed)
        .trim();

    if basename.is_empty() {
        return None;
    }

    let stem = Path::new(basename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(basename);

    Some(stem.to_string())
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
        assert_eq!(
            friendly_name_from_process(
                "cursor.exe",
                r"C:\Users\me\AppData\Local\Programs\cursor\Cursor.exe"
            ),
            "Cursor"
        );
        assert_eq!(
            friendly_name_from_process(
                "notion.exe",
                r"C:\Users\me\AppData\Local\Programs\Notion\Notion.exe"
            ),
            "Notion"
        );
        // Unix-style path should also map.
        assert_eq!(
            friendly_name_from_process("code", "/usr/share/code/code"),
            "Visual Studio Code"
        );
        // Prefer exec_name when path is empty.
        assert_eq!(
            friendly_name_from_process("chrome.exe", ""),
            "Google Chrome"
        );
    }
}
