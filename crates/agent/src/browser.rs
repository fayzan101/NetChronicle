use regex::Regex;
use std::sync::LazyLock;

const BROWSER_APPS: &[&str] = &[
    "chrome", "firefox", "msedge", "edge", "brave", "opera", "vivaldi",
];

static DOMAIN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b([a-z0-9][-a-z0-9]*(?:\.[a-z0-9][-a-z0-9]*)+\.[a-z]{2,})\b").unwrap());

pub fn is_browser(app_name: &str) -> bool {
    let lower = app_name.to_lowercase();
    BROWSER_APPS.iter().any(|b| lower.contains(b))
}

pub fn extract_domain(title: &str) -> Option<String> {
    DOMAIN_RE
        .find(title)
        .map(|m| m.as_str().to_lowercase())
}

pub fn browser_url_proxy(title: &str, domain: &str) -> String {
    if title.to_lowercase().contains(domain) {
        format!("https://{domain}")
    } else {
        format!("https://{domain}/")
    }
}
