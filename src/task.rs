use colored::*;
use inquire::Text;
use nom::Finish;
// use nannou_osc as osc;
use rosc::OscType;
use std::fmt;
use std::io::{stdout, Write};
use std::thread;
use termion::screen::*;

use super::analyser::lexer::Lexer;
use super::analyser::parser::{parse_message, Expr, Ident, Literal, Parser, Stmt};
use super::analyser::token::Tokens;
use super::osc;
use super::prompt;

pub enum Task {
  Monitor(String),
  Send(String),
}

impl fmt::Display for Task {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &*self {
      Task::Monitor(val) => write!(f, "monitor = {}", val),
      Task::Send(val) => write!(f, "send = {}", val),
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
  let mut screen = AlternateScreen::from(stdout());
  println!( "{}",
    &format!( "{} {} {} {} {} {} {} {} {}",
      format!("Sending OSC messages to {:?}: {:?} \n",address, port).bold().white().dimmed(),
      "Use the following format to send messages: <address> <value>\n".white().dimmed(),
      "- <address> is osc path to communicate with\n".green().dimmed(),
      "- <value> is a number or a string without wrapping in double quotes (can have multiple values) \n".green().dimmed(),
      " . Example:".white().dimmed(), "/s_new default -1 0 0 freq 850\n".cyan().dimmed(),
      " . will be parsed as".white().dimmed(), "(\"s_new\",[String(\"default\"), Int(-1), Int(0), Int(0), String(\"freq\"), Int(850)])\n".cyan().dimmed(),
      "- to exit = Ctrl-C".green().dimmed(),
    ).dimmed()
  );
  screen.flush().unwrap();

  let handler = thread::spawn(move || loop {
    // loop {
    let msg = Text::new("");
    let osc_msg = msg
      .with_render_config(prompt::get_render_config())
      .prompt()
      .unwrap();
    let osc_msg_vec = Lexer::lex_tokens(osc_msg.as_bytes()).finish().unwrap();

    let tokens = Tokens::new(&osc_msg_vec.1);
    let (_, stmt) =
      Parser::parse_tokens(tokens).unwrap_or({ (Tokens::new(&Vec::new()), Vec::new()) });

    match stmt.split_first() {
      Some((first, tail)) => {
        let osc_path = match first {
          Stmt::ExprStmt(stmt) => match stmt {
            Expr::Lit(Literal::OscPath(path)) => path,
            // Expr::Ident(Ident(val)) => val,
            _ => "/osc/adress/is/needed",
          },
        };
        if (osc_path) == ":q" {
          break;
        }
        let argument_msg = tail
          .iter()
          .map(|x| match x {
            Stmt::ExprStmt(v) => parse_message(v),
          })
          .collect::<Vec<OscType>>();
        send_packet(port, address.clone(), osc_path, argument_msg);
      }
      None => println!("[ERROR] parsing msg: please check if argument is valid."),
    }

    // }
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
  println!("[SUCCESS] packets = {:?}", packet);
  sender.send(packet).ok();
}
