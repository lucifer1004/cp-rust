use std::env;
use std::fs::File;
use std::io::{copy, Error};
use std::process::{Command, Stdio};

fn show_help() {
  println!("Usage:\ncreate [file]: create a new file\nexec [file]: exec a file");
}

fn main() -> Result<(), Error> {
  let args: Vec<String> = env::args().collect();
  if args.len() <= 2 {
    show_help();
    return Ok(());
  }

  match &args[1][..] {
    "create" => {
      for i in 2..args.len() {
        File::create(format!("src/bin/{}.rs", &args[i]))?;
      }
    }
    "exec" => {
      let mut child = Command::new("sh")
        .arg("-c")
        .arg(format!("cargo run --bin {}", &args[2]))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

      let input_path = "test.in";

      let mut file = File::open(input_path).ok().expect("failed to open file");
      copy(&mut file, child.stdin.as_mut().unwrap())?;
      let output = child.wait_with_output().unwrap();

      println!("{}", String::from_utf8(output.stdout).unwrap());
    }
    _ => show_help(),
  }

  Ok(())
}
