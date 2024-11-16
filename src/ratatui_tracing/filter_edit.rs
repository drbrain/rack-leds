use std::sync::{Arc, Mutex};

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Padding, Widget},
};
use tracing_subscriber::filter::{self, Directive};
use tui_textarea::{CursorMove, TextArea};

#[derive(Clone, Default)]
pub struct FilterEdit<'a> {
    text_area: Arc<Mutex<TextArea<'a>>>,
    error: Arc<Mutex<Option<filter::ParseError>>>,
}

impl<'a> FilterEdit<'a> {
    pub fn new(text: String) -> Self {
        let text_area = new_text_area(text);

        Self {
            text_area: Arc::new(Mutex::new(text_area)),
            error: Default::default(),
        }
    }

    pub fn clear(&self) {
        let mut text_area = self.text_area.lock().unwrap();

        *text_area = new_text_area("");
    }

    pub fn directive(&self) -> Option<Directive> {
        let mut text_area = self.text_area.lock().unwrap();

        let text = text_area.lines().join("");

        match text.parse::<Directive>() {
            Ok(directive) => {
                *text_area = new_text_area("");

                Some(directive)
            }
            Err(parse_error) => {
                drop(text_area);

                let mut error = self.error.lock().unwrap();

                *error = Some(parse_error);

                None
            }
        }
    }

    pub fn insert(&self, text: String) {
        let mut text_area = self.text_area.lock().unwrap();

        *text_area = new_text_area(text);
    }

    pub fn validate(&self) -> Option<filter::ParseError> {
        let text_area = self.text_area.lock().unwrap();

        let text = text_area.lines().join("");

        text.parse::<Directive>().err()
    }

    pub fn key(&self, key: KeyEvent) {
        let mut text_area = self.text_area.lock().unwrap();

        text_area.input(key);
    }

    pub fn text(&self) -> String {
        let text_area = self.text_area.lock().unwrap();

        text_area.lines().join("")
    }
}

impl Widget for &FilterEdit<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Clear.render(area, buf);

        let dialog_border = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from("Filters — Add").bold())
            .title_bottom(Line::from("Esc to dismiss").right_aligned().italic())
            .padding(Padding::symmetric(1, 0));

        let [input_area, error_area, _] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(dialog_border.inner(area));

        dialog_border.render(area, buf);

        let parse_error = self.validate();
        let input_border_color = if parse_error.is_none() {
            Color::Green
        } else {
            Color::Red
        };

        let input_border = Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(input_border_color));

        if let Some(parse_error) = parse_error {
            Line::from(parse_error.to_string())
                .style(Style::default().fg(Color::Red))
                .render(error_area, buf);
        }

        let mut text_area = self.text_area.lock().unwrap();
        text_area.set_block(input_border);

        text_area.render(input_area, buf);
    }
}

fn new_text_area<'a>(text: impl Into<String>) -> TextArea<'a> {
    let mut text_area = TextArea::new(vec![text.into()]);
    text_area.set_cursor_line_style(Style::default());
    text_area.set_placeholder_style(Style::default().dim().italic());
    text_area.set_placeholder_text("target=level");
    text_area.move_cursor(CursorMove::Bottom);
    text_area.move_cursor(CursorMove::End);

    text_area
}
