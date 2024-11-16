use std::marker::PhantomData;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Wrap},
};

use crate::ratatui_tracing::widgets::EventLogState;

#[derive(Default)]
pub struct EventLog<'a> {
    _data: PhantomData<&'a ()>,
}

impl<'a> StatefulWidget for EventLog<'a> {
    type State = EventLogState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let epoch = state.epoch();

        let block = Block::new().title("Log").borders(Borders::ALL);
        let block_inner = block.inner(area);

        let text: Vec<Line> = state
            .log
            .iter()
            .map(|line| line.to_line(epoch, &state.format))
            .collect();
        let text = Text::from(text);

        let text = Paragraph::new(text).block(block).wrap(Wrap { trim: false });

        // NOTE: Scrolling is hard https://github.com/ratatui/ratatui/issues/174
        // and Lists don't allow wrapping https://github.com/ratatui/ratatui/issues/128
        let line_offset = text
            .line_count(block_inner.width)
            .saturating_sub(block_inner.height.into())
            .try_into()
            .unwrap_or(0);

        let text = text.scroll((line_offset, 0));

        text.render(area, buf)
    }
}
