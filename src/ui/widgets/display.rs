use ratatui::{
    prelude::{Buffer, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Context},
        Widget,
    },
};

use crate::{ui::Gradient, update::Switch, Layout, Update};

pub struct Display<'a> {
    update: &'a Update,
}

impl<'a> Display<'a> {
    pub fn new(update: &'a Update) -> Self {
        Self { update }
    }

    fn paint_switch(&self, switch: &Switch, layout: Layout, context: &mut Context) {
        let recv_gradient = Gradient::blue(switch.receive());
        let tmit_gradient = Gradient::green(switch.transmit());

        switch.paint(context, layout, &recv_gradient, &tmit_gradient);
    }
}

impl<'a> Widget for Display<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let canvas = Canvas::default()
            .x_bounds([0.0, self.update.x_bound()])
            .y_bounds([0.0, self.update.y_bound()])
            .marker(Marker::Block)
            .background_color(Color::Black)
            .paint(|context| match self.update {
                Update::Switch {
                    device: switch,
                    layout,
                } => self.paint_switch(switch, *layout, context),
            });

        canvas.render(area, buf);
    }
}
