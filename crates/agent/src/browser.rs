use regex::Regex;
use std::sync::LazyLock;

const BROWSER_APPS: &[&str] = &[
    "chrome", "firefox", "msedge", "edge", "brave", "opera", "vivaldi",
];

static DOMAIN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b((?:[a-z0-9](?:[-a-z0-9]*[a-z0-9])?\.)+[a-z]{2,})\b").unwrap()
});

static URL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)(https?://[^\s]+)").unwrap());

static BROWSER_SUFFIXES: &[&str] = &[
    " - Google Chrome",
    " - Microsoft\u{200B} Edge", // includes zero-width char variant
    " - Microsoft Edge",
    " — Mozilla Firefox",
    " - Mozilla Firefox",
    " - Brave",
    " - Opera",
    " - Vivaldi",
    " | Microsoft Edge",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserContext {
    pub page_title: String,
    pub domain: Option<String>,
    pub url: String,
}

pub fn is_browser(app_name: &str) -> bool {
    let lower = app_name.to_lowercase();
    BROWSER_APPS.iter().any(|b| lower.contains(b))
}

pub fn extract_domain_from_url(url: &str) -> Option<String> {
    let url = url.trim();
    if let Some(stripped) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    {
        let host = stripped
            .split('/')
            .next()?
            .split('?')
            .next()?
            .split(':')
            .next()?;
        if host.contains('.') {
            return Some(host.to_lowercase());
        }
    }
    extract_domain(url)
}

pub fn extract_domain(text: &str) -> Option<String> {
    DOMAIN_RE.find(text).map(|m| m.as_str().to_lowercase())
}

static PROFILE_SUFFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" - Profile(?: \d+)?$").unwrap());

pub fn strip_browser_suffix(title: &str) -> String {
    let mut cleaned = title.trim().to_string();
    for suffix in BROWSER_SUFFIXES {
        if let Some(stripped) = cleaned.strip_suffix(suffix) {
            cleaned = stripped.trim().to_string();
            break;
        }
    }

    if let Some(stripped) = PROFILE_SUFFIX_RE.find(&cleaned) {
        cleaned = cleaned[..stripped.start()].trim().to_string();
    }

    cleaned
}

pub fn parse_browser_context(app_name: &str, window_title: &str) -> Option<BrowserContext> {
    if !is_browser(app_name) {
        return None;
    }

    let cleaned = strip_browser_suffix(window_title);

    if let Some(url_match) = URL_RE.find(&cleaned) {
        let url = url_match.as_str().trim_end_matches([')', ']']).to_string();
        let domain = extract_domain_from_url(&url);
        return Some(BrowserContext {
            page_title: cleaned,
            domain,
            url,
        });
    }

    if let Some(domain) = extract_domain(&cleaned) {
        return Some(BrowserContext {
            page_title: cleaned.clone(),
            domain: Some(domain.clone()),
            url: format!("https://{domain}"),
        });
    }

    Some(BrowserContext {
        page_title: cleaned,
        domain: None,
        url: String::new(),
    })
}

pub fn browser_url_proxy(context: &BrowserContext) -> String {
    if !context.url.is_empty() {
        return context.url.clone();
    }
    if let Some(domain) = &context.domain {
        return format!("https://{domain}");
    }
    context.page_title.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_chrome_suffix() {
        let ctx = parse_browser_context("chrome.exe", "GitHub - Google Chrome").unwrap();
        assert_eq!(ctx.page_title, "GitHub");
        assert_eq!(ctx.domain.as_deref(), None);
    }

    #[test]
    fn extracts_domain_from_title() {
        let ctx = parse_browser_context(
            "msedge",
            "Pull Requests · github.com/user/repo - Profile 1 - Microsoft Edge",
        )
        .unwrap();
        assert_eq!(ctx.domain.as_deref(), Some("github.com"));
    }

    #[test]
    fn extracts_url_from_title() {
        let ctx = parse_browser_context(
            "firefox",
            "https://stackoverflow.com/questions/1/test — Mozilla Firefox",
        )
        .unwrap();
        assert_eq!(ctx.domain.as_deref(), Some("stackoverflow.com"));
        assert!(ctx.url.starts_with("https://"));
    }
}
