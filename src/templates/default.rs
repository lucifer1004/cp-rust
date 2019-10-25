use std::io;

fn main() {
  let mut num = String::new();
  io::stdin().read_line(&mut num).unwrap();
  let n: i32 = num.trim().parse().unwrap();

  println!("{}", n);
}
