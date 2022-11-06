use crate::line_manager::LineManager;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListItem, Widget};
use tui::Terminal;
use tui_textarea::{Input, Key, TextArea};

pub fn run(command_string: String, receiver: Receiver<String>) -> io::Result<()> {
    let mut manager = LineManager::new(5000);
    let mut term = setup_terminal()?;
    let mut textarea = create_text_area();
    let layout =
        Layout::default().constraints([Constraint::Length(3), Constraint::Min(1)].as_slice());
    let mut filter = String::new();

    loop {
        if crossterm::event::poll(Duration::from_millis(50))? {
            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => break,
                input => {
                    if textarea.input(input) {
                        filter = textarea.lines()[0].clone();
                    }
                }
            }
        }

        if let Some(new_line) = try_receive_new_line(&receiver) {
            manager.add_line(new_line)
        }

        let list_lines = manager.filter(filter.clone());

        term.draw(|f| {
            let chunks = layout.split(f.size());
            let widget = textarea.widget();
            f.render_widget(widget, chunks[0]);

            let lines_widget = render_lines_widget(list_lines, &command_string);

            f.render_widget(lines_widget, chunks[1]);
        })?;
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

fn create_text_area<'a>() -> TextArea<'a> {
    let mut textarea = TextArea::default();
    textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));
    textarea.set_style(Style::default().fg(Color::White));
    textarea.set_cursor_line_style(Style::default());

    textarea
}

fn try_receive_new_line(receiver: &Receiver<String>) -> Option<String> {
    match receiver.try_recv() {
        Ok(value) => Some(value),
        _ => None,
    }
}

fn render_lines_widget(list_lines: Vec<String>, command_string: &str) -> impl Widget {
    let list_items = list_lines
        .iter()
        .cloned()
        .map(ListItem::new)
        .collect::<Vec<ListItem>>();

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(command_string.to_owned())
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    list
}
