//! Vedic Astrology web server: the self-hosted website, without the desktop window.
//!
//! ```text
//! vedic_astrology_web [--listen 0.0.0.0:8080] [--knowledge <folder>]
//! vedic_astrology_web --import-texts <knowledge folder> [--personal]
//! vedic_astrology_web --health-check [--listen 127.0.0.1:8080]
//! ```
//!
//! Environment: VA_LISTEN, VA_KNOWLEDGE, and the AI writer's VA_AI_BASE_URL, VA_AI_API_KEY,
//! VA_AI_MODEL, VA_AI_USER, VA_AI_ORG (see docs/PACKAGING.md).

use vedic_astrology::{import, knowledge, web};
use std::io::{Read, Write};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    let listen = arg("--listen").or_else(|| std::env::var("VA_LISTEN").ok()).unwrap_or_else(|| "0.0.0.0:8080".into());

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("vedic_astrology_web [--listen ADDR:PORT] [--knowledge DIR] | --import-texts DIR [--personal] | --health-check");
        return;
    }
    // Container health check: the runtime image has no shell or curl, so the program checks itself.
    if args.iter().any(|a| a == "--health-check") {
        std::process::exit(if health(&listen) { 0 } else { 1 });
    }
    if let Some(dir) = arg("--import-texts") {
        let sum = import::import(std::path::Path::new(&dir), args.iter().any(|a| a == "--personal"), usize::MAX, &mut |l| eprintln!("{l}"));
        for e in &sum.errors {
            eprintln!("problem: {e}");
        }
        std::process::exit(if sum.files > 0 { 0 } else { 1 });
    }
    let knowledge_dir = arg("--knowledge").or_else(|| std::env::var("VA_KNOWLEDGE").ok()).map(PathBuf::from).unwrap_or_else(knowledge::default_dir);
    if let Err(e) = web::serve(&listen, knowledge_dir) {
        eprintln!("could not serve on {listen}: {e}");
        std::process::exit(1);
    }
}

/// GET /health on the local port.
fn health(listen: &str) -> bool {
    let port = listen.rsplit(':').next().unwrap_or("8080");
    let Ok(mut s) = std::net::TcpStream::connect(format!("127.0.0.1:{port}")) else { return false };
    let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(5)));
    if s.write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").is_err() {
        return false;
    }
    let mut reply = String::new();
    let _ = s.read_to_string(&mut reply);
    reply.starts_with("HTTP/1.1 200")
}
