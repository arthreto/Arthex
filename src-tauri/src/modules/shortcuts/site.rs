//! Website addresses: http and https only, with a real host name.

use tauri::Url;

pub fn parse(input: &str) -> Option<Url> {
    let input = input.trim();
    if input.is_empty() || input.len() > 2048 {
        return None;
    }
    let url = if has_scheme(input) {
        Url::parse(input)
    } else {
        Url::parse(&format!("https://{input}"))
    }
    .ok()?;
    let host = url.host_str()?;
    (matches!(url.scheme(), "http" | "https") && host.contains('.')).then_some(url)
}

/// `mail.google.com` gives `Mail`, `www.github.com` gives `Github`.
pub fn default_name(url: &Url) -> String {
    let host = url.host_str().unwrap_or_default();
    let label = host
        .strip_prefix("www.")
        .unwrap_or(host)
        .split('.')
        .next()
        .unwrap_or_default();
    let mut chars = label.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn has_scheme(input: &str) -> bool {
    input.split_once(':').is_some_and(|(scheme, _)| {
        scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "+.-".contains(c))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_bare_hosts_and_http_urls() {
        assert_eq!(
            parse("youtube.com").unwrap().as_str(),
            "https://youtube.com/"
        );
        assert_eq!(
            parse(" http://example.org/a?b ").unwrap().as_str(),
            "http://example.org/a?b"
        );
    }

    #[test]
    fn rejects_other_schemes_and_hosts_without_a_dot() {
        for input in [
            "",
            "javascript:alert(1)",
            "file:///C:/Windows",
            "ftp://example.org",
            "intranet",
            "localhost:3000",
        ] {
            assert!(parse(input).is_none(), "{input}");
        }
    }

    #[test]
    fn names_a_site_after_its_host() {
        assert_eq!(default_name(&parse("www.github.com").unwrap()), "Github");
        assert_eq!(default_name(&parse("mail.google.com").unwrap()), "Mail");
    }
}
