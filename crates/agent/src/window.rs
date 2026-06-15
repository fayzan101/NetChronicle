#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForegroundWindow {
    pub app_name: String,
    pub window_title: String,
}

pub fn current_foreground() -> anyhow::Result<ForegroundWindow> {
    let window = x_win::get_active_window().map_err(|e| anyhow::anyhow!("{e}"))?;
    let app_name = if window.info.exec_name.is_empty() {
        window.info.name.clone()
    } else {
        window.info.exec_name.clone()
    };

    Ok(ForegroundWindow {
        app_name,
        window_title: window.title,
    })
}
