use inquire::{
  error::InquireResult, Select, Text, CustomType,
  ui::{Attributes, Color, RenderConfig, StyleSheet, Styled},
};

use std::fmt;
use nannou_osc as osc;
use std::str::FromStr;

#[derive(PartialEq, Debug)]
enum Val {
  I32(i32),
  F32(f32),
  F64(f64),
  String(String),
}

impl FromStr for Val {
    type Err = & 'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match (s.parse::<i32>(), s.parse::<f32>(), s.parse::<f64>(), s.parse::<String>()) {
            (Ok(i),_,_,_)     => Ok(Val::I32(i)),
            (_,Ok(f),_,_)     => Ok(Val::F32(f)),
            (_,_,Ok(f),_)     => Ok(Val::F64(f)),
            (_,_,_,Ok(_s))    => Ok(Val::String(_s)),
            _    => Err("Unrecognized type."),
        }
    }
}


pub fn parse_message(message: String) -> osc::Type {
  parse_message_auto(message)
}

fn parse_message_auto(message: String) -> osc::Type {
  let parsed = message.parse::<Val>().unwrap();
  match parsed {
    Val::I32(val) => osc::Type::Int(val),
    Val::F32(val) => osc::Type::Float(val),
    Val::F64(val) => osc::Type::Double(val),
    Val::String(val) => osc::Type::String(val),
  }
}

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 57110;

enum Task {
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

fn get_render_config() -> RenderConfig {
  let mut render_config = RenderConfig::default();
  render_config.prompt_prefix = Styled::new(">").with_fg(Color::LightRed);
  render_config.error_message = render_config
      .error_message
      .with_prefix(Styled::new("❌").with_fg(Color::LightRed));

  render_config.answer = StyleSheet::new()
      .with_attr(Attributes::ITALIC)
      .with_fg(Color::LightYellow);

  render_config.help_message = StyleSheet::new().with_fg(Color::DarkYellow);

  render_config
}

fn msg_parse(port: u16, address: String, osc_path: &str, osc_args: Vec<osc::Type>) {
  let full_address = format!("{}:{}", address, port);
    
  let sender = osc::sender()
  .expect("Could not bind to default socket")
  .connect(full_address)
  .expect("Could not connect to socket at address");
  
  let packet = (osc_path, osc_args);
  sender.send(packet).ok();
}

fn monitor(port: u16) { 
  let recv = osc::receiver(port).expect("Could not connect to receiver address"); 
  loop {
    println!("{:?}", recv.recv().unwrap());
  }
}

fn send(port: u16, address: String) {
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
    let osc_msg = msg.with_render_config(get_render_config()).prompt().unwrap();
    let osc_msg_vec = osc_msg.split(' ').into_iter().collect::<Vec<&str>>();

    if let Some((first, tail)) = osc_msg_vec.split_first() {
      let osc_path = first;
      let argument_msg = tail.into_iter().map(|x| parse_message(x.to_string())).collect();
      msg_parse(port, address.clone(), osc_path ,argument_msg);
    }
  }
}

fn main() -> InquireResult<()> {
  let tasks = vec![
    Task::Monitor("Monitor OSC messages".to_string()),
    Task::Send("Send OSC messages".to_string()),
  ];

  let task = Select::new("What do you want to do?", tasks).prompt().unwrap();
  let port: u16 = CustomType::new("What port do you want to connect to?")
        .with_default(( DEFAULT_PORT, &|i| format!("{}", i) ))
        .with_error_message("Please type a valid number")
        .prompt()
        .unwrap();
  let address: String = Text::new("What host IP do you want to connect to?")
        .with_default(DEFAULT_IP)
        .prompt()
        .unwrap();


  match task {
    Task::Monitor(_) => monitor(port),
    Task::Send(_) => send(port, address),
  };


  Ok(())
}