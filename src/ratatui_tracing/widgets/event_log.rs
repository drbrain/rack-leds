use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Paragraph, StatefulWidget, Wrap},
};

use crate::ratatui_tracing::widgets::EventLogState;

pub struct EventLog<'a> {
    block: Block<'a>,
    title: String,
}

impl<'a> Default for EventLog<'a> {
    fn default() -> Self {
        let block = Block::bordered().border_type(BorderType::Rounded);
        let title = "Log".to_string();

        Self { block, title }
    }
}

impl<'a> StatefulWidget for EventLog<'a> {
    type State = EventLogState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let epoch = state.epoch();

        let block = self.block.clone().title(self.title);
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
