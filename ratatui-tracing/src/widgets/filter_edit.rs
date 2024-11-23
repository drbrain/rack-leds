use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, Padding, Widget},
};

use crate::widgets::FilterEditState;

/// A widget for creating or editing a filter [`tracing_subscriber::filter::Directive`]
pub struct FilterEdit<'a> {
    block: Option<Block<'a>>,
    error_text_style: Style,
    input_line_error_style: Style,
    input_line_ok_style: Style,
}

impl<'a> FilterEdit<'a> {
    /// Wrap the edit view with a [`Block`] widget
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);

        self
    }

    /// Set the style for the text of a directive parse error
    pub fn error_text_style(mut self, style: impl Into<Style>) -> Self {
        self.error_text_style = style.into();

        self
    }

    /// Set the input line style when the input is invalid
    pub fn input_line_error_style(mut self, style: impl Into<Style>) -> Self {
        self.input_line_error_style = style.into();

        self
    }

    /// Set the input line style when the input is valid
    pub fn input_line_ok_style(mut self, style: impl Into<Style>) -> Self {
        self.input_line_ok_style = style.into();

        self
    }

    fn input_line_style(&self, is_ok: bool) -> Style {
        if is_ok {
            self.input_line_ok_style
        } else {
            self.input_line_error_style
        }
    }
}

impl<'a> Default for FilterEdit<'a> {
    /// The default `FilterEdit` has a green input line when valid and a red when invalid.  Error
    /// text is red.
    fn default() -> Self {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::symmetric(1, 0));

        let error_text_style = Style::default().fg(Color::Red);
        let input_line_error_style = Style::default().fg(Color::Red);
        let input_line_ok_style = Style::default().fg(Color::Green);

        Self {
            block: Some(block),
            error_text_style,
            input_line_error_style,
            input_line_ok_style,
        }
    }
}

impl<'a> StatefulWidget for &FilterEdit<'a> {
    type State = FilterEditState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        Clear.render(area, buf);

        let area = if let Some(block) = &self.block {
            block.clone().render(area, buf);

            block.inner(area)
        } else {
            area
        };

        let [input_area, error_area, _] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        let parse_error = state.validate();

        let input_border = Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Thick)
            .border_style(self.input_line_style(parse_error.is_none()));

        if let Some(parse_error) = parse_error {
            Line::from(parse_error.to_string())
                .style(self.error_text_style)
                .render(error_area, buf);
        }

        state.text_area.set_block(input_border);

        state.text_area.render(input_area, buf);
    }
}
