use regex::Regex;
use rosc::OscType;
use std::str::FromStr;

use super::lexer::token::{Token};
use super::scanner::*;

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
  // U8(u8)
}

// TODO: find proper name
// enum Val2 {
//   U8(u8),
//   U32(u32)
// }

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

// impl FromStr for Val2 {
//   type Err = &'static str;
//   fn from_str(s: &str) -> Result<Self, Self::Err> {
//     match (
//       s.parse::<u8>(),
//       s.parse::<u32>(),
//     ) {
//       (Ok(v), _,) => Ok(Val2::U8(v)),
//       (_, Ok(v),) => Ok(Val2::U32(v)),
//       _ => Err("Unrecognized type."),
//     }
//   }
// }

pub fn parse_message(message: &Token) -> OscType {
  // match message {
  //   (m, DataType::Scalar) => parse_scalar(m),
  //   (m, DataType::Compound) => parse_compound(m),
  // }

  match message {
    Token::Ident(val) | Token::StringLiteral(val) => OscType::String(val.clone()),
    Token::IntLiteral(val) => OscType::Int(val.clone()),
    Token::FloatLiteral(val) => OscType::Float(val.clone()),
    Token::BoolLiteral(val) => OscType::Bool(val.clone()),
    _ => OscType::Nil 
  }
}

// fn parse_compound(message: String) -> OscType {
//   let osc_msg_vec = message
//     .try_into_compound_args()
//     .ok()
//     .unwrap()
//     .into_iter()
//     .collect::<Vec<(String, DataType)>>();

//   let argument_msg = osc_msg_vec
//     .iter()
//     .map(|x| parse_message(x.clone()))
//     .collect();

//   // println!("osc_msg_vec = {:?}", osc_msg_vec);

//   OscType::Array(argument_msg)
// }

fn parse_scalar(message: String) -> OscType {
  // handle numeric literals type conversion.
  // eg.`1.234_f64` is equivalent to `1.234 as f64`
  let number_types = Regex::new(r"(_i32)$|(_i64)$|(_f32)$|(_f64)$").unwrap();
  if number_types.is_match(&message) {
    let m = &message;
    let num: Vec<_> = Regex::new(r"[_]").unwrap().split(m).collect();
    return match num[1] {
      "i32" => OscType::Int(num[0].parse::<i32>().unwrap()),
      "i64" => OscType::Long(num[0].parse::<i64>().unwrap()),
      "f32" => OscType::Float(num[0].parse::<f32>().unwrap()),
      "f64" => OscType::Double(num[0].parse::<f64>().unwrap()),
      _ => OscType::Nil,
    };
  }

  let parsed = message.parse::<Val>().unwrap();
  match parsed {
    Val::I32(val) => OscType::Int(val),
    Val::I64(val) => OscType::Long(val),
    Val::F32(val) => OscType::Float(val),
    Val::F64(val) => OscType::Double(val),
    Val::Char(val) => OscType::Char(val),
    Val::Boolean(val) => OscType::Bool(val),
    Val::String(val) => OscType::String(val),
    Val::Nil => OscType::Nil,
    Val::Inf => OscType::Inf,
  }
}

// test
// /s_new "default after whitespace" 1002 'A' 'TbcS' freq 12.4533 -12 1.234_f64 [12 20 15]

// /s_new "default after whitespace" 1002 TbcS freq 12.4533 -12 -13.453
