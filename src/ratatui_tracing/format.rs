mod inner;
mod scope;
mod time_format;

use std::sync::{Arc, Mutex, RwLock};

pub use inner::FormatInner;
use ratatui::{
    layout::{Alignment, Constraint},
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Clear, HighlightSpacing, Padding, Row, StatefulWidget, Table,
        TableState, Widget,
    },
};
pub use scope::ScopeDisplay;
use time_format::TimeFormat;

#[derive(Clone)]
pub struct Format {
    inner: Arc<RwLock<FormatInner>>,
    state: Arc<Mutex<TableState>>,
}

impl Format {
    pub fn read(&self) -> FormatInner {
        let guard = self.inner.read().unwrap();

        guard.clone()
    }

    pub fn row_last(&self) {
        let mut guard = self.state.lock().unwrap();

        guard.select_last()
    }

    pub fn row_edit(&self) {
        let selected = {
            let guard = self.state.lock().unwrap();

            guard.selected()
        };

        let Some(selected) = selected else {
            return;
        };

        let mut format = self.inner.write().unwrap();

        match selected {
            0 => {
                format.time = format.time.next();
            }
            1 => {
                format.display_level = !format.display_level;
            }
            2 => {
                format.display_scope = format.display_scope.next();
            }
            3 => {
                format.display_scope_fields = !format.display_scope_fields;
            }
            4 => {
                format.display_target = !format.display_target;
            }
            _ => (),
        }
    }

    pub fn row_first(&self) {
        let mut guard = self.state.lock().unwrap();

        guard.select_first()
    }

    pub fn row_next(&self) {
        let mut guard = self.state.lock().unwrap();

        guard.select_next()
    }

    pub fn row_previous(&self) {
        let mut guard = self.state.lock().unwrap();

        guard.select_previous()
    }
}

impl Default for Format {
    fn default() -> Self {
        let state = TableState::new().with_selected_cell((0, 1));

        Self {
            inner: Default::default(),
            state: Arc::new(Mutex::new(state)),
        }
    }
}

impl Widget for &Format {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Clear.render(area, buf);

        let format = self.read();

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from("Format").bold())
            .title_bottom(Line::from("Esc to dismiss").right_aligned().italic())
            .padding(Padding::symmetric(1, 0));

        let rows = format.as_rows();
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

        let mut state = self.state.lock().unwrap();

        StatefulWidget::render(table, area, buf, &mut state);
    }
}
