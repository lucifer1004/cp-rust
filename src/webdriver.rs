use std::process::Command;

use fantoccini::Client;
use sysinfo::{ProcessExt, SystemExt};

#[derive(Clone)]
pub struct Session {
  pub client: Option<Client>,
}

pub async fn init() -> Session {
  let mut system = sysinfo::System::new();
  system.refresh_all();

  let mut driver_ready = false;
  for (_pid, process) in system.get_processes() {
    if process.name().contains("geckodriver") {
      driver_ready = true;
      break;
    }
  }

  if !driver_ready {
    let gecko_binary: &str;
    if cfg!(target_os = "linux") {
      gecko_binary = "./third/geckodriver";
    } else {
      gecko_binary = "./third/geckodriver-mac";
    }
    Command::new(gecko_binary)
      .spawn()
      .expect("failed to start web driver");
  }

  Session {
    client: Some(
      Client::new("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver"),
    ),
  }
}
