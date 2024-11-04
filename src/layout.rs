use eyre::{OptionExt, Result};
use std::fmt::Display;

use crate::collector::prometheus;

#[derive(Clone, Copy, Debug)]
pub enum Layout {
    SwitchFive,
    SwitchEight,
    SwitchEightPlusTwo,
    SwitchSixteenPlusTwo,
    Unknown,
}

impl Layout {
    pub async fn new(connection: &prometheus::Connection, labels: impl Display) -> Result<Self> {
        let query = format!("sysDescr{{{labels}}}");
        let description = connection.get_label(query, "sysDescr").await?;

        if description.starts_with("USW-8-150W,") {
            return Ok(Self::SwitchEightPlusTwo);
        } else if description.starts_with("US-8,") {
            return Ok(Self::SwitchEight);
        } else if description.starts_with("USW-Flex ") {
            return Ok(Self::SwitchFive);
        }

        let query = format!("count(ifHCInOctets{{{labels}, ifAlias=~\"(Port|SFP) .*\"}})");
        let result = connection.get_values(query).await?;

        let interfaces = result.first().ok_or_eyre("No interfaces found")?;

        match interfaces {
            5 => Ok(Self::SwitchFive),
            8 => Ok(Self::SwitchEight),
            10 => Ok(Self::SwitchEightPlusTwo),
            18 => Ok(Self::SwitchSixteenPlusTwo),
            _ => Ok(Self::Unknown),
        }
    }

    pub fn simulate(ports: usize) -> Self {
        match ports {
            5 => Self::SwitchFive,
            8 => Self::SwitchEight,
            10 => Self::SwitchEightPlusTwo,
            18 => Self::SwitchSixteenPlusTwo,
            _ => Self::Unknown,
        }
    }

    pub fn coordinate(&self, index: usize) -> (f64, f64) {
        match self {
            Layout::SwitchFive | Layout::SwitchEight => (index as f64, 0.0),
            Layout::SwitchEightPlusTwo => (if index < 8 { index } else { index + 1 } as f64, 0.0),
            Layout::SwitchSixteenPlusTwo => (
                if index < 16 {
                    index / 2
                } else {
                    (index / 2) + 1
                } as f64,
                (index % 2) as f64,
            ),
            Layout::Unknown => (0.0, 0.0),
        }
    }

    pub fn height(&self) -> u16 {
        match self {
            Layout::SwitchFive | Layout::SwitchEight | Layout::SwitchEightPlusTwo => 1,
            Layout::SwitchSixteenPlusTwo => 2,
            Layout::Unknown => 1,
        }
    }

    pub fn width(&self) -> u16 {
        match self {
            Layout::SwitchFive => 5,
            Layout::SwitchEight => 8,
            Layout::SwitchEightPlusTwo => 11,
            Layout::SwitchSixteenPlusTwo => 10,
            Layout::Unknown => 1,
        }
    }

    pub fn x_bound(&self) -> f64 {
        self.width().saturating_sub(1).into()
    }

    pub fn y_bound(&self) -> f64 {
        if self.height() == 1 {
            1.0
        } else {
            self.height().saturating_sub(1).into()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn coordinate_switch_five() {
        let layout = Layout::SwitchFive;

        assert_eq!((0.0, 0.0), layout.coordinate(0));
        assert_eq!((1.0, 0.0), layout.coordinate(1));
        assert_eq!((4.0, 0.0), layout.coordinate(4));
    }

    #[test]
    fn coordinate_switch_eight() {
        let layout = Layout::SwitchEight;

        assert_eq!((0.0, 0.0), layout.coordinate(0));
        assert_eq!((1.0, 0.0), layout.coordinate(1));
        assert_eq!((7.0, 0.0), layout.coordinate(7));
    }

    #[test]
    fn coordinate_switch_eight_plus_two() {
        let layout = Layout::SwitchEightPlusTwo;

        assert_eq!((0.0, 0.0), layout.coordinate(0));
        assert_eq!((1.0, 0.0), layout.coordinate(1));
        assert_eq!((7.0, 0.0), layout.coordinate(7));

        assert_eq!((9.0, 0.0), layout.coordinate(8));
        assert_eq!((10.0, 0.0), layout.coordinate(9));
    }

    #[test]
    fn coordinate_switch_sixteen_plus_two() {
        let layout = Layout::SwitchSixteenPlusTwo;

        assert_eq!((0.0, 0.0), layout.coordinate(0));
        assert_eq!((0.0, 1.0), layout.coordinate(1));
        assert_eq!((1.0, 0.0), layout.coordinate(2));
        assert_eq!((1.0, 1.0), layout.coordinate(3));
        assert_eq!((7.0, 0.0), layout.coordinate(14));
        assert_eq!((7.0, 1.0), layout.coordinate(15));

        assert_eq!((9.0, 0.0), layout.coordinate(16));
        assert_eq!((9.0, 1.0), layout.coordinate(17));
    }
}
