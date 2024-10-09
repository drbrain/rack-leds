use colorgrad::{BlendMode, Color, Gradient as _, GradientBuilder, LinearGradient};
use itertools::{
    Itertools,
    MinMaxResult::{MinMax, NoElements, OneElement},
};
use palette::Srgb;

pub struct Gradient {
    inner: LinearGradient,
}

impl Gradient {
    pub fn blue(values: &[u64]) -> Self {
        let dark = Color::from_hsla(210.0, 1.0, 0.12, 1.0);
        let light = Color::from_hsla(210.0, 1.0, 0.5, 1.0);

        Self::new(dark, light, values)
    }

    pub fn green(values: &[u64]) -> Self {
        let dark = Color::from_hsla(150.0, 1.0, 0.12, 1.0);
        let light = Color::from_hsla(150.0, 1.0, 0.5, 1.0);

        Self::new(dark, light, values)
    }

    fn new(dark: Color, light: Color, values: &[u64]) -> Self {
        let inner = GradientBuilder::new()
            .colors(&[dark, light])
            .domain(&domain(values))
            .mode(BlendMode::Rgb)
            .build::<LinearGradient>()
            .unwrap();

        Self { inner }
    }

    pub fn at(&self, value: f32) -> Srgb<u8> {
        let color = if value > 0.0 {
            self.inner.at(value)
        } else {
            Color::new(0.0, 0.0, 0.0, 0.0)
        };

        Srgb::new(color.r, color.g, color.b).into_format()
    }
}

fn domain(values: &[u64]) -> Vec<f32> {
    match values.iter().filter(|v| **v != 0).minmax() {
        NoElements => vec![0.0, 1.0],
        OneElement(one) => vec![0.0, *one as f32],
        MinMax(min, max) => vec![*min as f32, *max as f32],
    }
}
