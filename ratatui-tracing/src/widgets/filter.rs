use ratatui::{
    prelude::*,
    widgets::{Block, Clear, HighlightSpacing, List, ListDirection, ListItem, StatefulWidget},
};

use crate::widgets::{FilterEdit, FilterState};

/// Widget to display and edit a tracing filter
///
/// [`Filter`] is a stateful widget that uses [`FilterState`] for displaying and editing the filter
/// directives.
pub struct Filter<'a> {
    block: Option<Block<'a>>,
    highlight_style: Style,
    highlight_symbol: String,
}

impl<'a> Filter<'a> {
    /// Wrap the log with a [`Block`] widget
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);

        self
    }

    /// Set the highlight style for the selected filter directive
    pub fn highlight_style(mut self, highlight_style: impl Into<Style>) -> Self {
        self.highlight_style = highlight_style.into();

        self
    }

    /// Set the symbol to display in front of the selected directive
    pub fn highlight_symbol(mut self, highlight_symbol: impl ToString) -> Self {
        self.highlight_symbol = highlight_symbol.to_string();

        self
    }

    fn list(&'a self, items: Vec<ListItem<'a>>) -> List<'a> {
        let list = List::new(items);

        let list = if let Some(ref block) = self.block {
            list.block(block.clone())
        } else {
            list
        };

        list.highlight_symbol(&self.highlight_symbol)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_style(self.highlight_style)
            .direction(ListDirection::TopToBottom)
    }
}

impl<'a> Default for Filter<'a> {
    /// The default `Filter` uses a [`Color::Black`] on [`Color::Gray`] highlight style and the
    /// `"❯"` highlight symbol
    fn default() -> Self {
        let highlight_stely = Style::default().bold().fg(Color::Black).bg(Color::Gray);
        let highlight_symbol = "❯".into();

        Self {
            block: None,
            highlight_style: highlight_stely,
            highlight_symbol,
        }
    }
}

impl<'a> StatefulWidget for Filter<'a> {
    type State = FilterState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);

        if state.is_viewing() {
            let items: Vec<_> = state
                .reloadable
                .directives()
                .iter()
                .map(|directive| ListItem::new(directive.to_string()))
                .collect();

            StatefulWidget::render(self.list(items), area, buf, &mut state.list_state);
        } else {
            let state = &mut state.filter_edit_state;

            self.block
                .map(|block| FilterEdit::default().block(block))
                .unwrap_or_default()
                .render(area, buf, state);
        }
    }
}
