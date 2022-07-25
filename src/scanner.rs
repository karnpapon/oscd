/// basically just a lightweight lexer, tokenizer
/// to capture string with whitespace inside single/double quotes, array, etc.
use core::mem;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseError;

impl Display for ParseError {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    f.write_str("missing closing quote")
  }
}

impl std::error::Error for ParseError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataType {
  Scalar,
  Compound,
}

enum CaptureSymbol {
  Delimiter,
  Backslash,
  Unquoted,
  UnquotedBackslash,
  SingleQuoted,
  DoubleQuoted,
  OpenSquareBracket,
  // CloseSquareBracket,
  Unbracket,
}

pub(crate) trait IntoArgs {
  fn try_into_args(&self) -> Result<Vec<(String, DataType)>, ParseError>;
  fn try_into_compound_args(&self) -> Result<Vec<(String, DataType)>, ParseError>;
}

impl<S: std::ops::Deref<Target = str>> IntoArgs for S {
  fn try_into_args(&self) -> Result<Vec<(String, DataType)>, ParseError> {
    use CaptureSymbol::*;

    let mut into = Vec::new();
    let mut words = Vec::new();
    let mut word = String::new();
    let mut chars = self.chars();
    let mut state = Delimiter;

    loop {
      let c = chars.next();
      state = match state {
        Delimiter => match c {
          None => break,
          Some('\'') => SingleQuoted,
          Some('\"') => DoubleQuoted,
          Some('\\') => Backslash,
          Some('\n') => Delimiter,
          Some('[') => OpenSquareBracket,
          Some(c) => {
            word.push(c);
            Unquoted
          }
        },
        Backslash => match c {
          None => {
            word.push('\\');
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Scalar,
            ));
            words.remove(0);
            break;
          }
          Some('\n') => Delimiter,
          Some(c) => {
            word.push(c);
            Unquoted
          }
        },
        Unquoted => match c {
          None => {
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Scalar,
            ));
            words.remove(0);
            break;
          }
          Some('\'') => SingleQuoted,
          Some('\"') => DoubleQuoted,
          Some('\\') => UnquotedBackslash,
          Some('\t') | Some(' ') | Some('\n') => {
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Scalar,
            ));
            words.remove(0);
            Delimiter
          }
          Some(c) => {
            word.push(c);
            Unquoted
          }
        },
        Unbracket => match c {
          None => {
            words.push(mem::take(&mut format!("[{}]", &mut word)));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Compound,
            ));
            words.remove(0);
            break;
          }
          Some('\'') => SingleQuoted,
          Some('\"') => DoubleQuoted,
          Some('\\') => UnquotedBackslash,
          Some('\t') | Some(' ') | Some('\n') => {
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Compound,
            ));
            words.remove(0);
            Delimiter
          }
          Some(c) => {
            word.push(c);
            Unbracket
          }
        },
        UnquotedBackslash => match c {
          None => {
            word.push('\\');
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Scalar,
            ));
            words.remove(0);
            break;
          }
          Some('\n') => Unquoted,
          Some(c) => {
            word.push(c);
            Unquoted
          }
        },
        SingleQuoted => match c {
          None => return Err(ParseError),
          Some('\'') => Unquoted,
          Some(c) => {
            word.push(c);
            SingleQuoted
          }
        },
        DoubleQuoted => match c {
          None => return Err(ParseError),
          Some('\"') | Some('\n') => Unquoted,
          Some(c) => {
            word.push(c);
            DoubleQuoted
          }
        },
        OpenSquareBracket => match c {
          None => return Err(ParseError),
          Some(']') => Unbracket,
          Some(c) => {
            word.push(c);
            OpenSquareBracket
          }
        },
      }
    }

    Ok(into)
  }

  // TODO:
  fn try_into_compound_args(&self) -> Result<Vec<(String, DataType)>, ParseError> {
    use CaptureSymbol::*;

    let mut into = Vec::new();
    let mut words = Vec::new();
    let mut word = String::new();
    let mut chars = self.chars();
    let mut state = Delimiter;

    loop {
      let c = chars.next();
      state = match state {
        Delimiter => match c {
          None => break,
          Some('\'') => SingleQuoted,
          Some('\"') => DoubleQuoted,
          Some('\\') => Backslash,
          Some('\n') | Some(' ') => Delimiter,
          Some('[') => OpenSquareBracket,
          Some(c) => {
            word.push(c);
            Unbracket
          }
        },
        Backslash => match c {
          None => {
            word.push('\\');
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Scalar,
            ));
            words.remove(0);
            break;
          }
          Some('\n') => Delimiter,
          Some(c) => {
            word.push(c);
            Unquoted
          }
        },
        Unquoted => match c {
          None => {
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Scalar,
            ));
            words.remove(0);
            break;
          }
          Some('\'') => SingleQuoted,
          Some('\"') => DoubleQuoted,
          Some('\\') => UnquotedBackslash,
          Some('\t') | Some(' ') | Some('\n') => {
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Scalar,
            ));
            words.remove(0);
            Delimiter
          }
          Some(c) => {
            word.push(c);
            Unquoted
          }
        },
        Unbracket => match c {
          None => {
            words.push(mem::take(&mut format!("[{}]", &mut word)));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Compound,
            ));
            words.remove(0);
            break;
          }
          Some('\'') => SingleQuoted,
          Some('\"') => DoubleQuoted,
          Some('\\') => UnquotedBackslash,
          Some('\t') | Some(' ') | Some('\n') => {
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Scalar,
            ));
            words.remove(0);
            OpenSquareBracket
          }
          Some(c) => {
            word.push(c);
            Unbracket
          }
        },
        UnquotedBackslash => match c {
          None => {
            word.push('\\');
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Scalar,
            ));
            words.remove(0);
            break;
          }
          Some('\n') => Unquoted,
          Some(c) => {
            word.push(c);
            Unquoted
          }
        },
        SingleQuoted => match c {
          None => return Err(ParseError),
          Some('\'') => Unquoted,
          Some(c) => {
            word.push(c);
            SingleQuoted
          }
        },
        DoubleQuoted => match c {
          None => return Err(ParseError),
          Some('\"') | Some('\n') => Unquoted,
          Some(c) => {
            word.push(c);
            DoubleQuoted
          }
        },
        OpenSquareBracket => match c {
          None => return Err(ParseError),
          Some(' ') => {
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Scalar,
            ));
            words.remove(0);
            OpenSquareBracket
          }
          Some(']') | Some('\t') | Some('\n') => {
            words.push(mem::take(&mut word));
            into.push((
              words.clone().into_iter().collect::<String>(),
              DataType::Scalar,
            ));
            words.remove(0);
            Delimiter
          }
          Some(c) => {
            word.push(c);
            OpenSquareBracket
          }
        },
      }
    }

    Ok(into)
  }
}

// /s_new [20.5 123.2_f64 thebkacj false] true
