use color_art::BlendMode;
use itertools::multizip;
use ratatui::{
    style::Color,
    widgets::canvas::{Context, Points},
};

use crate::ui::Gradient;

#[derive(Clone, Debug)]
pub struct AccessPoint {
    channel_utilization_24_ghz: u64,
    channel_utilization_5_ghz: u64,
    receive_ap: u64,
    receive_wan_24_ghz: u64,
    receive_wan_5_ghz: u64,
    stations_24_ghz: u64,
    stations_5_ghz: u64,
    transmit_ap: u64,
    transmit_wan_24_ghz: u64,
    transmit_wan_5_ghz: u64,
}

impl AccessPoint {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        channel_utilization_24_ghz: u64,
        channel_utilization_5_ghz: u64,
        receive_ap: u64,
        receive_wan_24_ghz: u64,
        receive_wan_5_ghz: u64,
        stations_24_ghz: u64,
        stations_5_ghz: u64,
        transmit_ap: u64,
        transmit_wan_24_ghz: u64,
        transmit_wan_5_ghz: u64,
    ) -> Self {
        Self {
            channel_utilization_24_ghz,
            channel_utilization_5_ghz,
            receive_ap,
            receive_wan_24_ghz,
            receive_wan_5_ghz,
            stations_24_ghz,
            stations_5_ghz,
            transmit_ap,
            transmit_wan_24_ghz,
            transmit_wan_5_ghz,
        }
    }

    pub fn channel_utilization(&self) -> Vec<u64> {
        vec![
            self.channel_utilization_24_ghz,
            self.channel_utilization_5_ghz,
        ]
    }

    pub fn paint(
        &self,
        context: &mut Context<'_>,
        layout: crate::Layout,
        recv_gradient: &Gradient,
        tmit_gradient: &Gradient,
        util_gradient: &Gradient,
        stations_gradient: &Gradient,
    ) {
        multizip((self.receive().iter(), self.transmit().iter()))
            .enumerate()
            .for_each(|(index, (recv, tmit))| {
                let coords = &[layout.coordinate(index)];

                let mixed = color_art::blend(
                    &recv_gradient.at(*recv),
                    &tmit_gradient.at(*tmit),
                    BlendMode::Screen,
                );

                let color = Color::Rgb(mixed.red(), mixed.green(), mixed.blue());

                context.draw(&Points { coords, color });
            });

        let offset = self.receive().len();

        self.channel_utilization()
            .iter()
            .enumerate()
            .for_each(|(index, util)| {
                let coords = &[layout.coordinate(index + offset)];
                let color = util_gradient.at(*util);

                let color = Color::Rgb(color.red(), color.green(), color.blue());

                context.draw(&Points { coords, color });
            });

        let offset = offset + self.channel_utilization().len();

        self.stations()
            .iter()
            .enumerate()
            .for_each(|(index, station)| {
                let coords = &[layout.coordinate(index + offset)];
                let color = stations_gradient.at(*station);

                let color = Color::Rgb(color.red(), color.green(), color.blue());

                context.draw(&Points { coords, color });
            });
    }

    pub fn receive(&self) -> Vec<u64> {
        vec![
            self.receive_ap,
            self.receive_wan_24_ghz,
            self.receive_wan_5_ghz,
        ]
    }

    pub fn stations(&self) -> Vec<u64> {
        vec![self.stations_24_ghz, self.stations_5_ghz]
    }

    pub fn transmit(&self) -> Vec<u64> {
        vec![
            self.transmit_ap,
            self.transmit_wan_24_ghz,
            self.transmit_wan_5_ghz,
        ]
    }
}
