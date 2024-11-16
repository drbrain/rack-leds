use std::marker::PhantomData;

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Clear, HighlightSpacing, Padding, Row, StatefulWidget, Table},
};

use crate::ratatui_tracing::widgets::FormatState;

#[derive(Default)]
pub struct Format<'a> {
    _data: PhantomData<&'a ()>,
}

impl<'a> Format<'a> {
    fn rows(&self, rows: Vec<(&'static str, &'static str)>) -> Vec<Row> {
        rows.into_iter()
            .map(|(name, value)| {
                Row::new(vec![
                    Text::from(name).alignment(Alignment::Left),
                    Text::from(value).alignment(Alignment::Right),
                ])
            })
            .collect()
    }
}

impl<'a> StatefulWidget for Format<'a> {
    type State = FormatState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from("Format").bold())
            .title_bottom(Line::from("Esc to dismiss").right_aligned().italic())
            .padding(Padding::symmetric(1, 0));

        let rows = self.rows(state.as_rows());
        let widths = Constraint::from_fills([1, 1]);

        let header = Row::new(vec![
            Text::from("Setting").alignment(Alignment::Center),
            Text::from("Display").alignment(Alignment::Center),
        ])
        .style(Style::default().bold())
        .bottom_margin(1);

        let table = Table::new(rows, widths)
            .block(block)
            .column_spacing(1)
            .header(header)
            .highlight_symbol("❯")
            .highlight_spacing(HighlightSpacing::Always)
            .row_highlight_style(Style::default().bold())
            .cell_highlight_style(Style::default().fg(Color::Black).bg(Color::Gray))
            .style(Style::default().bg(Color::Black));

        StatefulWidget::render(table, area, buf, &mut state.table);
    }
}
