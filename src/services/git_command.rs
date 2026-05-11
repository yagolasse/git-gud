//! Git system command abstraction layer.
//!
//! Centralizes all system `git` invocations behind a configurable path,
//! canonical environment variables, and classified error types.
//! Provides both synchronous (transitional) and async (tokio) runners.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

static GIT_CONFIG: OnceLock<GitConfig> = OnceLock::new();

pub fn init_config(config: GitConfig) {
    let _ = GIT_CONFIG.set(config);
}

pub fn config() -> &'static GitConfig {
    GIT_CONFIG.get_or_init(|| GitConfig::default())
}

/// Configuration for the system git binary
#[derive(Debug, Clone)]
pub struct GitConfig {
    /// Path to the git binary (default "git")
    pub binary: PathBuf,
    /// Path to the GIT_ASKPASS helper (set to talk to the GUI)
    pub askpass: Option<PathBuf>,
    /// Port the GUI is listening on for askpass IPC
    pub askpass_port: Option<u16>,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            binary: PathBuf::from("git"),
            askpass: None,
            askpass_port: None,
        }
    }
}

impl GitConfig {
    /// Builds the environment-variable overrides for a git subprocess
    pub fn env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("GIT_TERMINAL_PROMPT".into(), "0".into());
        vars.insert("GCM_INTERACTIVE".into(), "Never".into());
        if let Some(ref askpass) = self.askpass {
            vars.insert("GIT_ASKPASS".into(), askpass.to_string_lossy().to_string());
            if let Some(port) = self.askpass_port {
                vars.insert("GIT_GUD_ASKPASS_PORT".into(), port.to_string());
            }
        }
        vars
    }
}

/// Classified error for git command failures
#[derive(Debug)]
pub enum GitCommandError {
    Auth(String),
    Network(String),
    RemoteRejected(String),
    Transport(String),
    Exit(i32, String),
    Spawn(String),
}

impl GitCommandError {
    pub fn is_auth(&self) -> bool {
        matches!(self, Self::Auth(_))
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Auth(s) | Self::Network(s) | Self::RemoteRejected(s) | Self::Transport(s) => s,
            Self::Exit(_, s) => s,
            Self::Spawn(s) => s,
        }
    }
}

impl std::fmt::Display for GitCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth(s) => write!(f, "Authentication failed: {}", s),
            Self::Network(s) => write!(f, "Network error: {}", s),
            Self::RemoteRejected(s) => write!(f, "Remote rejected: {}", s),
            Self::Transport(s) => write!(f, "Transport error: {}", s),
            Self::Exit(code, s) => write!(f, "Git exited with code {}: {}", code, s),
            Self::Spawn(s) => write!(f, "Could not start git: {}", s),
        }
    }
}

fn classify(stderr: &str, exit_code: i32) -> GitCommandError {
    let stderr_lower = &stderr.to_lowercase();
    let msg = stderr.trim().to_string();

    if stderr_lower.contains("permission denied")
        && (stderr_lower.contains("publickey") || stderr_lower.contains("password"))
    {
        return GitCommandError::Auth(msg);
    }
    if stderr_lower.contains("could not resolve host")
        || stderr_lower.contains("could not resolve hostname")
        || stderr_lower.contains("failed to connect")
        || stderr_lower.contains("connection refused")
        || stderr_lower.contains("connection reset")
        || stderr_lower.contains("network is unreachable")
    {
        return GitCommandError::Network(msg);
    }
    if stderr_lower.contains("[rejected]") || stderr_lower.contains("non-fast-forward") {
        return GitCommandError::RemoteRejected(msg);
    }
    if stderr_lower.contains("host key verification failed")
        || stderr_lower.contains("no matching host key")
    {
        return GitCommandError::Auth(msg);
    }

    if exit_code != 0 {
        GitCommandError::Exit(exit_code, msg)
    } else {
        GitCommandError::Transport(msg)
    }
}

/// Run a git command synchronously (blocking).
pub fn run_blocking(
    repo_path: &std::path::Path,
    args: &[&str],
) -> Result<String, GitCommandError> {
    let cfg = config();

    let mut cmd = std::process::Command::new(&cfg.binary);
    cmd.args(args).current_dir(repo_path);

    for (key, val) in cfg.env_vars() {
        cmd.env(key, val);
    }

    let output = cmd.output().map_err(|e| GitCommandError::Spawn(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(classify(&stderr, output.status.code().unwrap_or(-1)))
    }
}

/// Run a git command asynchronously via tokio.
#[allow(dead_code)]
pub async fn run_async(
    repo_path: &std::path::Path,
    args: &[&str],
) -> Result<String, GitCommandError> {
    let cfg = config();

    let mut cmd = tokio::process::Command::new(&cfg.binary);
    cmd.args(args).current_dir(repo_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    for (key, val) in cfg.env_vars() {
        cmd.env(key, val);
    }

    let output = cmd.output().await.map_err(|e| GitCommandError::Spawn(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(classify(&stderr, output.status.code().unwrap_or(-1)))
    }
}

/// Run a git command asynchronously and stream lines as they arrive.
/// Returns a receiver for (line, is_stderr) pairs.
#[allow(dead_code)]
pub fn run_streaming(
    repo_path: &std::path::Path,
    args: &[&str],
) -> Result<tokio::sync::mpsc::Receiver<(String, bool)>, GitCommandError> {
    let cfg = config();

    let mut cmd = tokio::process::Command::new(&cfg.binary);
    cmd.args(args).current_dir(repo_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    for (key, val) in cfg.env_vars() {
        cmd.env(key, val);
    }

    let mut child = cmd.spawn().map_err(|e| GitCommandError::Spawn(e.to_string()))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let (tx, rx) = tokio::sync::mpsc::channel::<(String, bool)>(64);

    if let Some(stdout) = stdout {
        let tx = tx.clone();
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx.send((line, false)).await.is_err() {
                    break;
                }
            }
        });
    }

    if let Some(stderr) = stderr {
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let mut reader = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx.send((line, true)).await.is_err() {
                    break;
                }
            }
        });
    }

    Ok(rx)
}
