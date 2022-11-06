use crate::line_manager::LineManager;
use std::io;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListItem, Widget};
use tui::Frame;
use tui_textarea::{Input, Key, TextArea};

pub struct App<'a> {
    textarea: TextArea<'a>,
    line_manager: LineManager,
    layout: Layout,
    filter: String,
    command_string: &'a str,
}

impl<'a> App<'a> {
    pub fn new(line_capacity: usize, command_string: &'a str) -> App<'a> {
        let line_manager = LineManager::new(line_capacity);
        let textarea = create_text_area();
        let layout =
            Layout::default().constraints([Constraint::Length(3), Constraint::Min(1)].as_slice());
        let filter = String::new();

        App {
            textarea,
            line_manager,
            layout,
            filter,
            command_string,
        }
    }

    pub fn poll_for_filter(&mut self) -> io::Result<bool> {
        if crossterm::event::poll(Duration::from_millis(50))? {
            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => return Ok(false),
                input => {
                    if self.textarea.input(input) {
                        self.filter = self.textarea.lines()[0].clone();
                    }
                }
            }
        }

        Ok(true)
    }

    pub fn add_line(&mut self, new_line: String) {
        self.line_manager.add_line(new_line);
    }

    pub fn draw_in_frame(&mut self, f: &mut Frame<CrosstermBackend<io::StdoutLock>>) {
        let chunks = self.layout.split(f.size());
        let widget = self.textarea.widget();
        f.render_widget(widget, chunks[0]);

        let list_lines = self.line_manager.filter(self.filter.clone());
        let lines_widget = self.render_lines_widget(list_lines, self.command_string);

        f.render_widget(lines_widget, chunks[1]);
    }

    fn render_lines_widget(
        &mut self,
        list_lines: Vec<String>,
        command_string: &str,
    ) -> impl Widget {
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
}

fn create_text_area<'a>() -> TextArea<'a> {
    let mut textarea = TextArea::default();
    textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));
    textarea.set_style(Style::default().fg(Color::White));
    textarea.set_cursor_line_style(Style::default());

    textarea
}
