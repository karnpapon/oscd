use super::task::{monitor, send, Task};
use crate::{DEFAULT_IP, DEFAULT_PORT};
use inquire::{
  error::InquireResult,
  ui::{Attributes, Color, RenderConfig, StyleSheet, Styled},
  CustomType, Select, Text,
};

pub fn get_render_config() -> RenderConfig {
  let mut render_config = RenderConfig::default();
  render_config.prompt_prefix = Styled::new(">").with_fg(Color::LightRed);
  render_config.error_message = render_config
    .error_message
    .with_prefix(Styled::new("❌").with_fg(Color::LightRed));

  render_config.answer = StyleSheet::new()
    .with_attr(Attributes::ITALIC)
    .with_fg(Color::LightYellow);

  render_config.help_message = StyleSheet::new().with_fg(Color::DarkYellow);

  render_config
}

pub fn prompt() -> InquireResult<()> {
  let tasks = vec![
    Task::Monitor("monitor OSC messages".to_string()),
    Task::Send("send OSC messages".to_string()),
  ];

  let task = Select::new("What do you want to do?", tasks)
    .prompt()
    .unwrap();
  let port: u16 = CustomType::new("What port do you want to connect to?")
    .with_default((DEFAULT_PORT, &|i| format!("{}", i)))
    .with_error_message("Please type a valid number")
    .prompt()
    .unwrap();

  match task {
    Task::Monitor(_) => monitor(port),
    Task::Send(_) => {
      let address: String = Text::new("What host IP do you want to connect to?")
        .with_default(DEFAULT_IP)
        .prompt()
        .unwrap();
      send(port, address)
    }
  };

  Ok(())
}
