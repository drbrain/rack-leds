use colorgrad::{BlendMode, Color, Gradient as _, GradientBuilder, LinearGradient};
use eyre::{Context, Result};
use itertools::{
    Itertools,
    MinMaxResult::{MinMax, NoElements, OneElement},
};
use palette::Srgb;

pub struct Gradient {
    inner: LinearGradient,
}

impl Gradient {
    pub fn blue(values: &[u64]) -> Result<Self> {
        let dark = Color::from_hsla(210.0, 1.0, 0.12, 1.0);
        let light = Color::from_hsla(210.0, 1.0, 0.5, 1.0);

        Self::new(dark, light, values)
    }

    pub fn green(values: &[u64]) -> Result<Self> {
        let dark = Color::from_hsla(150.0, 1.0, 0.12, 1.0);
        let light = Color::from_hsla(150.0, 1.0, 0.5, 1.0);

        Self::new(dark, light, values)
    }

    fn new(dark: Color, light: Color, values: &[u64]) -> Result<Self> {
        let inner = GradientBuilder::new()
            .colors(&[dark.clone(), light.clone()])
            .domain(&domain(values))
            .mode(BlendMode::Rgb)
            .build::<LinearGradient>()
            .wrap_err(format!(
                "Unable to create gradient for {dark:?} {light:?} {values:?}"
            ))?;

        Ok(Self { inner })
    }

    /// Look up a color in the gradient domain, use the background color if the value is 0.
    pub fn at(&self, value: u64) -> Srgb {
        let color = if value > 0 {
            self.inner.at(value as f32)
        } else {
            Color::new(0.0, 0.0, 0.0, 0.0)
        };

        Srgb::new(color.r, color.g, color.b)
    }
}

fn domain(values: &[u64]) -> Vec<f32> {
    match values.iter().filter(|v| **v != 0).minmax() {
        NoElements => vec![0.0, 1.0],
        OneElement(one) => vec![0.0, *one as f32],
        MinMax(min, max) => vec![*min as f32, *max as f32],
    }
}
