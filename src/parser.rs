use nannou_osc as osc;
use regex::Regex;
use std::str::FromStr;

#[derive(PartialEq, Debug)]
enum Val {
  I32(i32),
  I64(i64),
  F32(f32),
  F64(f64),
  Boolean(bool),
  String(String),
  Char(char),
  Nil,
  Inf,
}

// TODO: ? use macros
impl FromStr for Val {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match (
      s == "Nil",
      s == "Inf",
      s.parse::<i32>(),
      s.parse::<i64>(),
      s.parse::<f32>(),
      s.parse::<f64>(),
      s.parse::<char>(),
      s.parse::<bool>(),
      s.parse::<String>(),
    ) {
      (true, _, _, _, _, _, _, _, _) => Ok(Val::Nil),
      (_, true, _, _, _, _, _, _, _) => Ok(Val::Inf),
      (_, _, Ok(i), _, _, _, _, _, _) => Ok(Val::I32(i)),
      (_, _, _, Ok(i), _, _, _, _, _) => Ok(Val::I64(i)),
      (_, _, _, _, Ok(f), _, _, _, _) => Ok(Val::F32(f)),
      (_, _, _, _, _, Ok(f), _, _, _) => Ok(Val::F64(f)),
      (_, _, _, _, _, _, Ok(c), _, _) => Ok(Val::Char(c)),
      (_, _, _, _, _, _, _, Ok(b), _) => Ok(Val::Boolean(b)),
      (_, _, _, _, _, _, _, _, Ok(st)) => Ok(Val::String(st)),
      _ => Err("Unrecognized type."),
    }
  }
}

pub fn parse_message(message: String) -> osc::Type {
  parse_message_auto(message)
}

fn parse_message_auto(message: String) -> osc::Type {
  // remove single/double quotes
  let re = Regex::new(r#"^['"](.*)['"]$"#).unwrap();
  let replace = re.replace(&message, "$1");

  // handle implicit conversion.
  // check number type eg.`1.234_f64` is equivalent to `1.234 as f64`
  let number_types = Regex::new(r"(_i32)$|(_i64)$|(_f32)$|(_f64)$").unwrap();
  if number_types.is_match(&replace) {
    let m = &replace.to_string();
    let num: Vec<_> = Regex::new(r"[_]").unwrap().split(m).collect();
    return match num[1] {
      "i32" => osc::Type::Int(num[0].parse::<i32>().unwrap()),
      "i64" => osc::Type::Long(num[0].parse::<i64>().unwrap()),
      "f32" => osc::Type::Float(num[0].parse::<f32>().unwrap()),
      "f64" => osc::Type::Double(num[0].parse::<f64>().unwrap()),
      _ => osc::Type::Nil,
    };
  }

  let parsed = replace.parse::<Val>().unwrap();
  match parsed {
    Val::I32(val) => osc::Type::Int(val),
    Val::I64(val) => osc::Type::Long(val),
    Val::F32(val) => osc::Type::Float(val),
    Val::F64(val) => osc::Type::Double(val),
    Val::Char(val) => osc::Type::Char(val),
    Val::Boolean(val) => osc::Type::Bool(val),
    Val::String(val) => osc::Type::String(val),
    Val::Nil => osc::Type::Nil,
    Val::Inf => osc::Type::Inf,
  }
}

// /s_new "default" 'V' 'tbC' freq 0 -1 1.234_f64
