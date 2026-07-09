#[cfg(windows)]
mod platform {
    use std::time::Duration;

    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    pub fn idle_duration() -> anyhow::Result<Duration> {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };

        unsafe {
            if !GetLastInputInfo(&mut info).as_bool() {
                anyhow::bail!("GetLastInputInfo failed");
            }
        }

        let tick = unsafe { windows::Win32::System::SystemInformation::GetTickCount() };
        let elapsed_ms = tick.saturating_sub(info.dwTime);
        Ok(Duration::from_millis(elapsed_ms as u64))
    }
}

#[cfg(not(windows))]
mod platform {
    use std::time::Duration;

    pub fn idle_duration() -> anyhow::Result<Duration> {
        Ok(Duration::from_secs(0))
    }
}

pub use platform::idle_duration;

pub fn is_user_idle(threshold: std::time::Duration) -> bool {
    idle_duration()
        .map(|duration| duration >= threshold)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_threshold_check() {
        assert!(!is_user_idle(std::time::Duration::from_secs(60 * 60)));
    }
}
