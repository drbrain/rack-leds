use ratatui::{
    prelude::{Buffer, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Context},
        Widget,
    },
};

use crate::{
    collector::{Switch, Update},
    ui::Gradient,
};

pub struct Display<'a> {
    update: &'a Update,
}

impl<'a> Display<'a> {
    pub fn new(update: &'a Update) -> Self {
        Self { update }
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
            .x_bounds([0.0, (self.update.width().saturating_sub(1)).into()])
            .y_bounds([0.0, (self.update.height().saturating_sub(1)).into()])
            .marker(Marker::Block)
            .background_color(Color::Black)
            .paint(|context| match self.update {
                Update::Switch(switch) => self.paint_switch(switch, context),
            });

        canvas.render(area, buf);
    }
}
