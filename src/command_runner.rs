use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

pub fn process_args<T: Iterator<Item = String>>(mut cli_args: T) -> (String, Vec<String>, String) {
    cli_args.next();

    let command = cli_args.next().unwrap();
    let args = cli_args.collect::<Vec<String>>();
    let command_string = format!("{} {}", &command, args.join(" "));

    (command, args, command_string)
}

pub fn run(command: String, args: Vec<String>) -> Receiver<String> {
    let (send, recv) = channel();

    thread::spawn(move || {
        let mut process = spawn_process(command, args);

        loop {
            if let Some(out) = process.stdout.as_mut() {
                let buf_reader = BufReader::new(out);

                for line in buf_reader.lines().flatten() {
                    send.send(line).unwrap_or(());
                }
            }
        }
    });

    recv
}

fn spawn_process(command: String, args: Vec<String>) -> Child {
    Command::new(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args)
        .spawn()
        .expect("Failed to start command!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processes_args() {
        let command_args = vec![
            "rust_bin".to_string(),
            "/bin/ls".to_string(),
            "-la".to_string(),
            "./".to_string(),
        ];

        let (command, args, command_string) = process_args(command_args.iter().cloned());

        assert_eq!(command, "/bin/ls".to_string());
        assert_eq!(args, vec!["-la".to_string(), "./".to_string()]);
        assert_eq!(command_string, "/bin/ls -la ./".to_string());
    }
}
