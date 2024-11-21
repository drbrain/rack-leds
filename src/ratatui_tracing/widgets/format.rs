use ratatui::{
    prelude::*,
    widgets::{Block, Clear, HighlightSpacing, Row, StatefulWidget, Table},
};

use crate::ratatui_tracing::widgets::FormatState;

pub struct Format<'a> {
    block: Option<Block<'a>>,
    cell_highlight_style: Style,
    header_style: Style,
    highlight_symbol: String,
    row_highlight_style: Style,
    table_style: Style,
}

impl<'a> Format<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);

        self
    }

    fn table(&'a self, rows: Vec<Row<'a>>) -> Table<'a> {
        let widths = Constraint::from_fills([1, 1]);

        let header = Row::new(vec![
            Text::from("Setting").alignment(Alignment::Center),
            Text::from("Display").alignment(Alignment::Center),
        ])
        .style(self.header_style)
        .bottom_margin(1);

        let table = Table::new(rows, widths);

        let table = if let Some(block) = &self.block {
            table.block(block.clone())
        } else {
            table
        };

        table
            .column_spacing(1)
            .header(header)
            .highlight_symbol(self.highlight_symbol.clone())
            .highlight_spacing(HighlightSpacing::Always)
            .row_highlight_style(self.row_highlight_style)
            .cell_highlight_style(self.cell_highlight_style)
            .style(self.table_style)
    }
}

impl<'a> Default for Format<'a> {
    fn default() -> Self {
        let header_style = Style::default().bold();
        let cell_highlight_style = Style::default().bold().fg(Color::Black).bg(Color::Gray);
        let highlight_symbol = "❯".into();
        let row_highlight_style = Style::default().bold();
        let table_style = Style::default().bg(Color::Reset);

        Self {
            block: None,
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

        let rows = self.rows(state.as_rows());
        let table = self.table(rows);

        StatefulWidget::render(table, area, buf, &mut state.table);
    }
}
