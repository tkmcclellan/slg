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
    command: String,

    /// Arguments for command.
    args: Vec<String>,

    /// Number of lines to retain in memory. Defaults to 1M.
    #[arg(long)]
    num_lines: Option<usize>,
}

fn main() -> io::Result<()> {
    let cli = CliArgs::parse();
    let command_string = cli.command.clone() + " " + &cli.args.join(" ");

    let line_limit = match cli.num_lines {
        Some(value) => value,
        None => DEFAULT_LINE_LIMIT,
    };

    match command_runner::run(cli.command, cli.args) {
        Ok(receiver) => ui::run(command_string, receiver, line_limit)?,
        Err(error_string) => println!("{}", error_string),
    }

    Ok(())
}
