use std::marker::PhantomData;

use ratatui::{
    prelude::*,
    widgets::{
        Block, BorderType, Clear, HighlightSpacing, List, ListItem, Padding, StatefulWidget,
    },
};

use crate::ratatui_tracing::widgets::{FilterEdit, FilterState};

#[derive(Clone, Default)]
pub struct Filter<'a> {
    _data: PhantomData<&'a ()>,
}

impl<'a> StatefulWidget for Filter<'a> {
    type State = FilterState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);

        if state.is_editing() {
            let state = &mut state.filter_edit_state;

            FilterEdit::default().render(area, buf, state);
        } else {
            let dialog_border = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(Line::from("Filters").bold())
                .title_bottom(Line::from("Esc to dismiss").right_aligned().italic())
                .padding(Padding::symmetric(1, 0));

            let items: Vec<_> = state
                .reloadable
                .directives()
                .iter()
                .map(|directive| ListItem::new(directive.to_string()))
                .collect();

            let list = List::new(items)
                .block(dialog_border)
                .highlight_symbol("❯")
                .highlight_spacing(HighlightSpacing::Always)
                .highlight_style(Style::default().bold().fg(Color::Black).bg(Color::Gray))
                .direction(ratatui::widgets::ListDirection::TopToBottom);

            StatefulWidget::render(list, area, buf, &mut state.list_state);
        }
    }
}
