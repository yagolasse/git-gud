use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshHost {
    pub pattern: String,
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub identity_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SshConfig {
    pub hosts: Vec<SshHost>,
}

impl SshConfig {
    pub fn load() -> Self {
        let path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".ssh")
            .join("config");

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => {
                log::debug!("No SSH config found at {:?}", path);
                return Self { hosts: Vec::new() };
            }
        };

        let mut hosts = Vec::new();
        let mut current: Option<SshHost> = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let (keyword, value) = match split_kv(trimmed) {
                Some(kv) => kv,
                None => continue,
            };

            if keyword.eq_ignore_ascii_case("host") {
                if let Some(host) = current.take() {
                    hosts.push(host);
                }
                current = Some(SshHost {
                    pattern: value.to_string(),
                    hostname: None,
                    port: None,
                    user: None,
                    identity_file: None,
                });
            } else if let Some(ref mut host) = current {
                match keyword.to_lowercase().as_str() {
                    "hostname" => host.hostname = Some(value.to_string()),
                    "port" => host.port = value.parse().ok(),
                    "user" => host.user = Some(value.to_string()),
                    "identityfile" => {
                        let resolved = resolve_tilde(PathBuf::from(value));
                        host.identity_file = Some(resolved);
                    }
                    _ => {}
                }
            }
        }

        if let Some(host) = current {
            hosts.push(host);
        }

        log::debug!("Loaded {} SSH hosts", hosts.len());
        Self { hosts }
    }

    pub fn find_host(&self, alias: &str) -> Option<&SshHost> {
        self.hosts.iter().find(|h| match_pattern(&h.pattern, alias))
    }
}

fn split_kv(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, |c: char| c.is_ascii_whitespace() || c == '=');
    let key = parts.next()?;
    let val = parts.next()?.trim().trim_matches('"');
    Some((key.trim(), val))
}

fn resolve_tilde(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&s[2..]);
        }
    }
    path
}

fn match_pattern(pattern: &str, alias: &str) -> bool {
    pattern
        .split(' ')
        .any(|p| p == alias || p == "*" || (p.starts_with('*') && alias.ends_with(&p[1..])))
}
