//! Askpass IPC bridge between system git and the GUI.
//!
//! When system git needs credentials (SSH passphrase, HTTPS password),
//! it calls the program set in `GIT_ASKPASS`. This module provides:
//!
//! - A TCP loopback server that runs inside the GUI
//! - A client subcommand that git spawns, which relays the prompt to the GUI
//!
//! ## Protocol
//!
//! 1. GUI spawns TCP listener on 127.0.0.1:0, stores the port
//! 2. `GIT_ASKPASS` is set to the git-gud binary, plus env vars for port
//! 3. Git invokes `git-gud askpass "Enter passphrase..."`  
//! 4. Client connects to 127.0.0.1:<port>, sends prompt + newline
//! 5. GUI reads prompt, stores request, shows dialog
//! 6. User types passphrase, GUI sends it back through TCP
//! 7. Client reads response, prints to stdout
//! 8. Git reads stdout

use std::sync::{Arc, Mutex, OnceLock};

static ASKPASS_STATE: OnceLock<AskpassState> = OnceLock::new();

pub fn set_state(state: AskpassState) {
    let _ = ASKPASS_STATE.set(state);
}

pub fn state() -> &'static AskpassState {
    ASKPASS_STATE.get().expect("askpass state not initialised")
}

/// A pending passphrase request
#[derive(Debug, Clone)]
pub struct PassphraseRequest {
    pub prompt: String,
}

/// Shared state between the TCP server and the GUI poll loop
pub type AskpassState = Arc<Mutex<AskpassRequests>>;

#[derive(Debug, Default)]
pub struct AskpassRequests {
    pub pending: Vec<PassphraseRequest>,
    pub response: Option<String>,
    pub server_port: Option<u16>,
}

impl AskpassRequests {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            response: None,
            server_port: None,
        }
    }
}

/// Start the TCP askpass server on a random port. Returns the bound port.
pub fn start_server(state: AskpassState) -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("failed to bind askpass server");
    let port = listener.local_addr().unwrap().port();

    {
        let mut s = state.lock().unwrap();
        s.server_port = Some(port);
    }

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };

            use std::io::{BufRead, BufReader, Write};
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut prompt = String::new();

            if reader.read_line(&mut prompt).is_err() {
                continue;
            }

            let request = PassphraseRequest {
                prompt: prompt.trim().to_string(),
            };

            let response = {
                let mut s = state.lock().unwrap();
                s.pending.push(request);

                while s.response.is_none() {
                    drop(s);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    s = state.lock().unwrap();
                    if s.pending.last().map(|r| r.prompt.as_str()) != Some(prompt.trim()) {
                        break;
                    }
                }

                s.response.take()
            };

            if let Some(ref pass) = response {
                let _ = writeln!(stream, "{}", pass);
            }
        }
    });

    port
}

/// Run the askpass client (invoked by git as a subprocess).
/// Connects to the running GUI, sends the prompt, prints the response to stdout.
pub fn run_client(prompt: &str, port: u16) {
    use std::io::{BufRead, BufReader, Write};

    let mut stream = match std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("askpass: could not connect to GUI: {}", e);
            std::process::exit(1);
        }
    };

    let _ = writeln!(stream, "{}", prompt);
    let _ = stream.flush();

    let mut reader = BufReader::new(&mut stream);
    let mut response = String::new();

    if reader.read_line(&mut response).is_err() {
        eprintln!("askpass: failed to read response");
        std::process::exit(1);
    }

    print!("{}", response.trim());
}
