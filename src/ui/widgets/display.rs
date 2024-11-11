use eyre::Result;
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
    ui::Gradient,
    update::{AccessPoint, Switch},
    Layout, Update,
};

pub struct Display<'a> {
    update: &'a Update,
}

impl<'a> Display<'a> {
    pub fn new(update: &'a Update) -> Self {
        Self { update }
    }

    fn paint_access_point(
        &self,
        access_point: &AccessPoint,
        layout: Layout,
        context: &mut Context<'_>,
    ) -> Result<()> {
        let recv_gradient = Gradient::blue(&access_point.receive())?;
        let tmit_gradient = Gradient::green(&access_point.transmit())?;

        let util_gradient = Gradient::percent_gyrr()?;
        let stations_gradient = Gradient::white(&access_point.stations())?;

        access_point.paint(
            context,
            layout,
            &recv_gradient,
            &tmit_gradient,
            &util_gradient,
            &stations_gradient,
        );

        Ok(())
    }

    fn paint_switch(&self, switch: &Switch, layout: Layout, context: &mut Context) -> Result<()> {
        let recv_gradient = Gradient::blue(switch.receive())?;
        let tmit_gradient = Gradient::green(switch.transmit())?;
        let poe_gradient = Gradient::red(switch.poe())?;

        switch.paint(
            context,
            layout,
            &recv_gradient,
            &tmit_gradient,
            &poe_gradient,
        );

        Ok(())
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
                Update::AccessPoint { device, layout, .. } => {
                    self.paint_access_point(device, *layout, context).unwrap();
                }
                Update::Switch {
                    device: switch,
                    layout,
                    ..
                } => self.paint_switch(switch, *layout, context).unwrap(),
            });

        canvas.render(area, buf);
    }
}
