#![allow(missing_docs)]

mod parser;
mod prompt;
mod task;

use prompt::prompt;

pub const DEFAULT_IP: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 57110;

fn main() {
  prompt().unwrap();
}
