use tabled::{
  settings::{
    object::{FirstRow, Rows},
    style::{On, Style},
    Alignment, Modify, ModifyList, Padding, Settings,
  },
  Tabled,
};

#[derive(Tabled)]
pub struct CodeEditor {
  msg_type: String,
  range: String,
  input: String,
  message: String,
}

impl CodeEditor {
  pub fn new(range: String, input: String, message: String, msg_type: String) -> Self {
    Self {
      msg_type,
      range,
      input,
      message,
    }
  }
}

type TableTheme = Settings<
  Settings<Settings<Settings, Style<On, On, On, On, (), On, 1, 0>>, Padding>,
  ModifyList<FirstRow, Alignment>,
>;

pub const THEME: TableTheme = Settings::empty()
  .with(Style::rounded())
  .with(Padding::new(1, 1, 0, 0))
  .with(Modify::list(Rows::first(), Alignment::center()));
