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
  for (_pid, process) in system.get_process_list() {
    if process.name() == "geckodriver" {
      driver_ready = true;
      break;
    }
  }

  if !driver_ready {
    Command::new("./third/geckodriver")
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
