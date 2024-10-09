use ratatui::{
    prelude::{Buffer, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Context},
        Block, Widget,
    },
};

use crate::{
    collector::{Switch, Update},
    ui::Gradient,
};

pub struct Display<'a> {
    updates: &'a Vec<Update>,
}

impl<'a> Display<'a> {
    pub fn new(updates: &'a Vec<Update>) -> Self {
        Self { updates }
    }

    fn paint_switch(&self, switch: &Switch, context: &mut Context) {
        let recv_gradient = Gradient::blue(switch.receive());
        let tmit_gradient = Gradient::green(switch.transmit());

        switch.paint(context, &recv_gradient, &tmit_gradient);
    }
}

impl<'a> Widget for Display<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let canvas = Canvas::default()
            .block(Block::bordered().title("Display"))
            .x_bounds([0.0, 53.0])
            .y_bounds([0.0, 11.0])
            .marker(Marker::Block)
            .background_color(Color::Black)
            .paint(|context| {
                for update in self.updates {
                    match update {
                        Update::Switch(switch) => self.paint_switch(switch, context),
                    }
                }
            });

        canvas.render(area, buf);
    }
}
