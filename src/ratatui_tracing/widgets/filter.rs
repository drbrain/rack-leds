use ratatui::{
    prelude::*,
    widgets::{
        Block, BorderType, Clear, HighlightSpacing, List, ListDirection, ListItem, Padding,
        StatefulWidget,
    },
};

use crate::ratatui_tracing::widgets::{FilterEdit, FilterState};

pub struct Filter<'a> {
    block: Block<'a>,
    block_help_style: Style,
    block_title_style: Style,
    list_highlight_style: Style,
    list_highlight_symbol: String,
}

impl<'a> Filter<'a> {}

impl<'a> Default for Filter<'a> {
    fn default() -> Self {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::symmetric(1, 0));

        let block_title_style = Style::default().bold();
        let block_help_style = Style::default().italic();
        let list_highlight_style = Style::default().bold().fg(Color::Black).bg(Color::Gray);
        let list_highlight_symbol = "❯".into();

        Self {
            block,
            block_help_style,
            block_title_style,
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

            let block = self
                .block
                .title(Line::from("Filters").style(self.block_title_style))
                .title_bottom(
                    Line::from("Esc to dismiss")
                        .right_aligned()
                        .style(self.block_help_style),
                );

            let list = List::new(items)
                .block(block)
                .highlight_symbol(&self.list_highlight_symbol)
                .highlight_spacing(HighlightSpacing::Always)
                .highlight_style(self.list_highlight_style)
                .direction(ListDirection::TopToBottom);

            StatefulWidget::render(list, area, buf, &mut state.list_state);
        }
    }
}
