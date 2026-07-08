const DEFAULT_IGNORE: &[&str] = &[
    "",
    "Program Manager",
    "Windows Shell Experience Host",
    "LockApp",
    "Search",
    "Start",
];

pub fn should_ignore(app_name: &str, window_title: &str, extra: &[String]) -> bool {
    let app = app_name.trim();
    let title = window_title.trim();

    if app.is_empty() && title.is_empty() {
        return true;
    }

    for pattern in DEFAULT_IGNORE {
        if app.eq_ignore_ascii_case(pattern) || title.eq_ignore_ascii_case(pattern) {
            return true;
        }
    }

    for pattern in extra {
        if app.eq_ignore_ascii_case(pattern) || title.contains(pattern) {
            return true;
        }
    }

    false
}

pub fn ignore_list_from_env() -> Vec<String> {
    std::env::var("AGENT_IGNORE_APPS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_program_manager() {
        assert!(should_ignore("Program Manager", "Desktop", &[]));
    }

    #[test]
    fn allows_normal_apps() {
        assert!(!should_ignore("Code", "main.rs - project", &[]));
    }
}
