use crate::line_manager::LineManager;
use std::io;
use std::time::Duration;
use tui::backend::Backend;
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
    command_string: &'a str,
    scroll: u16,
}

pub enum PollResult {
    NewFilter,
    NoNewFilter,
    Escape,
}

impl<'a> App<'a> {
    pub fn new(line_capacity: usize, command_string: &'a str) -> App<'a> {
        let line_manager = LineManager::new(line_capacity);
        let textarea = create_text_area();
        let layout =
            Layout::default().constraints([Constraint::Min(1), Constraint::Length(3)].as_slice());

        App {
            textarea,
            line_manager,
            layout,
            command_string,
            scroll: 0,
        }
    }

    pub fn poll_for_filter(&mut self) -> io::Result<PollResult> {
        if crossterm::event::poll(Duration::ZERO)? {
            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => return Ok(PollResult::Escape),
                Input { key: Key::Up, .. } => self.scroll_up(),
                Input { key: Key::Down, .. } => self.scroll_down(),
                input => {
                    if self.textarea.input(input) {
                        self.line_manager
                            .update_filter(self.textarea.lines()[0].clone());

                        return Ok(PollResult::NewFilter);
                    }
                }
            }
        }

        Ok(PollResult::NoNewFilter)
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

    pub fn draw_in_frame<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = self.layout.split(f.size());
        let widget = self.textarea.widget();
        f.render_widget(widget, chunks[1]);

        let list_lines = self.line_manager.filter(f.size().height as usize);
        let lines_widget = self.render_lines_widget(list_lines, self.command_string);

        f.render_widget(lines_widget, chunks[0]);
    }

    fn render_lines_widget(
        &mut self,
        list_lines: Vec<String>,
        command_string: &str,
    ) -> impl Widget + '_ {
        let highlighted_style = Style::default().fg(Color::Black).bg(Color::Yellow);

        let line_spans = list_lines
            .iter()
            .cloned()
            .map(|line| {
                if !self.line_manager.filter.as_str().is_empty() {
                    let mut spans = Vec::new();

                    let mut last_index = 0;

                    for rmatch in self.line_manager.filter.find_iter(&line) {
                        let match_range = (rmatch.start(), rmatch.end());

                        match match_range {
                            // front of string
                            (0, back) => {
                                spans.push(Span::styled(
                                    rmatch.as_str().to_string(),
                                    highlighted_style,
                                ));
                                last_index = back;
                            }
                            // back of string
                            (_, back) if back == line.len() - 1 => {
                                spans.push(Span::styled(
                                    rmatch.as_str().to_string(),
                                    highlighted_style,
                                ));
                            }
                            // middle of string
                            (front, back) => {
                                let before_match = &line[last_index..front];

                                if !before_match.is_empty() {
                                    spans.push(Span::from(before_match.to_string()));
                                }

                                spans.push(Span::styled(
                                    rmatch.as_str().to_string(),
                                    highlighted_style,
                                ));

                                last_index = back;
                            }
                        }
                    }

                    if last_index != line.len() - 1 {
                        spans.push(Span::from(line[last_index..line.len()].to_string()));
                    }

                    Spans::from(spans)
                } else {
                    Spans::from(Span::raw(line))
                }
            })
            .collect::<Vec<Spans>>();

        let paragraph = Paragraph::new(line_spans)
            .block(
                Block::default()
                    .title(format!(
                        "{} - {}",
                        command_string.to_owned(),
                        self.line_manager.count()
                    ))
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
    textarea.set_style(Style::default());
    textarea.set_cursor_line_style(Style::default());

    textarea
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_app_scrolls_up() {
        let mut app = App::new(1, "Test");

        app.scroll_up();

        assert_eq!(app.scroll, 0);
    }

    #[test]
    fn scrolled_app_scrolls_up() {
        let mut app = App::new(1, "Test");

        app.scroll = 2;

        app.scroll_up();

        assert_eq!(app.scroll, 1);
    }

    #[test]
    fn new_app_scrolls_down() {
        let mut app = App::new(1, "Test");

        app.scroll_down();

        assert_eq!(app.scroll, 1);
    }

    #[test]
    fn adds_line() {
        let mut app = App::new(1, "Test");

        app.add_line("This is a line!".to_string());

        assert_eq!(app.line_manager.lines, vec!["This is a line!".to_string()]);
    }
}
