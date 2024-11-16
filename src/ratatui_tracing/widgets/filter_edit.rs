use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, Padding, Widget},
};

use crate::ratatui_tracing::widgets::FilterEditState;

pub struct FilterEdit<'a> {
    block: Block<'a>,
    block_help_style: Style,
    block_title_style: Style,
    error_text_style: Style,
    input_line_error_style: Style,
    input_line_ok_style: Style,
}

impl<'a> FilterEdit<'a> {
    fn input_line_style(&self, is_ok: bool) -> Style {
        if is_ok {
            self.input_line_ok_style
        } else {
            self.input_line_error_style
        }
    }
}

impl<'a> Default for FilterEdit<'a> {
    fn default() -> Self {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::symmetric(1, 0));

        let block_title_style = Style::default().bold();
        let block_help_style = Style::default().italic();
        let error_text_style = Style::default().fg(Color::Red);
        let input_line_error_style = Style::default().fg(Color::Red);
        let input_line_ok_style = Style::default().fg(Color::Green);

        Self {
            block,
            block_help_style,
            block_title_style,
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

        let block = self
            .block
            .clone()
            .title(Line::from("Filters — Add").style(self.block_title_style))
            .title_bottom(Line::from("Esc to dismiss").style(self.block_help_style));

        let [input_area, error_area, _] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(block.inner(area));

        block.render(area, buf);

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
