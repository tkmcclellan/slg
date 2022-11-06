mod command_runner;
mod line_manager;
mod ui;

use std::{env, io};

fn main() -> io::Result<()> {
    let (command, args, command_string) = command_runner::process_args(env::args());

    let receiver = command_runner::run(command, args);
    ui::run(command_string, receiver)?;

    Ok(())
}
