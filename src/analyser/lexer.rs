use std::str::FromStr;
use std::str::Utf8Error;
use std::{str, vec};

use nom::branch::*;
use nom::bytes::complete::{tag, take, take_while_m_n};
use nom::character::complete::{
  alpha1, alphanumeric1, anychar, char as char1, digit1, multispace0,
};
use nom::combinator::{cond, iterator, map, map_res, opt, recognize};
use nom::multi::many0;
use nom::number::complete::double;
use nom::sequence::{delimited, pair, separated_pair, terminated, tuple};
use nom::*;

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
  map(
    delimited(
      tag("\'"),
      map(cond(true, anychar), |x| x.unwrap()),
      tag("\'"),
    ),
    Token::Char,
  )(input)
}

fn lex_blob(input: &[u8]) -> IResult<&[u8], Token> {
  map(delimited(tag("%["), blobs, tag("]")), Token::Blob)(input)
}

fn blobs(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
  let mut it = iterator(input, terminated(digit1, tag(",")));

  let parsed = it.map(|v| str::from_utf8(v).unwrap());
  let byte1 = parsed
    .into_iter()
    .map(|p| p.parse::<u8>().unwrap())
    .collect::<Vec<_>>();
  let res: IResult<_, _> = it.finish();

  Ok((res.unwrap().0, byte1))
}

fn lex_reserved_ident(input: &[u8]) -> IResult<&[u8], Token> {
  map_res(recognize(pair(alpha1, many0(alphanumeric1))), |s| {
    let c = complete_byte_slice_str_from_utf8(s);
    c.map(|syntax| match syntax {
      "true" => Token::BoolLiteral(true),
      "false" => Token::BoolLiteral(false),
      "Nil" => Token::Nil,
      "Inf" => Token::Inf,
      _ => Token::Nil,
    })
  })(input)
}

fn lex_osc_path(input: &[u8]) -> IResult<&[u8], Token> {
  map_res(
    recognize(pair(
      tag("/"),
      many0(alt((
        alphanumeric1,
        tag("_"),
        tag("-"),
        tag("/"),
        tag("*"),
        tag("?"),
        tag("!"),
        tag("#"),
        tag("["),
        tag("]"),
      ))),
    )),
    |s| {
      let c = complete_byte_slice_str_from_utf8(s);
      c.map(|syntax| match syntax.chars().next().unwrap() {
        '/' => Token::OSCPath(syntax.to_string()),
        _ => Token::Nil,
      })
    },
  )(input)
}

// Integers parsing
fn lex_integer(input: &[u8]) -> IResult<&[u8], Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), int_parser),
    |(sign, value)| {
      let s = sign
        .and_then(|s| if s[0] == b'-' { Some(-1i32) } else { None })
        .unwrap_or(1i32)
        * value;
      Token::IntLiteral(s)
    },
  )(input)
}

fn int_parser(input: &[u8]) -> IResult<&[u8], i32> {
  let int_str = map_res(terminated(digit1, opt(tag("_i32"))), str::from_utf8);
  map_res(int_str, FromStr::from_str)(input)
}

fn lex_long_integer(input: &[u8]) -> IResult<&[u8], Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), long_int_parser),
    |(sign, value)| {
      let s = sign
        .and_then(|s| if s[0] == b'-' { Some(-1i64) } else { None })
        .unwrap_or(1i64)
        * value;
      Token::Long(s)
    },
  )(input)
}

fn long_int_parser(input: &[u8]) -> IResult<&[u8], i64> {
  let int_str = map_res(terminated(digit1, tag("_i64")), str::from_utf8);
  map_res(int_str, FromStr::from_str)(input)
}

fn lex_float(input: &[u8]) -> IResult<&[u8], Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), float_parser),
    |(sign, value)| {
      let s = sign
        .and_then(|s| if s[0] == b'-' { Some(-1f32) } else { None })
        .unwrap_or(1f32)
        * value;
      Token::FloatLiteral(s)
    },
  )(input)
}

fn float_parser(input: &[u8]) -> IResult<&[u8], f32> {
  let float_bytes = recognize(alt((
    delimited(digit1, tag("."), opt(digit1)),
    delimited(opt(digit1), tag("."), digit1),
  )));
  let float_str = map_res(terminated(float_bytes, opt(tag("_f32"))), str::from_utf8);
  map_res(float_str, FromStr::from_str)(input)
}

fn lex_double_float(input: &[u8]) -> IResult<&[u8], Token> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), double_parser),
    |(sign, value)| {
      let s = sign
        .and_then(|s| if s[0] == b'-' { Some(-1f64) } else { None })
        .unwrap_or(1f64)
        * value;
      Token::Double(s)
    },
  )(input)
}

fn double_parser(input: &[u8]) -> IResult<&[u8], f64> {
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
  let (inp, _) = tag("#")(input)?;
  let (remain, (red, green, blue, alpha)) =
    tuple((hex_primary, hex_primary, hex_primary, hex_primary))(std::str::from_utf8(inp).unwrap())
      .unwrap();
  let col = Color {
    red,
    green,
    blue,
    alpha,
  };
  Ok((remain.as_bytes(), Token::Color(col)))
}

pub fn lex_midimsg(input: &[u8]) -> IResult<&[u8], Token> {
  let (_input, _) = tag("~")(input)?;
  let (remaining, (port, status, data1, data2)) =
    tuple((hex_primary, hex_primary, hex_primary, hex_primary))(
      std::str::from_utf8(_input).unwrap(),
    )
    .unwrap();
  let msg = MidiMsg {
    port,
    status,
    data1,
    data2,
  };
  Ok((remaining.as_bytes(), Token::MidiMessage(msg)))
}

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

fn lex_token(input: &[u8]) -> IResult<&[u8], Token> {
  alt((
    lex_osc_path,
    lex_punctuations,
    lex_blob,
    lex_string,
    lex_timemsg,
    lex_midimsg,
    lex_color,
    lex_long_integer,
    lex_double_float,
    lex_float,
    lex_integer,
    lex_reserved_ident,
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
