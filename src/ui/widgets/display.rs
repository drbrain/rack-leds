use ratatui::{
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Points},
        Block, Widget,
    },
};

pub struct Display {}

impl Display {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Display {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let canvas = Canvas::default()
            .block(Block::bordered().title("Display"))
            .x_bounds([0.0, 53.0])
            .y_bounds([0.0, 11.0])
            .marker(Marker::Block)
            .background_color(Color::Black)
            .paint(|ctx| {
                ctx.draw(&Points {
                    coords: &[(0.0, 0.0)],
                    color: Color::Red,
                });
            });

        canvas.render(area, buf);
    }
}
