use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::ops::Range;
use std::{slice, u8};
use std::{str, vec};

use bytes::complete::{is_a, take_while};
use combinator::{fail, map_res};
use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_till1, take_while_m_n};
use nom::character::complete::{alphanumeric1, anychar, char as char1, digit1, multispace0};
use nom::combinator::{cond, map, opt, recognize};
use nom::multi::{many0, separated_list0};
use nom::number::complete::double;
use nom::sequence::{delimited, pair, terminated, tuple};
use nom::*;
use sequence::preceded;

use super::token::{Color, MidiMsg, TimeMsg, Token};

// ------- custom error handling for fault-torelant parser ----------
// https://eyalkalderon.com/blog/nom-error-recovery/

pub type LocatedSpan<'a> = nom_locate::LocatedSpan<&'a str, State<'a>>;
pub type IResult<'a, T> = nom::IResult<LocatedSpan<'a>, T>;

trait ToRange {
  fn to_range(&self) -> Range<usize>;
}

trait Getter {
  fn get_unoffsetted_string(&self) -> String;
}

impl<'a> ToRange for LocatedSpan<'a> {
  fn to_range(&self) -> Range<usize> {
    let start = self.location_offset();
    let end = start + self.fragment().len();
    start..end
  }
}

impl<'a> Getter for LocatedSpan<'a> {
  fn get_unoffsetted_string(&self) -> String {
    let self_bytes = self.fragment().as_bytes();
    let self_ptr = self_bytes.as_ptr();
    unsafe {
      assert!(
        self.location_offset() <= isize::MAX as usize,
        "offset is too big"
      );
      let orig_input_ptr = self_ptr.offset(-(self.location_offset() as isize));
      let bytes = slice::from_raw_parts(orig_input_ptr, self.location_offset());
      match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(e) => format!("Failed to convert: {}", e),
      }
    }
  }
}

impl<'a, T: Default> Getter for IResult<'a, T> {
  fn get_unoffsetted_string(&self) -> String {
    let binding = RefCell::new(Vec::new());
    let err = State(&binding);
    self
      .as_ref()
      .unwrap_or(&(LocatedSpan::new_extra("", err), T::default()))
      .0
      .get_unoffsetted_string()
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Error(Range<usize>, String, String, String);

impl Error {
  pub fn print_error(&self) -> (String, String, String, String) {
    (
      format!("({}..{})", self.0.clone().start, self.0.clone().end),
      self.1.clone(),
      self.2.clone(),
      self.3.clone(),
    )
  }
}

#[derive(Clone, Debug)]
pub struct State<'a>(pub &'a RefCell<Vec<Error>>);

impl<'a> State<'a> {
  pub fn report_error(&self, error: Error) {
    self.0.borrow_mut().push(error);
  }

  pub fn is_empty(&self) -> bool {
    self.0.borrow().is_empty()
  }
}

pub fn expect<'a, F, E, T: Display>(
  mut parser: F,
  error_msg: E,
) -> impl FnMut(LocatedSpan<'a>) -> IResult<Option<T>>
where
  F: FnMut(LocatedSpan<'a>) -> IResult<T>,
  E: ToString,
{
  move |input| match parser(input) {
    Ok((remaining, out)) => Ok((remaining, Some(out))),
    Err(nom::Err::Error(input)) | Err(nom::Err::Failure(input)) => {
      let err = Error(
        input.input.to_range(),
        input.input.fragment().to_string(),
        error_msg.to_string(),
        format!(
          "{}",
          Token::Illegal(Box::new(Token::StringLiteral(
            input.input.fragment().to_string() // TODO:
          )))
        ),
      );
      input.input.extra.report_error(err);
      Ok((input.input, None))
    }
    Err(err) => Err(err),
  }
}

// -------------------------------------------

macro_rules! syntax {
  ($func_name: ident, $tag_string: literal, $output_token: expr) => {
    fn $func_name(s: LocatedSpan) -> IResult<Token> {
      map(tag($tag_string), |_| $output_token)(s)
    }
  };
}

// --------- punctuations ---------

syntax! {comma_punctuation, ",", Token::Comma}
syntax! {lbracket_punctuation, "[", Token::LBracket}
syntax! {rbracket_punctuation, "]", Token::RBracket}

pub fn lex_punctuations(input: LocatedSpan) -> IResult<Token> {
  alt((
    comma_punctuation,
    lbracket_punctuation,
    rbracket_punctuation,
  ))(input)
}

// --------- String ---------
fn pis(input: LocatedSpan) -> IResult<Vec<u8>> {
  let inp = input.clone();

  let (i1, c1) = take(1usize)(input)?;
  match c1.as_bytes() {
    b"\"" => Ok((inp, vec![])),
    c => pis(i1).map(|(slice, done)| (slice, concat_slice_vec(c, done))),
  }
}

fn concat_slice_vec(c: &[u8], done: Vec<u8>) -> Vec<u8> {
  let mut new_vec = c.to_vec();
  new_vec.extend(&done);
  new_vec
}

fn convert_vec_utf8(v: Vec<u8>) -> String {
  let slice = v.as_slice();
  let string_slice = str::from_utf8(slice).unwrap();
  string_slice.to_owned()
}

fn string(input: LocatedSpan) -> IResult<String> {
  let inp = input.clone();
  delimited(tag("\""), map(pis, convert_vec_utf8), tag("\""))(inp)
}

fn lex_string(input: LocatedSpan) -> IResult<Token> {
  if input.fragment().is_empty() {
    return map_res(is_a(""), |s| Err(nom::Err::Error(s)))(input);
  }
  map(string, Token::StringLiteral)(input)
}

// ------------- Char -------------

fn lex_char(input: LocatedSpan) -> IResult<Token> {
  map(
    delimited(
      tag("\'"),
      map(cond(true, anychar), |c| {
        if let Some(char) = c {
          if char.is_alphabetic() {
            char
          } else {
            let err = Error(
              input.to_range(),
              input.fragment().to_string(),
              r#"invalid char, or ending single quote is missing/mismatch."#.to_string(),
              format!("{}", Token::Char('\0')),
            );
            input.extra.report_error(err);
            '\0'
          }
        } else {
          '\0'
        }
      }),
      tag("\'"),
    ),
    Token::Char,
  )(input.clone())
}

// --------- Blob<Vec<u8>> ---------

fn lex_blob(input: LocatedSpan) -> IResult<Token> {
  let err_msg = "Blob value should be <u8>";
  // TODO: handle displaying error msg when result (from `expect`) is None eg. input `%[10,1.2,4]`
  map(
    delimited(
      tag("%["),
      separated_list0(tag(","), expect(digit1, err_msg)),
      tag("]"),
    ),
    |spans| {
      let vec_blob_val = spans
        .into_iter()
        .filter_map(
          |opt_span| match opt_span.clone()?.fragment().parse::<u8>() {
            Ok(val) => Some(val),
            Err(e) => {
          let err = Error(
                opt_span.clone()?.to_range(),
                opt_span.clone()?.fragment().to_string(),
                format!(r#"{:}, {:}"#, e, err_msg.to_string()),
                format!("{}", Token::Blob(Vec::default())),
              );
              input.extra.report_error(err);
              None
            }
          },
        )
        .collect::<Vec<u8>>();
      if vec_blob_val.is_empty() {
        let err = Error(
          input.to_range(),
          input.fragment().to_string(),
          err_msg.to_string(),
          format!("{}", Token::Blob(Vec::default())),
        );
        input.extra.report_error(err);
        return Token::Illegal(Box::new(Token::Blob(vec![])));
      }
      Token::Blob(vec_blob_val)
    },
  )(input.clone())
}

// --------- Ident (Bool, Nil, Inf) ---------
fn lex_reserved_ident(input: LocatedSpan) -> IResult<Token> {
  map(recognize(alt((tag("true"), tag("false"), tag("Nil"), tag("Inf")))), |span: LocatedSpan|  {
        match *span {
        "true" => Token::BoolLiteral(true),
        "false" => Token::BoolLiteral(false),
        "Nil" => Token::Nil,
        "Inf" => Token::Inf,
        rest => {
          Token::Illegal(Box::new(Token::Ident(rest.to_string())))
        }
      }
    })(input.clone())
}

// --------- osc_path ---------

fn osc_method_segment(input: LocatedSpan) -> IResult<String> {
  map(
    alt((take_while(|b: char| {
      (b.is_alphanumeric() || b.is_ascii_punctuation()) && b != '/'
    }),)),
    |s: LocatedSpan| s.fragment().to_string(),
  )(input)
}

fn osc_method(input: LocatedSpan) -> IResult<Vec<String>> {
  separated_list0(tag("/"), osc_method_segment)(input)
}

fn lex_osc_path(input: LocatedSpan) -> IResult<Token> {
  map(
    recognize(preceded(tag("/"), osc_method)),
    |s: LocatedSpan| match s.chars().next().unwrap() {
      '/' => Token::OSCPath(s.fragment().to_string()),
      _ => Token::Nil,
    },
  )(input)
}

// --------- Int(i32) ---------

fn lex_integer(input: LocatedSpan) -> IResult<Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), int_parser),
    |(sign, value)| {
      let s = sign
        .and_then(|s| {
          if s.starts_with('-') {
            Some(-1i32)
          } else {
            None
          }
        })
        .unwrap_or(1i32)
        * value;
      Token::IntLiteral(s)
    },
  )(input)
}

fn int_parser(input: LocatedSpan) -> IResult<i32> {
  map(terminated(digit1, opt(tag("_i32"))), |s: LocatedSpan| {
    let s = s.fragment().to_string();
    s.parse::<i32>().unwrap()
  })(input)
}

// --------- Long(i64) ---------

fn lex_long_integer(input: LocatedSpan) -> IResult<Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), long_int_parser),
    |(sign, value)| {
      let s = sign
        .and_then(|s| {
          if s.starts_with('-') {
            Some(-1i64)
          } else {
            None
          }
        })
        .unwrap_or(1i64)
        * value;
      Token::Long(s)
    },
  )(input)
}

fn long_int_parser(input: LocatedSpan) -> IResult<i64> {
  map(terminated(digit1, tag("_i64")), |s: LocatedSpan| {
    let s = s.fragment().to_string();
    s.parse::<i64>().unwrap()
  })(input)
}

// fn long_int_parser(input: LocatedSpan) -> IResult<i64> {
//   let err_msg = "invalid long integer, currently only support casting to i32, i64 eg. 123_i64";

//   map(expect(terminated(digit1, tag("_i64")), err_msg), |s| {
//     if s.is_some() {
//       let s = s.unwrap().fragment().to_string();
//       s.parse::<i64>().unwrap_or(0)
//     } else {
//       0
//     }
//   })(input)
// }

// --------- Float(i32) ---------

fn lex_float(input: LocatedSpan) -> IResult<Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), float_parser),
    |(sign, value)| {
      let s = sign
        .and_then(|s| {
          if s.starts_with('-') {
            Some(-1f32)
          } else {
            None
          }
        })
        .unwrap_or(1f32)
        * value;
      Token::FloatLiteral(s)
    },
  )(input)
}

fn float_parser(input: LocatedSpan) -> IResult<f32> {
  let float_bytes = recognize(alt((
    delimited(digit1, tag("."), opt(digit1)),
    delimited(opt(digit1), tag("."), digit1),
  )));
  map(
    terminated(float_bytes, opt(tag("_f32"))),
    |s: LocatedSpan| {
      let s = s.fragment().to_string();
      s.parse::<f32>().unwrap()
    },
  )(input)
}

// --------- Float(f64) ---------

fn lex_double_float(input: LocatedSpan) -> IResult<Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), double_parser),
    |(sign, value)| {
      let s = sign
        .and_then(|s| {
          if s.starts_with('-') {
            Some(-1f64)
          } else {
            None
          }
        })
        .unwrap_or(1f64)
        * value;
      Token::Double(s)
    },
  )(input)
}

fn double_parser(input: LocatedSpan) -> IResult<f64> {
  terminated(double, tag("_f64"))(input)
}

// --------- Color ---------

fn from_hex(input: LocatedSpan) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(*input, 16)
}

fn is_hex_digit(c: char) -> bool {
  c.is_ascii_hexdigit()
}

fn hex_primary(input: LocatedSpan) -> IResult<u8> {
  map(take_while_m_n(2, 2, is_hex_digit), |s| match from_hex(s) {
    Ok(v) => v,
    Err(e) => {
      println!("parsing from_hex error {}", e);
      0u8
    }
  })(input)
}

pub fn lex_color(input: LocatedSpan) -> IResult<Token> {
  let (inp, _) = tag("#")(input)?;
  let (remain, (red, green, blue, alpha)) =
    tuple((hex_primary, hex_primary, hex_primary, hex_primary))(inp)?;
  let col = Color {
    red,
    green,
    blue,
    alpha,
  };

  Ok((remain, Token::Color(col)))
}

// --------- MidiMsg ---------

pub fn lex_midimsg(input: LocatedSpan) -> IResult<Token> {
  let (inp, _) = tag("~")(input)?;
  let (remain, (port, status, data1, data2)) =
    tuple((hex_primary, hex_primary, hex_primary, hex_primary))(inp)?;
  let msg = MidiMsg {
    port,
    status,
    data1,
    data2,
  };
  Ok((remain, Token::MidiMessage(msg)))
}

// --------- TimeMsg ---------

fn parse_digits_with_underscores(input: LocatedSpan) -> IResult<String> {
  let parser = many0(alt((
    map(digit1, |s: LocatedSpan| s.to_string()),
    map(char1('_'), |_| "_".to_string()),
  )));

  map(parser, |parts| parts.concat().replace('_', ""))(input)
}

fn parse_seconds(input: LocatedSpan) -> IResult<u32> {
  let (input, seconds_str) = parse_digits_with_underscores(input)?;
  let seconds = seconds_str.parse::<u32>().unwrap_or(0);
  Ok((input, seconds))
}

fn parse_fractional(input: LocatedSpan) -> IResult<u32> {
  let (input, fractional_str) =
    opt(terminated(parse_digits_with_underscores, many0(char1(' '))))(input)?;

  let fractional = fractional_str.unwrap_or_else(|| "".to_string());
  let fractional = if !fractional.is_empty() {
    fractional.parse::<u32>().unwrap_or(0)
  } else {
    0
  };

  Ok((input, fractional))
}

fn parse_time_segment(input: LocatedSpan) -> IResult<(u32, u32)> {
  let mut parser = alt((map(
    tuple((parse_seconds, char1('.'), parse_fractional)),
    |(seconds, _, fractional): (u32, char, u32)| (seconds, fractional),
  ),));

  parser(input)
}

pub fn lex_timemsg(input: LocatedSpan) -> IResult<Token> {
  let (inp, _) = tag("@")(input.clone())?;
  let (remaining, (seconds, fractional)) = parse_time_segment(inp)?;
  let msg = TimeMsg {
    seconds,
    fractional,
  };
  Ok((remaining, Token::TimeMsg(msg)))
}

// --------- Error ---------

fn get_err_msg(input: LocatedSpan) -> (Token, String) {
  let input_string = input.fragment().to_string();
  let default_err_msg = format!("Unexpected: {}", input_string);

  let err_msg: (Token, String) = match input.fragment().chars().next() {
    Some(val) => match val {
      '\"' => (
        Token::Illegal(Box::new(Token::StringLiteral(input_string))),
        "Invalid string: the ending double quote is possibly missing.".to_string(),
      ),
      '%' => (
        Token::Illegal(Box::new(Token::Blob(vec![]))),
        "Invalid blob: blob format is %[<u8>,<u8>,..] eg. %[10,255,200]".to_string(),
      ),
      '@' => (
        Token::Illegal(Box::new(Token::TimeMsg(TimeMsg::default()))),
        "Invalid time msg: time format is `@<second>.<fractional>` eg. @2_208_988_800.2_123"
          .to_string(),
      ),
      '\'' => (
        Token::Illegal(Box::new(Token::Char('\0'))),
        "Invalid char: char format is '<char>' eg. 'a'".to_string(),
      ),
      '#' => (
        Token::Illegal(Box::new(Token::Color(Color::default()))),
        "Invalid color code: color format is `#<red><green><blue><alpha>` eg. #2F14DF2A"
          .to_string(),
      ),
      '~' => (
        Token::Illegal(Box::new(Token::MidiMessage(MidiMsg::default()))),
        "Invalid midi msg: msg format is `~<port><status><data1><data2>` eg. ~01F14FA4".to_string(),
      ),
      '_' => (Token::Illegal(Box::new(Token::IntLiteral(0))), "Invalid integer: numeric literals type conversion support only i32,i64(Long Int) eg. 123_i64".to_string()),
      _ => (Token::Illegal(Box::new(Token::Ident(input_string))), "Invalid argument: If you intended to pass an identity keyword, only [true, false, Nil, Inf] are valid".to_string()) 
    },
    None => (Token::Illegal(Box::new(Token::Ident(input_string))), default_err_msg), 
  };

  err_msg
}
fn lex_error(input: LocatedSpan) -> IResult<Token> {
  map(take_till1(|c| c == '\n'), |span: LocatedSpan| {
    let err_msg = get_err_msg(span.clone());
    let err = Error(
      span.to_range(),
      span.fragment().to_string(),
      err_msg.1.clone(),
      err_msg.0.to_string(),
    );
    span.extra.report_error(err);
    Token::Illegal(Box::new(Token::StringLiteral(span.fragment().to_string())))
  })(input.clone())
}

fn lex_token(input: LocatedSpan) -> IResult<Token> {
  alt((
    lex_osc_path,
    lex_punctuations,
    lex_string,
    lex_blob,
    lex_timemsg,
    lex_midimsg,
    lex_color,
    lex_long_integer,
    lex_double_float,
    lex_float,
    lex_integer,
    lex_reserved_ident,
    lex_char,
    lex_error, 
  ))(input)
}

fn lex_tokens(input: LocatedSpan) -> IResult<Vec<Token>> {
  many0(delimited(multispace0, lex_token, multispace0))(input)
}

pub struct Lexer;

impl Lexer {
  pub fn lex_tokens(input: LocatedSpan) -> IResult<Vec<Token>> {
    lex_tokens(input).map(|(slice, result)| (slice, [&result[..], &vec![Token::EOF][..]].concat()))
  }

  pub fn analyse(source: &str) -> (Vec<Token>, Vec<Error>) {
    let errors = RefCell::new(Vec::new());
    let input = LocatedSpan::new_extra(source, State(&errors));
    match Self::lex_tokens(input) {
      Ok((_, expr)) => (expr, errors.into_inner()),
      Err(_) => (vec![], errors.into_inner()),
    }
  }
}

#[cfg(test)]
mod tests {
  use nom_locate::LocatedSpan;

  use super::*;

  #[test]
  fn test_valid_osc_addresses() {
    let errors = RefCell::new(Vec::new());
    let valid_osc_addr = [
      "/",
      "/cue/selected/level",
      "/cue/selected/level/0/1/+",
      "/cue/selected/level/0/1/-",
      "/cue/0/{synth,drum}/-",
      "/cue/0/{synth,drum}/.",
      "/cue/0/{synth,drum}/*",
      "/cue/0/[synth]/*",
      "/press/bank/*/1",
      "/press/bank/*/1?",
      "/press/bank/1/",
    ];

    for addr in valid_osc_addr.iter() {
      assert_eq!(
        lex_osc_path(LocatedSpan::new_extra(addr, State(&errors))).get_unoffsetted_string(),
        addr.to_string()
      );
    }
  }

  #[test]
  fn test_invalid_osc_addresses() {
    let errors = RefCell::new(Vec::new());
    let invalid_osc_addr = [
      "+", "#/", ")", " ",
      "1",
      // "//",
      // "/cue/selected/level/0/1//",
      // "/cue///level/0/1//",
    ];

    for addr in invalid_osc_addr.iter() {
      assert_ne!(
        lex_osc_path(LocatedSpan::new_extra(addr, State(&errors))).get_unoffsetted_string(),
        addr.to_string()
      );
    }
  }

  #[test]
  fn test_valid_blob() {
    let errors = RefCell::new(Vec::new());
    let valid_blob_msg = ["%[0]", "%[10,20,30]", "%[]", "%[255]"];

    for addr in valid_blob_msg.iter() {
      assert_eq!(
        lex_blob(LocatedSpan::new_extra(addr, State(&errors))).get_unoffsetted_string(),
        addr.to_string()
      );
    }
  }

  // #[test]
  // fn test_invalid_blob() {
  //   let invalid_blob_msg = [
  //     (
  //       "%[-5,-12,43]",
  //       vec![Error(
  //         2..12,
  //         "-5,-12,43]".to_string(),
  //         "Blob value should be <u8>".to_string(),
  //         format!("{}", Token::Illegal("".to_string())),
  //       )],
  //     ),
  //     (
  //       "%[257]",
  //       vec![
  //         Error(
  //           2..5,
  //           "257".to_string(),
  //           "number too large to fit in target type, Blob value should be <u8>".to_string(),
  //           format!("{}", Token::Blob(Vec::default())),
  //         ),
  //         Error(
  //           0..6,
  //           "%[257]".to_string(),
  //           "Blob value should be <u8>".to_string(),
  //           format!("{}", Token::Blob(Vec::default())),
  //         ),
  //       ],
  //     ),
  //     (
  //       "%[100,200,257]",
  //       vec![Error(
  //         10..13,
  //         "257".to_string(),
  //         "number too large to fit in target type, Blob value should be <u8>".to_string(),
  //         format!("{}", Token::Blob(Vec::default())),
  //       )],
  //     ),
  //     (
  //       "%['test']",
  //       vec![Error(
  //         2..9,
  //         "'test']".to_string(),
  //         "Blob value should be <u8>".to_string(),
  //         format!("{}", Token::Illegal("".to_string())),
  //       )],
  //     ),
  //   ];
  //   for (addr, expected_msg) in invalid_blob_msg.iter() {
  //     let errors = RefCell::new(Vec::new());
  //     let blob_res = LocatedSpan::new_extra(&**addr, State(&errors));
  //     let exp = lex_blob(blob_res);
  //     if let Ok(v) = exp {
  //       let res_exp = v.0.extra.0.borrow();
  //       assert_eq!(*res_exp, *expected_msg)
  //     } else {
  //       assert_eq!(*errors.borrow(), *expected_msg);
  //     }
  //   }
  // }
}
