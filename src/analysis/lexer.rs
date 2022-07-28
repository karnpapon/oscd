use nom::branch::*;
use nom::bytes::complete::{is_not, tag, take, take_till, take_while_m_n};
use nom::character::complete::{alpha1, alphanumeric1, char as char1, digit0, anychar, digit1, multispace0};
use nom::character::{is_alphabetic, is_alphanumeric, is_digit};
use nom::combinator::{map, map_res, cond, opt, recognize};
use nom::error::*;
use nom::multi::many0;
use nom::number::complete::{double};
use nom::sequence::{delimited, pair, separated_pair, terminated, tuple};
use nom::*;

use std::str;
use std::str::FromStr;
use std::str::Utf8Error;

use super::token::{Color, MidiMsg, TimeMsg, Token};

macro_rules! syntax {
  ($func_name: ident, $tag_string: literal, $output_token: expr) => {
    fn $func_name<'a>(s: &'a [u8]) -> IResult<&[u8], Token> {
      map(tag($tag_string), |_| $output_token)(s)
    }
  };
}

// punctuations
syntax! {comma_punctuation, ",", Token::Comma}
syntax! {lbracket_punctuation, "[", Token::LBracket}
syntax! {rbracket_punctuation, "]", Token::RBracket}

pub fn lex_punctuations(input: &[u8]) -> IResult<&[u8], Token> {
  alt((
    comma_punctuation,
    lbracket_punctuation,
    rbracket_punctuation,
  ))(input)
}

// Strings
fn pis(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
  use std::result::Result::*;

  let (i1, c1) = take(1usize)(input)?;
  match c1.as_bytes() {
    b"\"" => Ok((input, vec![])),
    c => pis(i1).map(|(slice, done)| (slice, concat_slice_vec(c, done))),
  }
}

fn concat_slice_vec(c: &[u8], done: Vec<u8>) -> Vec<u8> {
  let mut new_vec = c.to_vec();
  new_vec.extend(&done);
  new_vec
}

fn convert_vec_utf8(v: Vec<u8>) -> Result<String, Utf8Error> {
  let slice = v.as_slice();
  str::from_utf8(slice).map(|s| s.to_owned())
}

fn complete_byte_slice_str_from_utf8(c: &[u8]) -> Result<&str, Utf8Error> {
  str::from_utf8(c)
}

fn string(input: &[u8]) -> IResult<&[u8], String> {
  delimited(tag("\""), map_res(pis, convert_vec_utf8), tag("\""))(input)
}

fn lex_string(input: &[u8]) -> IResult<&[u8], Token> {
  map(string, Token::StringLiteral)(input)
}

fn lex_char(input: &[u8]) -> IResult<&[u8], Token> {
  map(delimited(
    tag("\'"),
    map(cond(true, anychar), |x| x), 
    tag("\'")), |x| Token::Char(x.unwrap()))
  (input)
  // map(cond(true, anychar), |x| Token::Char(x.unwrap()))(input)
}

// Reserved or ident (eg. Nil, Inf, OSCpath)
fn lex_reserved_ident(input: &[u8]) -> IResult<&[u8], Token> {
  map_res(
    recognize(pair(
      alt((alpha1, tag("_"), tag("/"), tag(":"))),
      many0(alt((alphanumeric1, tag("_"), tag("/"), tag(":")))),
    )),
    |s| {
      let c = complete_byte_slice_str_from_utf8(s);
      c.map(|syntax| match syntax {
        "true" => Token::BoolLiteral(true),
        "false" => Token::BoolLiteral(false),
        "Nil" => Token::Nil,
        "Inf" => Token::Inf,
        _ => match syntax.chars().next().unwrap() {
          '/' => Token::OSCPath(syntax.to_string()),
          _ => Token::Ident(syntax.to_string()),
        },
      })
    },
  )(input)
}

// Integers parsing
fn lex_integer(input: &[u8]) -> IResult<&[u8], Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), unsigned_int),
    |(sign, value)| {
      let s = sign
        .and_then(|s| if s[0] == b'-' { Some(-1i32) } else { None })
        .unwrap_or(1i32)
        * value;
      Token::IntLiteral(s)
    },
  )(input)
}

fn unsigned_int(input: &[u8]) -> IResult<&[u8], i32> {
  let float_str = map_res(digit1, str::from_utf8);
  map_res(float_str, FromStr::from_str)(input)
}

fn lex_long_integer(input: &[u8]) -> IResult<&[u8], Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), unsigned_long_int),
    |(sign, value)| {
      let s = sign
        .and_then(|s| if s[0] == b'-' { Some(-1i64) } else { None })
        .unwrap_or(1i64)
        * value;
      Token::Long(s)
    },
  )(input)
}

fn unsigned_long_int(input: &[u8]) -> IResult<&[u8], i64> {
  let float_str = map_res(terminated(digit1, tag("_i64")), str::from_utf8);
  map_res(float_str, FromStr::from_str)(input)
}

fn lex_float(input: &[u8]) -> IResult<&[u8], Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), unsigned_float),
    |(sign, value)| {
      let s = sign
        .and_then(|s| if s[0] == b'-' { Some(-1f32) } else { None })
        .unwrap_or(1f32)
        * value;
      Token::FloatLiteral(s)
    },
  )(input)
}

fn unsigned_float(input: &[u8]) -> IResult<&[u8], f32> {
  let float_bytes = recognize(alt((
    delimited(digit1, tag("."), opt(digit1)),
    delimited(opt(digit1), tag("."), digit1),
  )));
  let float_str = map_res(float_bytes, str::from_utf8);
  map_res(float_str, FromStr::from_str)(input)
}

fn lex_double_float(input: &[u8]) -> IResult<&[u8], Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), unsigned_double_float),
    |(sign, value)| {
      let s = sign
        .and_then(|s| if s[0] == b'-' { Some(-1f64) } else { None })
        .unwrap_or(1f64)
        * value;
      Token::Double(s)
    },
  )(input)
}

fn unsigned_double_float(input: &[u8]) -> IResult<&[u8], f64> {
  terminated(double, tag("_f64"))(input)
}

fn lex_illegal(input: &[u8]) -> IResult<&[u8], Token> {
  map(take(1usize), |_| Token::Illegal)(input)
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
  c.is_ascii_hexdigit()
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
  map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

pub fn lex_color(input: &[u8]) -> IResult<&[u8], Token> {
  let (_input, _) = tag("#")(input)?;
  let (__input, (red, green, blue, alpha)) =
    tuple((hex_primary, hex_primary, hex_primary, hex_primary))(
      std::str::from_utf8(_input).unwrap(),
    )
    .expect("cannot parse color arg");
  let col = Color {
    red,
    green,
    blue,
    alpha,
  };
  Ok((__input.as_bytes(), Token::Color(col)))
}

// -------------------
// /s_new ~2F14FA4C

pub fn lex_midimsg(input: &[u8]) -> IResult<&[u8], Token> {
  let (_input, _) = tag("~")(input)?;
  let (__input, (port, status, data1, data2)) =
    tuple((hex_primary, hex_primary, hex_primary, hex_primary))(
      std::str::from_utf8(_input).unwrap(),
    )
    .expect("cannot parse midimsg arg");
  let msg = MidiMsg {
    port,
    status,
    data1,
    data2,
  };
  Ok((__input.as_bytes(), Token::MidiMessage(msg)))
}

// -------------------
// /s_new @123456789:20
// TODO: handle unsupport case eg. @123456789:20:123
pub fn lex_timemsg(input: &[u8]) -> IResult<&[u8], Token> {
  let (_input, _) = tag("@")(input)?;
  let (remaining, (seconds, fractional)) = separated_pair(digit1, char1(':'), digit1)(_input)?;
  let msg = TimeMsg {
    seconds: std::str::from_utf8(seconds)
      .unwrap()
      .parse::<u32>()
      .unwrap(),
    fractional: std::str::from_utf8(fractional)
      .unwrap()
      .parse::<u32>()
      .unwrap(),
  };
  Ok((remaining, Token::TimeMsg(msg)))
}

// -------------------

fn lex_token(input: &[u8]) -> IResult<&[u8], Token> {
  alt((
    lex_punctuations,
    lex_string,
    lex_reserved_ident,
    lex_double_float,
    lex_timemsg,
    lex_midimsg,
    lex_color,
    lex_float,
    lex_long_integer,
    lex_integer,
    lex_char,
    lex_illegal,
  ))(input)
}

fn lex_tokens(input: &[u8]) -> IResult<&[u8], Vec<Token>> {
  many0(delimited(multispace0, lex_token, multispace0))(input)
}

pub struct Lexer;

impl Lexer {
  pub fn lex_tokens(bytes: &[u8]) -> IResult<&[u8], Vec<Token>> {
    lex_tokens(bytes).map(|(slice, result)| (slice, [&result[..], &vec![Token::EOF][..]].concat()))
  }
}
