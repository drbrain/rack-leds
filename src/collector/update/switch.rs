use ratatui::{
    style::Color,
    widgets::canvas::{Context, Points},
};

use crate::ui::Gradient;

#[derive(Clone)]
pub struct Switch {
    receive: Vec<u64>,
    transmit: Vec<u64>,
}

impl Switch {
    pub fn empty() -> Self {
        Self {
            receive: vec![],
            transmit: vec![],
        }
    }

    pub fn new(receive: Vec<u64>, transmit: Vec<u64>) -> Self {
        Self { receive, transmit }
    }

    pub fn height(&self) -> u16 {
        if self.receive.len() > 8 {
            2
        } else {
            1
        }
    }

    pub fn paint(&self, context: &mut Context, recv_gradient: &Gradient, tmit_gradient: &Gradient) {
        let ports: Vec<_> = self
            .receive
            .iter()
            .zip(self.transmit.iter())
            .map(|(recv, tmit)| {
                let mixed: palette::Srgb<u8> =
                    (recv_gradient.at(*recv) + tmit_gradient.at(*tmit)).into_format();

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

    pub fn receive(&self) -> &Vec<u64> {
        &self.receive
    }

    pub fn transmit(&self) -> &Vec<u64> {
        &self.transmit
    }

    pub fn width(&self) -> u16 {
        let width = match self.receive.len() {
            // 8 ports or less
            len if len <= 8 => len,
            // 2^n ports + 2 SFP
            len if (len - 2).is_power_of_two() => (len / 2) + 2,
            // even number of ports
            len if len % 2 == 0 => len / 2,
            // odd number of ports
            len => (len / 2) + 1,
        };

        width.try_into().unwrap_or(u16::MAX)
    }
}
