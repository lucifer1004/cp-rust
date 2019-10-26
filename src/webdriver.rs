use std::process::{Command, Stdio};

pub fn init() {
  let ps = Command::new("sh")
    .arg("-c")
    .arg("ps")
    .arg("-ef")
    .stdout(Stdio::piped())
    .spawn()
    .expect("failed to start web driver");
  let output = ps.wait_with_output().unwrap();
  let output = String::from_utf8(output.stdout).unwrap();
  if !output.contains("geckodriver") {
    Command::new("sh")
      .arg("-c")
      .arg("./third/geckodriver")
      .spawn()
      .expect("failed to start web driver");
  }
}
