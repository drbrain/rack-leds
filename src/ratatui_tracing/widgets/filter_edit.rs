use std::marker::PhantomData;

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, Padding, Widget},
};

use crate::ratatui_tracing::widgets::FilterEditState;

#[derive(Clone, Default)]
pub struct FilterEdit<'a> {
    _data: PhantomData<&'a ()>,
}

impl<'a> StatefulWidget for FilterEdit<'a> {
    type State = FilterEditState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
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

        let parse_error = state.validate();
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

        state.text_area.set_block(input_border);

        state.text_area.render(input_area, buf);
    }
}
