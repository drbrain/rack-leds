use ratatui::{
    prelude::*,
    widgets::{Block, Clear, HighlightSpacing, List, ListDirection, ListItem, StatefulWidget},
};

use crate::ratatui_tracing::widgets::{FilterEdit, FilterState};

pub struct Filter<'a> {
    block: Option<Block<'a>>,
    list_highlight_style: Style,
    list_highlight_symbol: String,
}

impl<'a> Filter<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);

        self
    }

    fn list(&'a self, items: Vec<ListItem<'a>>) -> List<'a> {
        let list = List::new(items);

        let list = if let Some(ref block) = self.block {
            list.block(block.clone())
        } else {
            list
        };

        list.highlight_symbol(&self.list_highlight_symbol)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_style(self.list_highlight_style)
            .direction(ListDirection::TopToBottom)
    }
}

impl<'a> Default for Filter<'a> {
    fn default() -> Self {
        let list_highlight_style = Style::default().bold().fg(Color::Black).bg(Color::Gray);
        let list_highlight_symbol = "❯".into();

        Self {
            block: None,
            list_highlight_style,
            list_highlight_symbol,
        }
    }
}

impl<'a> StatefulWidget for Filter<'a> {
    type State = FilterState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);

        if state.is_editing() {
            let state = &mut state.filter_edit_state;

            FilterEdit::default().render(area, buf, state);
        } else {
            let items: Vec<_> = state
                .reloadable
                .directives()
                .iter()
                .map(|directive| ListItem::new(directive.to_string()))
                .collect();

            StatefulWidget::render(self.list(items), area, buf, &mut state.list_state);
        }
    }
}
