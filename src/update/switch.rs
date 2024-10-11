use ratatui::{
    style::Color,
    widgets::canvas::{Context, Points},
};

use crate::{ui::Gradient, Layout};

#[derive(Clone, Debug)]
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

    pub fn paint(
        &self,
        context: &mut Context,
        layout: Layout,
        recv_gradient: &Gradient,
        tmit_gradient: &Gradient,
    ) {
        self.receive
            .iter()
            .zip(self.transmit.iter())
            .enumerate()
            .for_each(|(port, (recv, tmit))| {
                let coords = &[layout.coordinate(port)];

                let mixed: palette::Srgb<u8> =
                    (recv_gradient.at(*recv) + tmit_gradient.at(*tmit)).into_format();

                let color = Color::Rgb(mixed.red, mixed.blue, mixed.green);

                context.draw(&Points { coords, color });
            });
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
            len if (len - 2).is_power_of_two() => (len / 2) + 1,
            // even number of ports
            len if len % 2 == 0 => len / 2,
            // odd number of ports
            len => (len / 2) + 1,
        };

        width.try_into().unwrap_or(u16::MAX)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn width() {
        {
            let switch = Switch::new(Vec::from([0; 5]), Vec::from([0; 5]));

            assert_eq!(5, switch.width());
        }

        {
            let switch = Switch::new(Vec::from([0; 8]), Vec::from([0; 8]));

            assert_eq!(8, switch.width());
        }

        {
            let switch = Switch::new(Vec::from([0; 16]), Vec::from([0; 16]));

            assert_eq!(8, switch.width());
        }

        {
            let switch = Switch::new(Vec::from([0; 18]), Vec::from([0; 18]));

            assert_eq!(10, switch.width());
        }
    }
}
