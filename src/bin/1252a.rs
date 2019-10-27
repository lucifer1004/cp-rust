use std::io;

fn main() {
  let mut num = String::new();
  io::stdin().read_line(&mut num).unwrap();
  let n: i32 = num.trim().parse().unwrap();
  let mut perm = String::new();
  io::stdin().read_line(&mut perm).unwrap();
  for num in perm.split_whitespace() {
    print!("{} ", n + 1 - num.parse::<i32>().unwrap());
  }
}
