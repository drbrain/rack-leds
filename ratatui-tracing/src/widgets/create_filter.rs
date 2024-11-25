use ratatui::{
    prelude::*,
    widgets::{Block, StatefulWidget},
};
use style::Styled;

use crate::widgets::CreateFilterState;

pub struct CreateFilter<'a> {
    block: Option<Block<'a>>,
    highlight_style: Style,
}

impl<'a> CreateFilter<'a> {
    /// Wrap the format options with a [`Block`] widget
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);

        self
    }

    /// Set the highlight style for the selected field
    pub fn highlight_style(mut self, style: impl Into<Style>) -> Self {
        self.highlight_style = style.into();

        self
    }

    fn render_directive(&self, area: Rect, buf: &mut Buffer, state: &mut CreateFilterState) {
        let directive = if let Some(directive) = state.directive() {
            Line::from(directive.to_string()).green()
        } else {
            Line::from("invalid directive (bug)").red()
        };

        directive.render(area, buf);
    }

    fn render_level(&self, area: Rect, buf: &mut Buffer, state: &mut CreateFilterState) {
        let [field, value] = Layout::horizontal(Constraint::from_fills([1, 1])).areas(area);

        Line::from("Level").bold().render(field, buf);

        let level: &'static str = state.level.into();
        let level = Line::from(level);

        let level = if state.level_selected() {
            level.set_style(self.highlight_style)
        } else {
            level
        };

        level.render(value, buf);
    }

    fn render_target(&self, area: Rect, buf: &mut Buffer, state: &mut CreateFilterState) {
        let [field, value] = Layout::horizontal(Constraint::from_fills([1, 1])).areas(area);

        Line::from(state.event.target.clone())
            .bold()
            .render(field, buf);

        let target = Line::from(state.target.to_string());

        let target = if state.target_selected() {
            target.set_style(self.highlight_style)
        } else {
            target
        };

        target.render(value, buf);
    }
}

impl<'a> Default for CreateFilter<'a> {
    fn default() -> Self {
        let highlight_style = Style::default().fg(Color::Black).bg(Color::DarkGray);

        Self {
            block: None,
            highlight_style,
        }
    }
}

impl<'a> StatefulWidget for CreateFilter<'a> {
    type State = CreateFilterState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = if let Some(block) = &self.block {
            let log_area = block.inner(area);
            block.render(area, buf);

            log_area
        } else {
            area
        };

        let [target_area, level_area, _, directive_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        self.render_target(target_area, buf, state);
        self.render_level(level_area, buf, state);
        self.render_directive(directive_area, buf, state);
    }
}
