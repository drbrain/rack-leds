use colorgrad::{BlendMode, GradientBuilder, LinearGradient};
use itertools::{
    Itertools,
    MinMaxResult::{MinMax, NoElements, OneElement},
};
use ratatui::{
    prelude::{Buffer, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Context},
        Block, Widget,
    },
};

use crate::collector::{Switch, Update};

pub struct Display<'a> {
    updates: &'a Vec<Update>,
}

impl<'a> Display<'a> {
    pub fn new(updates: &'a Vec<Update>) -> Self {
        Self { updates }
    }

    fn green(&self, values: &[u64]) -> LinearGradient {
        let dark = colorgrad::Color::from_hsla(150.0, 1.0, 0.12, 1.0);
        let light = colorgrad::Color::from_hsla(150.0, 1.0, 0.5, 1.0);

        GradientBuilder::new()
            .colors(&[dark, light])
            .domain(&domain(values))
            .mode(BlendMode::Rgb)
            .build::<LinearGradient>()
            .unwrap()
    }

    fn blue(&self, values: &[u64]) -> LinearGradient {
        let dark = colorgrad::Color::from_hsla(210.0, 1.0, 0.12, 1.0);
        let light = colorgrad::Color::from_hsla(210.0, 1.0, 0.5, 1.0);

        GradientBuilder::new()
            .colors(&[dark, light])
            .domain(&domain(values))
            .mode(BlendMode::Rgb)
            .build::<LinearGradient>()
            .unwrap()
    }

    fn paint_switch(&self, switch: &Switch, context: &mut Context) {
        let recv_gradient = self.blue(switch.receive());
        let tmit_gradient = self.green(switch.transmit());

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

fn domain(values: &[u64]) -> Vec<f32> {
    match values.iter().filter(|v| **v != 0).minmax() {
        NoElements => vec![0.0, 1.0],
        OneElement(one) => vec![0.0, *one as f32],
        MinMax(min, max) => vec![*min as f32, *max as f32],
    }
}
