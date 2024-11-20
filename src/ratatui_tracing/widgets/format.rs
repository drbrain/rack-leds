use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Clear, HighlightSpacing, Padding, Row, StatefulWidget, Table},
};

use crate::ratatui_tracing::widgets::FormatState;

pub struct Format<'a> {
    block: Block<'a>,
    block_help_style: Style,
    block_title_style: Style,
    cell_highlight_style: Style,
    header_style: Style,
    highlight_symbol: String,
    row_highlight_style: Style,
    table_style: Style,
}

impl<'a> Default for Format<'a> {
    fn default() -> Self {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::symmetric(1, 0));

        let block_help_style = Style::default().italic();
        let block_title_style = Style::default().bold();
        let header_style = Style::default().bold();
        let cell_highlight_style = Style::default().bold().fg(Color::Black).bg(Color::Gray);
        let highlight_symbol = "❯".into();
        let row_highlight_style = Style::default().bold();
        let table_style = Style::default().bg(Color::Reset);

        Self {
            block,
            block_help_style,
            block_title_style,
            cell_highlight_style,
            header_style,
            highlight_symbol,
            row_highlight_style,
            table_style,
        }
    }
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

        let block = self
            .block
            .clone()
            .title(Line::from("Format").style(self.block_title_style))
            .title_bottom(
                Line::from("Esc to dismiss")
                    .right_aligned()
                    .style(self.block_help_style),
            );

        let rows = self.rows(state.as_rows());
        let widths = Constraint::from_fills([1, 1]);

        let header = Row::new(vec![
            Text::from("Setting").alignment(Alignment::Center),
            Text::from("Display").alignment(Alignment::Center),
        ])
        .style(self.header_style)
        .bottom_margin(1);

        let table = Table::new(rows, widths)
            .block(block)
            .column_spacing(1)
            .header(header)
            .highlight_symbol(self.highlight_symbol.clone())
            .highlight_spacing(HighlightSpacing::Always)
            .row_highlight_style(self.row_highlight_style)
            .cell_highlight_style(self.cell_highlight_style)
            .style(self.table_style);

        StatefulWidget::render(table, area, buf, &mut state.table);
    }
}
