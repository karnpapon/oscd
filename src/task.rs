use colored::*;
use rosc::OscType;
use rustyline::completion::FilenameCompleter;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{CompletionType, Config, EditMode, Editor};
use rustyline_derive::{Completer, Helper, Hinter, Validator};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::fmt;
use std::io::{stdout, Write};
use std::thread;
use tabled::Table;
use termion::screen::*;

use super::analyser::lexer::Lexer;
use super::analyser::parser::{parse_message, Expr, Literal, Parser, Stmt};
use super::analyser::token::Tokens;
use super::osc;
use super::table::{CodeEditor, THEME};

#[derive(Helper, Completer, Hinter, Validator)]
pub struct MyHelper {
  #[rustyline(Completer)]
  completer: FilenameCompleter,
  highlighter: MatchingBracketHighlighter,
  #[rustyline(Validator)]
  validator: MatchingBracketValidator,
  #[rustyline(Hinter)]
  hinter: HistoryHinter,
  colored_prompt: String,
}

impl Highlighter for MyHelper {
  fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
    &'s self,
    prompt: &'p str,
    default: bool,
  ) -> Cow<'b, str> {
    if default {
      Borrowed(&self.colored_prompt)
    } else {
      Borrowed(prompt)
    }
  }

  // https://www.lihaoyi.com/post/BuildyourownCommandLinewithANSIescapecodes.html#256-colors
  fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
    Owned("\x1b[38;5;240m".to_owned() + hint + "\x1b[0m")
  }

  fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
    self.highlighter.highlight(line, pos)
  }

  fn highlight_char(&self, line: &str, pos: usize) -> bool {
    self.highlighter.highlight_char(line, pos)
  }
}

pub enum Task {
  Monitor(String),
  Send(String),
}

impl fmt::Display for Task {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Task::Monitor(val) => {
        let fmt = format!("{} = {}", "Monitor".blue(), val);
        write!(f, "{}", fmt)
      }
      Task::Send(val) => {
        let fmt = format!("{} = {}", "Send".green(), val);
        write!(f, "{}", fmt)
      }
    }
  }
}

pub fn monitor(port: u16) {
  let recv = osc::receiver(port).expect("Could not connect to receiver address");
  loop {
    println!("{:?}", recv.recv().unwrap());
  }
}

pub fn send(port: u16, address: String) {
  let config = Config::builder()
    .history_ignore_space(true)
    .completion_type(CompletionType::List)
    .edit_mode(EditMode::Emacs)
    .build();
  let h = MyHelper {
    completer: FilenameCompleter::new(),
    highlighter: MatchingBracketHighlighter::new(),
    hinter: HistoryHinter {},
    colored_prompt: "".to_owned(),
    validator: MatchingBracketValidator::new(),
  };

  let mut rl = Editor::with_config(config).unwrap();
  rl.set_helper(Some(h));
  let mut screen = AlternateScreen::from(stdout());
  println!( "{}",
    &format!( "\x1b[38;5;242m{} \x1b[38;5;242m{} \x1b[38;5;242m{} \x1b[38;5;242m{} \x1b[38;5;242m{} \x1b[38;5;242m{} \x1b[38;5;242m{} \x1b[38;5;242m{} \x1b[38;5;242m{}\x1b[38;5;242m{}",
      format!("Sending OSC messages to {:?}: {:?} \n",address, port).bold(),
      "Use the following format to send messages: <address> <value>\n",
      "- <address> is osc path to communicate with\n",
      "- <value> is a number or a string without wrapping in double quotes (can have multiple values) \n",
      " . Example:", "/s_new \"default\" -1 0 0 \"freq\" 850\n",
      " . will be parsed as", "(\"s_new\",[String(\"default\"), Int(-1), Int(0), Int(0), String(\"freq\"), Int(850)])\n",
      "- to exit = Ctrl-C",
      "\n",
    )
  );
  screen.flush().unwrap();

  let handler = thread::spawn(move || loop {
    let p = "> ".to_string();
    rl.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{p}\x1b[0m");
    let readline = rl.readline(&p);

    match readline {
      Err(err) => {
        println!("Error: {:?}", err);
        break;
      }
      Ok(input) => {
        let (osc_msg_vec, lex_error) = Lexer::analyse(&input);
        let tokens = Tokens::new(&osc_msg_vec);
        let vec = Vec::new();
        let (_, stmt) = Parser::parse_tokens(tokens).unwrap_or((Tokens::new(&vec), Vec::new()));

        match (stmt.split_first(), lex_error.is_empty()) {
          (Some((first, tail)), true) => {
            match first {
              Stmt::ExprStmt(Expr::Lit(Literal::OscPath(osc_path))) => {
                let argument_msg = tail
                  .iter()
                  .map(|x| match x {
                    Stmt::ExprStmt(v) => parse_message(v),
                  })
                  .collect::<Vec<OscType>>();
                send_packet(port, address.clone(), osc_path, argument_msg);
              }
              _ => println!(
                "{}{}",
                "[ERROR]: ".to_string().red().dimmed(),
                "osc path should start with / eg. /s_new".white().dimmed()
              ),
            };
          }
          (_, _) => {
            let mut data = vec![];
            for err in lex_error {
              let errors = err.print_error();
              data.push(CodeEditor::new(errors.0, errors.1, errors.2, errors.3));
            }

            let mut table = Table::new(data);
            table.with(THEME);

            println!(
              "\n{}{}",
              "[ERROR]: ".to_string().red().dimmed(),
              "parsing msg".to_string().white().dimmed()
            );
            println!("{table}\n");
          }
        }

        rl.add_history_entry(input.as_str()).unwrap();
      }
    }
  });

  handler.join().unwrap();
}

pub fn send_packet(port: u16, address: String, osc_path: &str, osc_args: Vec<OscType>) {
  let full_address = format!("{}:{}", address, port);

  let sender = osc::sender()
    .expect("Could not bind to default socket")
    .connect(full_address)
    .expect("Could not connect to socket at address");

  let packet = (osc_path, osc_args);
  match sender.send(packet.clone()) {
    Ok(value) => println!(
      "{}{}",
      "[SUCCESS]: ".green().dimmed(),
      format!("packets = {:?}", packet).white().dimmed()
    ),
    Err(e) => println!(
      "{}{}",
      "[ERR]: ".red().dimmed(),
      format!("{:?}", e).white().dimmed()
    ),
  }
}
