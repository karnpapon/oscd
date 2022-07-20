mod parser;
mod task;
mod prompt;

use prompt::prompt;

pub const DEFAULT_IP: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 57110;

fn main() {
  prompt().unwrap();
}
