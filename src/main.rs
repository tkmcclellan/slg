mod command_runner;
mod line_manager;
mod ui;

use clap::Parser;
use std::io;

const DEFAULT_LINE_LIMIT: usize = 1_000_000;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// Command to be run.
    command: Vec<String>,

    /// Number of lines to retain in memory. Defaults to 1M.
    #[arg(long)]
    num_lines: Option<usize>,
}

fn main() -> io::Result<()> {
    let cli = CliArgs::parse();
    let (command, args, command_string) = command_runner::process_args(cli.command.iter().cloned());
    let line_limit = match cli.num_lines {
        Some(value) => value,
        None => DEFAULT_LINE_LIMIT,
    };

    let receiver = command_runner::run(command, args);
    ui::run(command_string, receiver, line_limit)?;

    Ok(())
}
