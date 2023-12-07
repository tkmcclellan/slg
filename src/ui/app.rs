use crate::line_manager::LineManager;
use cansi::v3::categorise_text;
use rayon::prelude::*;
use regex::Regex;
use std::cmp::max;
use std::io;
use std::sync::{Arc, Mutex};
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
    line_manager: Arc<Mutex<LineManager>>,
    filter: Regex,
    layout: Layout,
    command_string: &'a str,
}

pub enum PollResult {
    NewFilter,
    NoNewFilter,
    Escape,
}

impl<'a> App<'a> {
    pub fn new(line_manager: Arc<Mutex<LineManager>>, command_string: &'a str) -> App<'a> {
        let textarea = create_text_area();
        let layout =
            Layout::default().constraints([Constraint::Length(3), Constraint::Min(1)].as_slice());
        let filter = Regex::new("").unwrap();

        App {
            textarea,
            line_manager,
            layout,
            command_string,
            filter,
        }
    }

    pub fn update_filter(&mut self, filter: String) {
        if let Ok(filter_regex) = Regex::new(&filter) {
            self.filter = filter_regex;
        }
    }

    pub fn poll_for_filter(&mut self) -> io::Result<PollResult> {
        if crossterm::event::poll(Duration::ZERO)? {
            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => return Ok(PollResult::Escape),
                Input { key: Key::Up, .. }
                | Input {
                    key: Key::MouseScrollUp,
                    ..
                } => self.scroll_up(1),
                Input { key: Key::Down, .. }
                | Input {
                    key: Key::MouseScrollDown,
                    ..
                } => self.scroll_down(1),
                Input {
                    key: Key::Char('k'),
                    ctrl: true,
                    ..
                } => self.scroll_up(10),
                Input {
                    key: Key::Char('j'),
                    ctrl: true,
                    ..
                } => self.scroll_down(10),
                Input {
                    key: Key::Char('s'),
                    ctrl: true,
                    ..
                } => self.scroll_to_bottom(),
                Input {
                    key: Key::Char('w'),
                    ctrl: true,
                    ..
                } => self.scroll_to_top(),
                input => {
                    if self.textarea.input(input) {
                        self.update_filter(self.textarea.lines()[0].clone());

                        return Ok(PollResult::NewFilter);
                    }
                }
            }
        }

        Ok(PollResult::NoNewFilter)
    }

    fn scroll_up(&self, scroll: usize) {
        self.line_manager.lock().unwrap().scroll_up(scroll)
    }

    fn scroll_down(&self, scroll: usize) {
        self.line_manager.lock().unwrap().scroll_down(scroll)
    }

    fn scroll_to_top(&self) {
        self.line_manager.lock().unwrap().scroll_to_top()
    }

    fn scroll_to_bottom(&self) {
        self.line_manager.lock().unwrap().scroll_to_bottom()
    }

    pub fn draw_in_frame<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = self.layout.split(f.size());
        let widget = self.textarea.widget();
        let num_lines = max(0, chunks[1].height - 2);
        f.render_widget(widget, chunks[0]);

        let line_manager = self.line_manager.lock().unwrap();

        let list_lines = line_manager.filter(&self.filter, num_lines as usize);
        let num_lines = line_manager.count();
        let lines_widget = self.render_lines_widget(&list_lines, self.command_string, num_lines);

        f.render_widget(lines_widget, chunks[1]);
    }

    fn render_lines_widget<'b>(
        &'b self,
        list_lines: &'b Vec<String>,
        command_string: &str,
        num_lines: usize,
    ) -> impl Widget + 'b {
        let mut line_spans = Vec::<Spans>::new();

        list_lines
            .par_iter()
            .map(|line| Spans::from(self.color_line(line)))
            .collect_into_vec(&mut line_spans);

        let paragraph = Paragraph::new(line_spans)
            .block(
                Block::default()
                    .title(format!("{} - {}", command_string.to_owned(), num_lines,))
                    .borders(Borders::ALL),
            )
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        paragraph
    }

    fn color_line<'b>(&self, line: &'b str) -> Vec<Span<'b>> {
        categorise_text(&line)
            .iter()
            .map(|result| {
                let mut style = Style::default();

                if let Some(fg) = result.fg {
                    style = style.fg(cansi_color_to_tui_color(fg))
                }

                if let Some(bg) = result.bg {
                    style = style.bg(cansi_color_to_tui_color(bg))
                }

                Span::styled(result.text, style)
            })
            .collect::<Vec<Span>>()
    }

    fn highlight_line(&self, line: String) -> Spans {
        let highlighted_style = Style::default().fg(Color::Black).bg(Color::Yellow);

        if !self.filter.as_str().is_empty() {
            let mut spans = Vec::new();

            let mut last_index = 0;

            for rmatch in self.filter.find_iter(&line) {
                let match_range = (rmatch.start(), rmatch.end());

                match match_range {
                    // front of string
                    (0, back) => {
                        spans.push(Span::styled(rmatch.as_str().to_string(), highlighted_style));
                        last_index = back;
                    }
                    // back of string
                    (_, back) if back == line.len() - 1 => {
                        spans.push(Span::styled(rmatch.as_str().to_string(), highlighted_style));
                    }
                    // middle of string
                    (front, back) => {
                        let before_match = &line[last_index..front];

                        if !before_match.is_empty() {
                            spans.push(Span::from(before_match.to_string()));
                        }

                        spans.push(Span::styled(rmatch.as_str().to_string(), highlighted_style));

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
    }
}

fn create_text_area<'a>() -> TextArea<'a> {
    let mut textarea = TextArea::default();
    textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));
    textarea.set_style(Style::default());
    textarea.set_cursor_line_style(Style::default());

    textarea
}

fn cansi_color_to_tui_color(cansi_color: cansi::v3::Color) -> tui::style::Color {
    match cansi_color {
        cansi::v3::Color::Black => tui::style::Color::Black,
        cansi::v3::Color::Red => tui::style::Color::Red,
        cansi::v3::Color::Green => tui::style::Color::Green,
        cansi::v3::Color::Yellow => tui::style::Color::Yellow,
        cansi::v3::Color::Blue => tui::style::Color::Blue,
        cansi::v3::Color::Magenta => tui::style::Color::Magenta,
        cansi::v3::Color::Cyan => tui::style::Color::Cyan,
        cansi::v3::Color::White => tui::style::Color::White,
        cansi::v3::Color::BrightBlack => tui::style::Color::Gray,
        cansi::v3::Color::BrightRed => tui::style::Color::LightRed,
        cansi::v3::Color::BrightGreen => tui::style::Color::LightGreen,
        cansi::v3::Color::BrightYellow => tui::style::Color::LightYellow,
        cansi::v3::Color::BrightBlue => tui::style::Color::LightBlue,
        cansi::v3::Color::BrightMagenta => tui::style::Color::LightMagenta,
        cansi::v3::Color::BrightCyan => tui::style::Color::LightCyan,
        cansi::v3::Color::BrightWhite => tui::style::Color::White,
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
