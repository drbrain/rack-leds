use ratatui::{
    prelude::*,
    widgets::{Block, Clear, HighlightSpacing, Row, StatefulWidget, Table},
};

use crate::widgets::FormatState;

/// Widget to configure [`super::EventLog`] format options
pub struct Format<'a> {
    block: Option<Block<'a>>,
    cell_highlight_style: Style,
    header_style: Style,
    highlight_symbol: String,
    row_highlight_style: Style,
    table_style: Style,
}

impl<'a> Format<'a> {
    /// Wrap the format options with a [`Block`] widget
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);

        self
    }

    /// Set the highlight style for the selected format cell
    pub fn cell_highlight_style(mut self, style: impl Into<Style>) -> Self {
        self.cell_highlight_style = style.into();

        self
    }

    /// Set the table header style
    pub fn header_style(mut self, style: impl Into<Style>) -> Self {
        self.header_style = style.into();

        self
    }

    /// Set the selected format row symbol
    pub fn highlight_symbol(mut self, symbol: impl ToString) -> Self {
        self.highlight_symbol = symbol.to_string();

        self
    }

    /// Set the selected table row highlight style
    pub fn row_highlight_style(mut self, style: impl Into<Style>) -> Self {
        self.row_highlight_style = style.into();

        self
    }

    /// Set the table style
    pub fn table_style(mut self, style: impl Into<Style>) -> Self {
        self.table_style = style.into();

        self
    }

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
    /// The default `Format` uses the following styles:
    /// * The selected cell has [`Color::Black`] text on a [`Color::Gray`] background
    /// * The highlight symbol is `"❯"`
    /// * The selected row has bold text
    /// * The table uses the terminal default background color
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

impl<'a> StatefulWidget for Format<'a> {
    type State = FormatState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);

        let rows = self.rows(state.as_rows());
        let table = self.table(rows);

        StatefulWidget::render(table, area, buf, &mut state.table);
    }
}
