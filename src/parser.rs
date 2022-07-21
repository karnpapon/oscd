use nannou_osc as osc;
use regex::Regex;
use std::str::FromStr;

#[derive(PartialEq, Debug)]
enum Val {
  I32(i32),
  F32(f32),
  F64(f64),
  Boolean(bool),
  String(String),
  Char(char),
  Nil,
  Inf,
}

// TODO: ? use macro
impl FromStr for Val {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match (
      s == "Nil",
      s == "Inf",
      s.parse::<i32>(),
      s.parse::<f32>(),
      s.parse::<f64>(),
      s.parse::<char>(),
      s.parse::<bool>(),
      s.parse::<String>(),
    ) {
      (true, _, _, _, _, _, _, _) => Ok(Val::Nil),
      (_, true, _, _, _, _, _, _) => Ok(Val::Inf),
      (_, _, Ok(i), _, _, _, _, _) => Ok(Val::I32(i)),
      (_, _, _, Ok(f), _, _, _, _) => Ok(Val::F32(f)),
      (_, _, _, _, Ok(f), _, _, _) => Ok(Val::F64(f)),
      (_, _, _, _, _, Ok(c), _, _) => Ok(Val::Char(c)),
      (_, _, _, _, _, _, Ok(b), _) => Ok(Val::Boolean(b)),
      (_, _, _, _, _, _, _, Ok(st)) => Ok(Val::String(st)),
      _ => Err("Unrecognized type."),
    }
  }
}

pub fn parse_message(message: String) -> osc::Type {
  parse_message_auto(message)
}

fn parse_message_auto(message: String) -> osc::Type {
  let re = Regex::new(r#"^['"](.*)['"]$"#).unwrap();
  let parsed = re.replace(&message, "$1").parse::<Val>().unwrap();
  match parsed {
    Val::I32(val) => osc::Type::Int(val),
    Val::F32(val) => osc::Type::Float(val),
    Val::F64(val) => osc::Type::Double(val),
    Val::Char(val) => osc::Type::Char(val),
    Val::Boolean(val) => osc::Type::Bool(val),
    Val::String(val) => osc::Type::String(val),
    Val::Nil => osc::Type::Nil,
    Val::Inf => osc::Type::Inf,
  }
}

// /s_new "default" 'V' 'tbC' freq 0 -1 123.4
