use ratatui::{
    prelude::*,
    widgets::{Block, StatefulWidget},
};

use crate::widgets::DetailState;

/// Widget to display details of an [`crate::Event`]
#[derive(Default)]
pub struct Detail<'a> {
    block: Option<Block<'a>>,
}

impl<'a> Detail<'a> {
    /// Wrap the event detail with a [`Block`] widget
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);

        self
    }
}

impl<'a> StatefulWidget for Detail<'a> {
    type State = DetailState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self
            .block
            .map(|block| {
                let log_area = block.inner(area);
                block.render(area, buf);

                log_area
            })
            .unwrap_or(area);

        state
            .event
            .to_pretty(state.epoch, &state.format)
            .render(area, buf);
    }
}
