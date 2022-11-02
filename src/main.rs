mod command_runner;
mod line_manager;
mod ui;

use std::{io, env};


fn main() -> io::Result<()> {
    let (command, args, command_string) = command_runner::process_args(env::args());

    let receiver = command_runner::start_command(command, args);
    ui::start_ui(command_string, receiver)?;

    Ok(())
}
