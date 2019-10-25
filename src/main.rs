use std::env;
use std::fs::File;
use std::io::{copy, Error};
use std::process::{Command, Stdio};

fn show_warning() {
  print!(
    r#"
  You should enter a valid command!
  Available commands: create|[n]ew|[c]ommit|[e]xec|[h]elp
    "#
  );
}

fn show_help() {
  println!(
    r#"
  Usage:
  create|new|n [file1] [file2] [...]: create new files
  commit|c [file]: git add and commit a file
  exec|e [file]: exec a file
  help|h: show help message
  "#
  );
}

fn main() -> Result<(), Error> {
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    show_warning();
    show_help();
    return Ok(());
  }

  match &args[1][..] {
    "create" | "new" | "n" => {
      for i in 2..args.len() {
        File::create(format!("src/bin/{}.rs", &args[i]))?;
      }
    }
    "commit" | "c" => {
      Command::new("sh").arg("-c").arg(format!(
        "git add ./src/bin/{}.rs && git commit -m \"{}\"",
        &args[2], &args[2]
      ));
    }
    "exec" | "e" => {
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
    "help" | "h" => show_help(),
    _ => {
      show_warning();
      show_help();
    }
  }

  Ok(())
}
