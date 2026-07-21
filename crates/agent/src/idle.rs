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

#[cfg(target_os = "macos")]
mod platform {
    use std::process::Command;
    use std::time::Duration;

    pub fn idle_duration() -> anyhow::Result<Duration> {
        let output = Command::new("ioreg")
            .args(["-c", "IOHIDSystem", "-d", "4", "-r"])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("ioreg failed");
        }

        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("HIDIdleTime") {
                let nanos: u64 = line
                    .split('=')
                    .nth(1)
                    .and_then(|part| part.trim().split_whitespace().next())
                    .and_then(|value| value.parse().ok())
                    .ok_or_else(|| anyhow::anyhow!("failed to parse HIDIdleTime"))?;
                return Ok(Duration::from_nanos(nanos));
            }
        }

        anyhow::bail!("HIDIdleTime not found")
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use std::process::Command;
    use std::time::Duration;

    pub fn idle_duration() -> anyhow::Result<Duration> {
        let output = Command::new("xprintidle")
            .output()
            .map_err(|_| anyhow::anyhow!("xprintidle not installed"))?;

        if !output.status.success() {
            anyhow::bail!("xprintidle failed");
        }

        let ms: u64 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid xprintidle output"))?;

        Ok(Duration::from_millis(ms))
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
        // On CI / unsupported platforms this may return false when detection fails.
        let _ = is_user_idle(std::time::Duration::from_secs(60 * 60));
    }
}
