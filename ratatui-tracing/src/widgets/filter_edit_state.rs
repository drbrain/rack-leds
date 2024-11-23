use crossterm::event::KeyEvent;
use ratatui::style::{Style, Stylize};
use tracing_subscriber::filter::{self, Directive};
use tui_textarea::{CursorMove, TextArea};

/// State of a [`super::FilterEdit`] widget
///
/// The text of a directive being added or edited and whether it has a parse error
#[derive(Default)]
pub struct FilterEditState<'a> {
    // Unfortunately TextArea doesn't implement ratatui's StatefulWidget so we have to have one
    // here in the FilterEdit widget's state
    pub(crate) text_area: TextArea<'a>,
    pub(crate) error: Option<filter::ParseError>,
}

impl<'a> FilterEditState<'a> {
    /// Clear the editing text
    pub fn clear(&mut self) {
        self.text_area = new_text_area("");
    }

    /// Attempt to parse a [`Directive`]
    ///
    /// Returns the directive if it is valid
    ///
    /// Returns `None` and stores a parse error if it is invalid
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

    /// Replace the text area content
    pub fn replace(&mut self, text: String) {
        self.text_area = new_text_area(text);
    }

    /// Check the text area for a parse error
    pub fn validate(&self) -> Option<filter::ParseError> {
        let text = self.text_area.lines().join("");

        text.parse::<Directive>().err()
    }

    /// Forward a key event to the text area
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
