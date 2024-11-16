use std::{marker::PhantomData, ops::DerefMut};

use ratatui::{
    prelude::*,
    widgets::{
        Block, BorderType, Clear, HighlightSpacing, List, ListItem, Padding, StatefulWidget,
    },
};

use crate::ratatui_tracing::widgets::{filter_state::ViewState, FilterEdit, FilterState};

#[derive(Clone, Default)]
pub struct Filter<'a> {
    _data: PhantomData<&'a ()>,
}

impl<'a> StatefulWidget for Filter<'a> {
    type State = FilterState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);

        match state.view_state() {
            ViewState::Add | ViewState::Edit { .. } => {
                let mut guard = state.filter_edit_state.lock().unwrap();

                let state = guard.deref_mut();

                FilterEdit::default().render(area, buf, state);
            }
            ViewState::View => {
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

                let mut state = state.list_state.lock().unwrap();

                StatefulWidget::render(list, area, buf, &mut state);
            }
        }
    }
}
