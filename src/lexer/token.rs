use nom::{
  bytes::complete::{tag, take_while_m_n},
  combinator::map_res,
  sequence::tuple,
  IResult, InputIter, InputLength, InputTake, Needed, Slice,
};
use std::iter::Enumerate;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
  Illegal,
  EOF,

  SemiColon,
  OSCPath(String),

  Ident(String),
  StringLiteral(String),
  IntLiteral(i32),
  Long(i64),
  FloatLiteral(f32),
  Double(f64),
  BoolLiteral(bool),
  Char(char),

  Comma,

  Return,

  LBracket,
  RBracket,

  // Color,
  Nil,
  Inf,
}

// #[derive(Debug, PartialEq)]
// pub struct Color {
//   pub red: u8,
//   pub green: u8,
//   pub blue: u8,
// }

// fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
//   u8::from_str_radix(input, 16)
// }

// fn is_hex_digit(c: char) -> bool {
//   c.is_digit(16)
// }

// fn hex_primary(input: &str) -> IResult<&str, u8> {
//   map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
// }

// pub fn hex_color(input: &str) -> IResult<&str, Color> {
//   let (input, _) = tag("#")(input)?;
//   let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

//   Ok((input, Color { red, green, blue }))
// }

// #[test]
// fn parse_color() {
//   assert_eq!(hex_color("#2F14DF"), Ok(("", Color {
//     red: 47,
//     green: 20,
//     blue: 223,
//   })));
// }

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

impl<'a> Slice<Range<usize>> for Tokens<'a> {
  #[inline]
  fn slice(&self, range: Range<usize>) -> Self {
    Tokens {
      tok: self.tok.slice(range.clone()),
      start: self.start + range.start,
      end: self.start + range.end,
    }
  }
}

impl<'a> Slice<RangeTo<usize>> for Tokens<'a> {
  #[inline]
  fn slice(&self, range: RangeTo<usize>) -> Self {
    self.slice(0..range.end)
  }
}

impl<'a> Slice<RangeFrom<usize>> for Tokens<'a> {
  #[inline]
  fn slice(&self, range: RangeFrom<usize>) -> Self {
    self.slice(range.start..self.end - self.start)
  }
}

impl<'a> Slice<RangeFull> for Tokens<'a> {
  #[inline]
  fn slice(&self, _: RangeFull) -> Self {
    Tokens {
      tok: self.tok,
      start: self.start,
      end: self.end,
    }
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
