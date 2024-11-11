use colorgrad::{BlendMode, Color, Gradient as _, GradientBuilder, LinearGradient};
use eyre::{Context, Result};
use itertools::{
    Itertools,
    MinMaxResult::{MinMax, NoElements, OneElement},
};
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

    pub fn red(values: &[u64]) -> Result<Self> {
        let dark = Color::from_hsla(0.0, 0.5, 0.12, 1.0);
        let light = Color::from_hsla(0.0, 0.5, 0.5, 1.0);

        Self::new(dark, light, values)
    }

    /// Gradient covering values from 0-100 from green to yellow to red to dark red
    pub fn percent_gyrr() -> Result<Self> {
        let green = Color::from_hsla(120.0, 0.75, 0.4, 1.0);
        let yellow = Color::from_hsla(60.0, 0.75, 0.4, 1.0);
        let red = Color::from_hsla(0.0, 0.75, 0.5, 1.0);
        let dark_red = Color::from_hsla(0.0, 1.0, 0.5, 1.0);

        let domain = vec![0.0, 100.0];
        let inner = GradientBuilder::new()
            .colors(&[green.clone(), yellow.clone(), red.clone(), dark_red.clone()])
            .domain(&domain)
            .mode(BlendMode::Rgb)
            .build::<LinearGradient>()
            .wrap_err("Unable to create percent GYRR gradient")?;

        Ok(Self { inner })
    }

    pub fn white(values: &[u64]) -> Result<Self> {
        let dark = Color::from_hsla(0.0, 0.0, 0.25, 1.0);
        let light = Color::from_hsla(0.0, 0.0, 0.75, 1.0);

        Self::new(dark, light, values)
    }

    fn new(dark: Color, light: Color, values: &[u64]) -> Result<Self> {
        let domain = domain(values);
        let inner = GradientBuilder::new()
            .colors(&[dark.clone(), light.clone()])
            .domain(&domain)
            .mode(BlendMode::Rgb)
            .build::<LinearGradient>()
            .wrap_err(format!(
                "Unable to create gradient for {dark:?} {light:?} {domain:?}"
            ))?;

        Ok(Self { inner })
    }

    /// Look up a color in the gradient domain, use the background color if the value is 0.
    pub fn at(&self, value: u64) -> color_art::Color {
        let color = if value > 0 {
            self.inner.at(value as f32)
        } else {
            Color::new(0.0, 0.0, 0.0, 0.0)
        };

        color_art::Color::from_rgb(255.0 * color.r, 255.0 * color.g, 255.0 * color.b)
            .unwrap_or_else(|e| panic!("impossible invalid color {color:?} ({e:?})"))
    }
}

fn domain(values: &[u64]) -> Vec<f32> {
    match values.iter().filter(|v| **v != 0).minmax() {
        NoElements => vec![0.0, 1.0],
        OneElement(one) => vec![0.0, *one as f32],
        MinMax(min, max) => {
            if min == max {
                vec![0.0, *max as f32]
            } else {
                vec![*min as f32, *max as f32]
            }
        }
    }
}
