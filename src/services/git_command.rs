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

/// A message produced by `run_streaming_std`.
pub enum StreamLine {
    /// One line of output from the git process (stdout or stderr).
    Output(String),
    /// The process has exited. `Ok(())` = success; `Err(msg)` = failure with git's stderr.
    Done(Result<(), String>),
}

/// Spawn the git command in background threads and stream output lines back via a channel.
/// The final message on the channel is always `StreamLine::Done`.
pub fn run_streaming_std(
    repo_path: &std::path::Path,
    args: Vec<String>,
) -> Result<std::sync::mpsc::Receiver<StreamLine>, GitCommandError> {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::sync::mpsc;

    let cfg = config();
    let mut cmd = std::process::Command::new(&cfg.binary);
    cmd.args(&args).current_dir(repo_path).stdout(Stdio::piped()).stderr(Stdio::piped());
    for (k, v) in cfg.env_vars() {
        cmd.env(k, v);
    }

    let mut child = cmd.spawn().map_err(|e| GitCommandError::Spawn(e.to_string()))?;
    let (tx, rx) = mpsc::channel::<StreamLine>();
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");

    // Stderr reader: sends each trimmed line as Output, accumulates for final error message.
    let tx_err = tx.clone();
    let (stderr_done_tx, stderr_done_rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut acc = String::new();
        for line in BufReader::new(stderr).lines().flatten() {
            let line = line.trim_end_matches('\r').to_string();
            if !acc.is_empty() { acc.push('\n'); }
            acc.push_str(&line);
            let _ = tx_err.send(StreamLine::Output(line));
        }
        let _ = stderr_done_tx.send(acc);
    });

    // Stdout reader + completion: reads stdout, then waits for the process and stderr thread.
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().flatten() {
            let line = line.trim_end_matches('\r').to_string();
            let _ = tx.send(StreamLine::Output(line));
        }
        let exit_ok = child.wait().map_or(false, |s| s.success());
        let stderr_msg = stderr_done_rx.recv().unwrap_or_default();
        let result = if exit_ok {
            Ok(())
        } else {
            let msg = stderr_msg.trim().to_string();
            Err(if msg.is_empty() { "git command failed".to_string() } else { msg })
        };
        let _ = tx.send(StreamLine::Done(result));
    });

    Ok(rx)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_auth_publickey() {
        let err = classify("Permission denied (publickey).", 1);
        assert!(matches!(err, GitCommandError::Auth(_)));
    }

    #[test]
    fn test_classify_auth_password() {
        let err = classify("Permission denied (password).", 1);
        assert!(matches!(err, GitCommandError::Auth(_)));
    }

    #[test]
    fn test_classify_auth_host_key() {
        let err = classify("Host key verification failed.", 1);
        assert!(matches!(err, GitCommandError::Auth(_)));
    }

    #[test]
    fn test_classify_network_resolve() {
        let err = classify("Could not resolve host: github.com", 1);
        assert!(matches!(err, GitCommandError::Network(_)));
    }

    #[test]
    fn test_classify_network_refused() {
        let err = classify("fatal: Connection refused", 1);
        assert!(matches!(err, GitCommandError::Network(_)));
    }

    #[test]
    fn test_classify_remote_rejected() {
        let err = classify("! [rejected] main -> main (non-fast-forward)", 1);
        assert!(matches!(err, GitCommandError::RemoteRejected(_)));
    }

    #[test]
    fn test_classify_remote_rejected_nff_only() {
        let err = classify("Updates were rejected because the tip of your current branch is behind its remote counterpart. non-fast-forward", 1);
        assert!(matches!(err, GitCommandError::RemoteRejected(_)));
    }

    #[test]
    fn test_classify_exit_unknown() {
        let err = classify("Some unknown git error occurred", 128);
        assert!(matches!(err, GitCommandError::Exit(128, _)));
    }

    #[test]
    fn test_classify_transport_on_zero_exit() {
        let err = classify("Some ambiguous message", 0);
        assert!(matches!(err, GitCommandError::Transport(_)));
    }

    #[test]
    fn test_error_message_accessible() {
        let err = classify("Permission denied (publickey).", 1);
        assert!(!err.message().is_empty());
    }

    #[test]
    fn test_is_auth() {
        assert!(classify("Permission denied (publickey).", 1).is_auth());
        assert!(!classify("Could not resolve host: github.com", 1).is_auth());
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
