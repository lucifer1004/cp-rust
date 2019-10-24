use std::env;
use std::fs::File;
use std::io::Error;

fn main() -> Result<(), Error> {
  let args: Vec<String> = env::args().collect();
  if args.len() <= 1 {
    panic!("Too few arguments!");
  }

  for i in 1..args.len() {
    File::create(format!("src/bin/{}.rs", &args[i]))?;
  }
  Ok(())
}
