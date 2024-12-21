use core::fmt;
use nom::{InputIter, InputLength, InputTake, Needed};
use std::iter::Enumerate;

#[derive(PartialEq, Debug, Clone, Default)]
pub enum Token {
  Illegal(String),
  EOF,

  Comma,
  LBracket,
  RBracket,
  Ident(String),

  OSCPath(String),
  StringLiteral(String),
  IntLiteral(i32),
  Long(i64),
  FloatLiteral(f32),
  Double(f64),
  BoolLiteral(bool),
  Char(char),
  TimeMsg(TimeMsg),
  MidiMessage(MidiMsg),
  Color(Color),
  Blob(Vec<u8>),
  #[default]
  Nil,
  Inf,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Color {
  pub red: u8,
  pub green: u8,
  pub blue: u8,
  pub alpha: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MidiMsg {
  pub port: u8,
  pub status: u8,
  pub data1: u8,
  pub data2: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TimeMsg {
  pub seconds: u32,
  pub fractional: u32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Tokens<'a> {
  pub tok: &'a [Token],
  pub start: usize,
  pub end: usize,
}

impl<'a> Tokens<'a> {
  pub fn new(vec: &'a [Token]) -> Self {
    Tokens {
      tok: vec,
      start: 0,
      end: vec.len(),
    }
  }
}

impl<'a> InputLength for Tokens<'a> {
  #[inline]
  fn input_len(&self) -> usize {
    self.tok.len()
  }
}

impl<'a> InputTake for Tokens<'a> {
  #[inline]
  fn take(&self, count: usize) -> Self {
    Tokens {
      tok: &self.tok[0..count],
      start: 0,
      end: count,
    }
  }

  #[inline]
  fn take_split(&self, count: usize) -> (Self, Self) {
    let (prefix, suffix) = self.tok.split_at(count);
    let first = Tokens {
      tok: prefix,
      start: 0,
      end: prefix.len(),
    };
    let second = Tokens {
      tok: suffix,
      start: 0,
      end: suffix.len(),
    };
    (second, first)
  }
}

impl InputLength for Token {
  #[inline]
  fn input_len(&self) -> usize {
    1
  }
}

impl<'a> InputIter for Tokens<'a> {
  type Item = &'a Token;
  type Iter = Enumerate<::std::slice::Iter<'a, Token>>;
  type IterElem = ::std::slice::Iter<'a, Token>;

  #[inline]
  fn iter_indices(&self) -> Enumerate<::std::slice::Iter<'a, Token>> {
    self.tok.iter().enumerate()
  }
  #[inline]
  fn iter_elements(&self) -> ::std::slice::Iter<'a, Token> {
    self.tok.iter()
  }
  #[inline]
  fn position<P>(&self, predicate: P) -> Option<usize>
  where
    P: Fn(Self::Item) -> bool,
  {
    self.tok.iter().position(predicate)
  }
  #[inline]
  fn slice_index(&self, count: usize) -> Result<usize, Needed> {
    if self.tok.len() >= count {
      Ok(count)
    } else {
      Err(Needed::Unknown)
    }
  }
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let name = match *self {
      Token::Illegal(_) => "Illegal",
      Token::EOF => "EOF",

      Token::Comma => "Comma",
      Token::LBracket => "LBracket",
      Token::RBracket => "RBracket",
      Token::Ident(_) => "Ident",

      Token::OSCPath(_) => "OSCPath",
      Token::StringLiteral(_) => "StringLiteral",
      Token::IntLiteral(_) => "IntLiteral",
      Token::Long(_) => "Long",
      Token::FloatLiteral(_) => "FloatLiteral",
      Token::Double(_) => "Double",
      Token::BoolLiteral(_) => "BoolLiteral",
      Token::Char(_) => "Char",
      Token::TimeMsg(_) => "TimeMsg",
      Token::MidiMessage(_) => "MidiMessage",
      Token::Color(_) => "Color",
      Token::Blob(_) => "Blob",
      Token::Nil => "Nil",
      Token::Inf => "Inf",
    };
    write!(f, "{}", name)
  }
}

// impl<'a> Slice<Range<usize>> for Tokens<'a> {
//   #[inline]
//   fn slice(&self, range: Range<usize>) -> Self {
//     Tokens {
//       tok: self.tok.slice(range.clone()),
//       start: self.start + range.start,
//       end: self.start + range.end,
//     }
//   }
// }

// impl<'a> Slice<RangeTo<usize>> for Tokens<'a> {
//   #[inline]
//   fn slice(&self, range: RangeTo<usize>) -> Self {
//     self.slice(0..range.end)
//   }
// }

// impl<'a> Slice<RangeFrom<usize>> for Tokens<'a> {
//   #[inline]
//   fn slice(&self, range: RangeFrom<usize>) -> Self {
//     self.slice(range.start..self.end - self.start)
//   }
// }

// impl<'a> Slice<RangeFull> for Tokens<'a> {
//   #[inline]
//   fn slice(&self, _: RangeFull) -> Self {
//     Tokens {
//       tok: self.tok,
//       start: self.start,
//       end: self.end,
//     }
//   }
// }
