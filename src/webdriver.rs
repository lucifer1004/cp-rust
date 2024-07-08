use std::process::Command;

use fantoccini::{Client, ClientBuilder};
use sysinfo::System;

#[derive(Clone)]
pub struct Session {
    pub client: Option<Client>,
}

pub async fn init() -> Session {
    let mut system = System::new();
    system.refresh_all();

    for (_pid, process) in system.processes() {
        if process.name().contains("geckodriver") {
            process.kill();
        }
    }

    let target_os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "macos"
    };

    let target_arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else {
        "aarch64"
    };

    let gecko_binary = format!(
        "./third/geckodriver-{}-{}{}",
        target_os,
        target_arch,
        if target_os == "windows" { ".exe" } else { "" }
    );

    Command::new(gecko_binary)
        .spawn()
        .expect("failed to start web driver");

    Session {
        client: Some(
            ClientBuilder::native()
                .connect("http://localhost:4444")
                .await
                .expect("failed to connect to WebDriver"),
        ),
    }
}
