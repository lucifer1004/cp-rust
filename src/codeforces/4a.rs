use std::io;

fn main() {
  let mut weight = String::new();
  io::stdin().read_line(&mut weight).unwrap();
  let weight: u32 = weight.trim().parse().unwrap();
  if weight % 2 == 0 && weight >= 4 {
    println!("YES");
  } else {
    println!("NO");
  }
}
