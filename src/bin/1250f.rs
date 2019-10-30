use std::cmp::min;
use std::io;

fn main() {
  let mut num = String::new();
  io::stdin().read_line(&mut num).unwrap();
  let n: i32 = num.trim().parse().unwrap();

  let mut ans = 500000;
  for i in 1..n + 1 {
    if n % i == 0 {
      ans = min(ans, 2 * (i + n / i));
    }
    if i * i > n {
      break;
    }
  }

  println!("{}", ans);
}
