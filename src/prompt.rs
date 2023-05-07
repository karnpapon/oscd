use super::task::{monitor, send, Task};
use crate::{DEFAULT_IP, DEFAULT_PORT};
use inquire::{error::InquireResult, CustomType, Select, Text};

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
