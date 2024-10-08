use colorgrad::{BlendMode, Gradient, GradientBuilder, LinearGradient};
use itertools::{
    Itertools,
    MinMaxResult::{MinMax, NoElements, OneElement},
};
use ratatui::{
    prelude::{Buffer, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Context, Points},
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
        let dark = colorgrad::Color::from_hsla(150.0, 1.0, 0.1, 1.0);
        let light = colorgrad::Color::from_hsla(150.0, 1.0, 0.5, 1.0);

        GradientBuilder::new()
            .colors(&[dark, light])
            .domain(&domain(values))
            .mode(BlendMode::Rgb)
            .build::<LinearGradient>()
            .unwrap()
    }

    fn blue(&self, values: &[u64]) -> LinearGradient {
        let dark = colorgrad::Color::from_hsla(210.0, 1.0, 0.1, 1.0);
        let light = colorgrad::Color::from_hsla(210.0, 1.0, 0.5, 1.0);

        GradientBuilder::new()
            .colors(&[dark, light])
            .domain(&domain(values))
            .mode(BlendMode::Rgb)
            .build::<LinearGradient>()
            .unwrap()
    }

    fn paint_switch(&self, switch: &Switch, context: &mut Context) {
        let blue_gradient = self.blue(switch.receive());
        let green_gradient = self.green(switch.transmit());

        let ports: Vec<_> = switch
            .receive()
            .iter()
            .zip(switch.transmit().iter())
            .map(|(recv, tmit)| {
                let blue = if *recv > 0 {
                    blue_gradient.at(*recv as f32)
                } else {
                    colorgrad::Color::new(0.0, 0.0, 0.0, 0.0)
                };
                let blue = palette::Srgb::new(blue.r, blue.g, blue.b);

                let green = if *tmit > 0 {
                    green_gradient.at(*tmit as f32)
                } else {
                    colorgrad::Color::new(0.0, 0.0, 0.0, 0.0)
                };
                let green = palette::Srgb::new(green.r, green.g, green.b);

                let mixed: palette::Srgb<u8> = (blue + green).into_format();

                Color::Rgb(mixed.red, mixed.blue, mixed.green)
            })
            .collect();

        for (port, color) in ports.iter().enumerate() {
            let col = if port < 16 { port / 2 } else { (port / 2) + 1 };
            let row = if port % 2 == 0 { 9.0 } else { 10.0 };

            context.draw(&Points {
                coords: &[(col as f64, row)],
                color: *color,
            });
        }
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
