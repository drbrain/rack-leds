mod switch;

pub use switch::Switch;

use crate::Layout;

#[derive(Clone, Debug)]
pub enum Update {
    Switch { device: Switch, layout: Layout },
}

impl Update {
    pub fn height(&self) -> u16 {
        self.layout().height()
    }

    fn layout(&self) -> Layout {
        match self {
            Update::Switch { layout, .. } => *layout,
        }
    }

    pub fn width(&self) -> u16 {
        self.layout().width()
    }

    pub fn x_bound(&self) -> f64 {
        self.layout().x_bound()
    }

    pub fn y_bound(&self) -> f64 {
        self.layout().y_bound()
    }
}
