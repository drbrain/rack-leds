use crossterm::event::KeyEvent;
use ratatui::style::{Style, Stylize};
use tracing_subscriber::filter::{self, Directive};
use tui_textarea::{CursorMove, TextArea};

#[derive(Default)]
pub struct FilterEditState<'a> {
    // Unfortunately TextArea doesn't implement ratatui's StatefulWidget so we have to have one
    // here in the FilterEdit widget's state
    pub(crate) text_area: TextArea<'a>,
    pub(crate) error: Option<filter::ParseError>,
}

impl<'a> FilterEditState<'a> {
    pub fn clear(&mut self) {
        self.text_area = new_text_area("");
    }

    pub fn directive(&mut self) -> Option<Directive> {
        let text = self.text_area.lines().join("");

        match text.parse::<Directive>() {
            Ok(directive) => {
                self.text_area = new_text_area("");

                Some(directive)
            }
            Err(parse_error) => {
                self.error = Some(parse_error);

                None
            }
        }
    }

    pub fn insert(&mut self, text: String) {
        self.text_area = new_text_area(text);
    }

    pub fn validate(&self) -> Option<filter::ParseError> {
        let text = self.text_area.lines().join("");

        text.parse::<Directive>().err()
    }

    pub fn key(&mut self, key: KeyEvent) {
        self.text_area.input(key);
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
