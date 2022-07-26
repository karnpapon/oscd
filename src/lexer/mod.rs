use nom::branch::*;
use nom::bytes::complete::{is_not, tag, take, take_while1};
use nom::character::complete::{
  alpha1, alphanumeric1, anychar, char as char1, digit1, multispace0,
};
use nom::character::{is_alphabetic, is_alphanumeric};
use nom::combinator::{map, map_parser, map_res, opt, recognize};
use nom::error::ErrorKind;
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, pair, tuple};
use nom::*;

use std::str;
use std::str::FromStr;
use std::str::Utf8Error;

pub mod token;
use token::Token;

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
    b"\\" => {
      let (i2, c2) = take(1usize)(i1)?;
      pis(i2).map(|(slice, done)| (slice, concat_slice_vec(c2, done)))
    }
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

fn convert_char(v: Vec<u8>) -> Result<char, Utf8Error> {
  let slice = v.as_slice();
  Ok(str::from_utf8(slice).unwrap().chars().next().unwrap())
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

// fn lex_arr(input: &[u8]) -> IResult<&[u8], Vec<String>> {
//   separated_list0(
//     char1(','),
//     map_res( is_not(", \t"))
//   )(input)
// }

fn lex_char(input: &[u8]) -> IResult<&[u8], Token> {
  // let method = take_while1(is_alphabetic);
  // let space = take_while1(|c| c == b' ');
  // let line_ending = tag("\r\n");
  // let tu = tuple(( method, space, line_ending ))(input)?;
  // let float_str = map_res(char1, str::from_utf8);
  // println!("inp method = {:?}", float_str);

  map(|inp| char1('A')(inp), Token::Char)(input)
}

// Reserved or ident
fn lex_reserved_ident(input: &[u8]) -> IResult<&[u8], Token> {
  map_res(
    recognize(pair(
      alt((alpha1, tag("_"), tag("/"), tag(":"))),
      many0(alt((alphanumeric1, tag("_"), tag("/")))),
    )),
    |s| {
      let c = complete_byte_slice_str_from_utf8(s);
      c.map(|syntax| match syntax {
        "true" => Token::BoolLiteral(true),
        "false" => Token::BoolLiteral(false),
        _ => match syntax.chars().nth(0).unwrap() {
          '/' => Token::OSCPath(syntax.to_string()),
          _ => Token::Ident(syntax.to_string()),
        },
      })
    },
  )(input)
}

fn complete_str_from_str<F: FromStr>(c: &str) -> Result<F, F::Err> {
  FromStr::from_str(c)
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

fn unsigned_float(input: &[u8]) -> IResult<&[u8], f32> {
  let float_bytes = recognize(alt((
    delimited(digit1, tag("."), opt(digit1)),
    delimited(opt(digit1), tag("."), digit1),
  )));
  let float_str = map_res(float_bytes, str::from_utf8);
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

fn lex_illegal(input: &[u8]) -> IResult<&[u8], Token> {
  map(take(1usize), |_| Token::Illegal)(input)
}

fn lex_token(input: &[u8]) -> IResult<&[u8], Token> {
  alt((
    lex_punctuations,
    lex_char,
    lex_string,
    lex_reserved_ident,
    lex_float,
    lex_integer,
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
