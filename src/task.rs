use std::fmt;
use nannou_osc as osc;
use inquire::{ Text };

use super::parser;
use super::prompt;

pub enum Task {
  Monitor(String),
  Send(String)
}

impl fmt::Display for Task {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &*self {
      Task::Monitor(val) => write!(f, "monitor = {}", val),
      Task::Send(val) => write!(f, "send = {}", val),
    }
  }
}

pub fn monitor(port: u16) { 
  let recv = osc::receiver(port).expect("Could not connect to receiver address"); 
  loop {
    println!("{:?}", recv.recv().unwrap());
  }
}

pub fn send(port: u16, address: String) {
  Text::new( 
    &format!( "{} {} {} {} {} {}",
      format!("Sending OSC messages to {:?}: {:?} \n",address, port),
      "Use the following format to send messages: <address> <value>\n",
      "\t - <address> is osc path to communicate with\n",
      "\t - <value> is a number or a string without wrapping in double quotes (can have multiple values) \n",
      "\t Example: /s_new default -1 0 0 freq 850\n",
      "\t will be parsed as (\"s_new\",[String(\"default\"), Int(-1), Int(0), Int(0), String(\"freq\"), Int(850)])"
    )
  ).prompt_skippable().unwrap();

  loop {
    let msg = Text::new("");
    let osc_msg = msg.with_render_config(prompt::get_render_config()).prompt().unwrap();
    let osc_msg_vec = osc_msg.split(' ').into_iter().collect::<Vec<&str>>();

    if let Some((first, tail)) = osc_msg_vec.split_first() {
      let osc_path = first;
      let argument_msg = tail.into_iter().map(|x| parser::parse_message(x.to_string())).collect();
      parser::msg_parse(port, address.clone(), osc_path ,argument_msg);
    }
  }
}
