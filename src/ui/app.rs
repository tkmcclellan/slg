use crate::line_manager::LineManager;
use std::io;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Layout};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Widget, Wrap};
use tui::Frame;
use tui_textarea::{Input, Key, TextArea};

pub struct App<'a> {
    textarea: TextArea<'a>,
    line_manager: LineManager,
    layout: Layout,
    filter: String,
    command_string: &'a str,
    scroll: u16,
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
            scroll: 0,
        }
    }

    pub fn poll_for_filter(&mut self) -> io::Result<bool> {
        if crossterm::event::poll(Duration::from_millis(50))? {
            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => return Ok(false),
                Input { key: Key::Up, .. } => self.scroll_up(),
                Input { key: Key::Down, .. } => self.scroll_down(),
                input => {
                    if self.textarea.input(input) {
                        self.filter = self.textarea.lines()[0].clone();
                    }
                }
            }
        }

        Ok(true)
    }

    fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    fn scroll_down(&mut self) {
        self.scroll += 1;
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
        let line_spans = list_lines
            .iter()
            .cloned()
            .map(|x| Spans::from(Span::raw(x)))
            .collect::<Vec<Spans>>();

        let paragraph = Paragraph::new(line_spans)
            .block(
                Block::default()
                    .title(command_string.to_owned())
                    .borders(Borders::ALL),
            )
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0));

        paragraph
    }
}

fn create_text_area<'a>() -> TextArea<'a> {
    let mut textarea = TextArea::default();
    textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));
    textarea.set_style(Style::default().fg(Color::White));
    textarea.set_cursor_line_style(Style::default());

    textarea
}
