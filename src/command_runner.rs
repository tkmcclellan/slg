use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

pub fn run(command: String, args: Vec<String>) -> Result<Receiver<String>, String> {
    let (send, recv) = channel();

    match spawn_process(&command, &args) {
        Ok(mut process) => {
            thread::spawn(move || loop {
                if let Some(out) = process.stdout.as_mut() {
                    let buf_reader = BufReader::new(out);

                    for line in buf_reader.lines().flatten() {
                        send.send(line).unwrap_or(());
                    }
                }
            });

            Ok(recv)
        }
        Err(error) => Err(format!(
            "Failed to start command \"{} {:?}\": {}",
            command, args, error
        )),
    }
}

fn spawn_process(command: &String, args: &Vec<String>) -> std::io::Result<Child> {
    Command::new(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args)
        .spawn()
}
