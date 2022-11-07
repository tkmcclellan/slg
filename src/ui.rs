mod app;

use app::App;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io;
use std::sync::mpsc::Receiver;
use tui::backend::CrosstermBackend;
use tui::Terminal;

pub fn run(
    command_string: String,
    receiver: Receiver<String>,
    line_limit: usize,
) -> io::Result<()> {
    let mut term = setup_terminal()?;
    let mut app = App::new(line_limit, &command_string);

    loop {
        match app.poll_for_filter() {
            Ok(true) => {}
            _ => break,
        }

        if let Some(new_line) = try_receive_new_line(&receiver) {
            app.add_line(new_line)
        }

        term.draw(|f| app.draw_in_frame(f))?;
    }

    cleanup_terminal(term)?;

    Ok(())
}

fn setup_terminal<'a>() -> io::Result<Terminal<CrosstermBackend<io::StdoutLock<'a>>>> {
    let mut stdout = io::stdout().lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let term = Terminal::new(backend)?;

    Ok(term)
}

fn cleanup_terminal(mut term: Terminal<CrosstermBackend<io::StdoutLock>>) -> io::Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    Ok(())
}

fn try_receive_new_line(receiver: &Receiver<String>) -> Option<String> {
    match receiver.try_recv() {
        Ok(value) => Some(value),
        _ => None,
    }
}
