#![allow(missing_docs)]

use clap::*;

mod analyser;
mod osc;
mod prompt;
mod task;

use prompt::prompt;

pub const DEFAULT_IP: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 57110;

fn main() {
  let _app = clap_app!(oscd =>
    (version: "0.1.0")
    (author: "Karnpapon Boonput <karnpapon@gmail.com>")
    (about: "a simple interactive OSC debugger")
  )
  .get_matches();

  prompt().unwrap();
}
