//! An inert subtree an opaque overlay buries, against an installed browser
//! (issue #950). Skips cleanly when there is none.
//!
//! The should-pass views in `inert-under-overlay.html` sit beside three
//! should-flag twins, each differing in the one fact the engine reads: no
//! `inert`, a translucent overlay, no overlay. The base engine (e3818550)
//! reports the should-pass views under every rule named here too.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

use impeccable_browser::BrowserEngine;
use impeccable_detect::engines::{ScanOptions, UrlEngine};

const FIXTURE: &str = "inert-under-overlay.html";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/antipatterns")
}

fn serve() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            std::thread::spawn(move || handle(stream));
        }
    });
    port
}

fn handle(mut stream: TcpStream) {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf).unwrap_or(0);
    let request = String::from_utf8_lossy(&buf[..n]);
    let path = request
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_string();
    let body = std::fs::read(fixtures_dir().join(path.trim_start_matches('/'))).unwrap_or_default();
    let head = format!(
        "HTTP/1.0 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&body);
}

fn engine() -> Option<BrowserEngine> {
    let env: HashMap<String, String> = std::env::vars().collect();
    if impeccable_browser::discovery::find_browser(&env).is_err() {
        eprintln!("skip: no installed browser found");
        return None;
    }
    Some(BrowserEngine::new(env))
}

#[test]
fn an_inert_view_an_opaque_overlay_buries_is_not_scored() {
    let Some(engine) = engine() else { return };
    let port = serve();
    let url = format!("http://127.0.0.1:{port}/{FIXTURE}");
    let findings = engine.detect_url(&url, &ScanOptions::default()).expect("scan");
    let flagged: Vec<(String, String)> = findings
        .iter()
        .map(|f| {
            (
                f.antipattern.clone(),
                f.extras.get("selector").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            )
        })
        .collect();
    let on = |rule: &str, selector: &str| flagged.iter().any(|(r, s)| r == rule && s.starts_with(selector));

    for want in [
        ("text-occlusion", "#flag-live-card"),
        ("text-occlusion", "#flag-live-status"),
        ("nested-cards", "#flag-live-card"),
        ("cramped-padding", "#flag-live-status"),
        ("nested-cards", "#flag-glass-card"),
        ("cramped-padding", "#flag-glass-status"),
        ("nested-cards", "#flag-shown-card"),
        ("cramped-padding", "#flag-shown-status"),
    ] {
        assert!(on(want.0, want.1), "missing {want:?} in {flagged:#?}");
    }
    for gone in [
        ("text-occlusion", "#pass-buried-card"),
        ("text-occlusion", "#pass-buried-status"),
        ("nested-cards", "#pass-buried-card"),
        ("cramped-padding", "#pass-buried-status"),
        ("text-occlusion", "#pass-shell-card"),
        ("text-occlusion", "#pass-shell-status"),
        ("nested-cards", "#pass-shell-card"),
        ("cramped-padding", "#pass-shell-status"),
    ] {
        assert!(!on(gone.0, gone.1), "unexpected {gone:?} in {flagged:#?}");
    }
}
