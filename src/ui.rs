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
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Terminal;
use tui_textarea::{Input, Key, TextArea};

fn create_text_area<'a>() -> TextArea<'a> {
    let mut textarea = TextArea::default();
    textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));
    textarea.set_style(Style::default().fg(Color::White));
    textarea.set_cursor_line_style(Style::default());

    textarea
}

pub fn start_ui(command_string: String, receiver: Receiver<String>) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    let mut manager = LineManager::new(5000);

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;
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

        let new_line_option: Option<String> = match receiver.try_recv() {
            Ok(value) => Some(value),
            _ => None,
        };

        if let Some(new_line) = new_line_option {
            manager.add_line(new_line)
        }

        let list_lines = manager.filter(filter.clone());

        term.draw(|f| {
            let chunks = layout.split(f.size());
            let widget = textarea.widget();
            f.render_widget(widget, chunks[0]);

            let list_items = list_lines
                .iter()
                .cloned()
                .map(ListItem::new)
                .collect::<Vec<ListItem>>();

            let list = List::new(list_items)
                .block(
                    Block::default()
                        .title(command_string.clone())
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::White));

            f.render_widget(list, chunks[1]);
        })?;
    }

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    Ok(())
}
