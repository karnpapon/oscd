use nom::branch::*;
use nom::bytes::complete::take;
use nom::combinator::{map, opt, verify};
use nom::error::{Error, ErrorKind};
use nom::multi::many0;
use nom::sequence::*;
use nom::Err;
use nom::*;
use rosc::{OscArray, OscType};

use super::lexer::token::{Token, Tokens};
use std::result::Result::*;

#[derive(PartialEq, Debug, Clone)]
pub enum Stmt {
  ExprStmt(Expr),
  ReturnStmt(Expr),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
  IdentExpr(Ident),
  LitExpr(Literal),
  ArrayExpr(Vec<Expr>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
  IntLiteral(i32),
  FloatLiteral(f32),
  BoolLiteral(bool),
  StringLiteral(String),
  OscPathLiteral(String),
}

#[derive(PartialEq, Debug, Eq, Clone)]
pub struct Ident(pub String);

pub type Program = Vec<Stmt>;

pub struct Parser;

impl Parser {
  pub fn parse_tokens(tokens: Tokens) -> IResult<Tokens, Program> {
    parse_program(tokens)
  }
}

fn parse_program(input: Tokens) -> IResult<Tokens, Program> {
  terminated(many0(parse_stmt), eof_tag)(input)
}

fn parse_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
  alt((parse_return_stmt, parse_expr_stmt))(input)
}

fn parse_return_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
  map(
    delimited(return_tag, parse_expr, opt(semicolon_tag)),
    Stmt::ReturnStmt,
  )(input)
}

fn parse_expr_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
  map(terminated(parse_expr, opt(semicolon_tag)), |expr| {
    Stmt::ExprStmt(expr)
  })(input)
}

macro_rules! tag_token (
  ($func_name:ident, $tag: expr) => (
      fn $func_name(tokens: Tokens) -> IResult<Tokens, Tokens> {
          verify(take(1usize), |t: &Tokens| t.tok[0] == $tag)(tokens)
      }
  )
);

fn parse_literal(input: Tokens) -> IResult<Tokens, Literal> {
  let (i1, t1) = take(1usize)(input)?;
  if t1.tok.is_empty() {
    Err(Err::Error(Error::new(input, ErrorKind::Tag)))
  } else {
    match t1.tok[0].clone() {
      Token::IntLiteral(name) => Ok((i1, Literal::IntLiteral(name))),
      Token::StringLiteral(s) => Ok((i1, Literal::StringLiteral(s))),
      Token::FloatLiteral(s) => Ok((i1, Literal::FloatLiteral(s))),
      Token::BoolLiteral(b) => Ok((i1, Literal::BoolLiteral(b))),
      Token::OSCPath(b) => Ok((i1, Literal::OscPathLiteral(b))),
      _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
    }
  }
}
fn parse_ident(input: Tokens) -> IResult<Tokens, Ident> {
  let (i1, t1) = take(1usize)(input)?;
  if t1.tok.is_empty() {
    Err(Err::Error(Error::new(input, ErrorKind::Tag)))
  } else {
    match t1.tok[0].clone() {
      Token::Ident(name) => Ok((i1, Ident(name))),
      _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
    }
  }
}

tag_token!(return_tag, Token::Return);
tag_token!(semicolon_tag, Token::SemiColon);
tag_token!(lbracket_tag, Token::LBracket);
tag_token!(rbracket_tag, Token::RBracket);
tag_token!(comma_tag, Token::Comma);
tag_token!(eof_tag, Token::EOF);

fn parse_lit_expr(input: Tokens) -> IResult<Tokens, Expr> {
  map(parse_literal, Expr::LitExpr)(input)
}

fn parse_ident_expr(input: Tokens) -> IResult<Tokens, Expr> {
  map(parse_ident, Expr::IdentExpr)(input)
}

pub fn parse_atom_expr(input: Tokens) -> IResult<Tokens, Expr> {
  alt((parse_lit_expr, parse_ident_expr, parse_array_expr))(input)
}

pub fn parse_expr(input: Tokens) -> IResult<Tokens, Expr> {
  parse_pratt_expr(input)
}

fn parse_pratt_expr(input: Tokens) -> IResult<Tokens, Expr> {
  let (i1, left) = parse_atom_expr(input)?;
  go_parse_pratt_expr(i1, left)
}

fn go_parse_pratt_expr(input: Tokens, left: Expr) -> IResult<Tokens, Expr> {
  let (i1, t1) = take(1usize)(input)?;
  if t1.tok.is_empty() {
    Ok((i1, left))
  } else {
    Ok((input, left))
  }
}

fn parse_exprs(input: Tokens) -> IResult<Tokens, Vec<Expr>> {
  map(
    pair(parse_expr, many0(parse_comma_exprs)),
    |(first, second)| [&vec![first][..], &second[..]].concat(),
  )(input)
}

fn parse_comma_exprs(input: Tokens) -> IResult<Tokens, Expr> {
  preceded(comma_tag, parse_expr)(input)
}

fn empty_boxed_vec(input: Tokens) -> IResult<Tokens, Vec<Expr>> {
  Ok((input, vec![]))
}

pub fn parse_array_expr(input: Tokens) -> IResult<Tokens, Expr> {
  map(
    delimited(
      lbracket_tag,
      alt((parse_exprs, empty_boxed_vec)),
      rbracket_tag,
    ),
    Expr::ArrayExpr,
  )(input)
}

pub fn parse_message(message: &Expr) -> OscType {
  match message {
    Expr::IdentExpr(v) => parse_identity(v),
    Expr::LitExpr(v) => parse_scalar(v),
    Expr::ArrayExpr(v) => parse_compound(v),
    _ => OscType::Nil,
  }
}

fn parse_identity(message: &Ident) -> OscType {
  match message {
    Ident(val) => OscType::String(val.clone()),
    _ => OscType::Nil,
  }
}

fn parse_scalar(message: &Literal) -> OscType {
  match message {
    Literal::IntLiteral(val) => OscType::Int(val.clone()),
    Literal::FloatLiteral(val) => OscType::Float(val.clone()),
    Literal::BoolLiteral(val) => OscType::Bool(val.clone()),
    Literal::StringLiteral(val) => OscType::String(val.clone()),
    Literal::OscPathLiteral(val) => OscType::String(val.clone()),
  }
}

fn parse_compound(message: &Vec<Expr>) -> OscType {
  let arr = message
    .into_iter()
    .map(|x| parse_message(x))
    .collect::<Vec<OscType>>();
  let aa = OscArray::from_iter(arr);
  OscType::Array(aa)
}

// test
// /s_new "default after whitespace" 1002 'A' 'TbcS' freq 12.4533 -12 1.234_f64 [12 20 15]
// /s_new "default after whitespace" 1002 TbcS freq 12.4533 -12 -13.453 [12,20,15]
