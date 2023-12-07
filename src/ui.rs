mod app;

use app::{App, PollResult};
use crossterm::event::DisableMouseCapture;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tui::backend::CrosstermBackend;
use tui::Terminal;

use crate::line_manager::LineManager;

pub fn run(command_string: String, line_manager: Arc<Mutex<LineManager>>) -> io::Result<()> {
    let mut term = setup_terminal()?;
    let mut app = App::new(line_manager, &command_string);

    loop {
        match app.poll_for_filter() {
            Ok(PollResult::NewFilter) => {}
            Ok(PollResult::NoNewFilter) => {}
            Ok(PollResult::Escape) => break,
            _ => break,
        }

        let sleep_duration = time::Duration::from_millis(5);
        thread::sleep(sleep_duration);

        term.draw(|f| app.draw_in_frame(f))?;
    }

    cleanup_terminal(term)?;

    Ok(())
}

fn setup_terminal<'a>() -> io::Result<Terminal<CrosstermBackend<io::StdoutLock<'a>>>> {
    let mut stdout = io::stdout().lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, DisableMouseCapture)?;
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
