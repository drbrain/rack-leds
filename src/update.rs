mod access_point;
mod switch;

pub use access_point::AccessPoint;
pub use switch::Switch;

use crate::{device::Id, Layout};

#[derive(Clone, Debug)]
pub enum Update {
    AccessPoint {
        id: Id,
        device: AccessPoint,
        layout: Layout,
    },
    Switch {
        id: Id,
        device: Switch,
        layout: Layout,
    },
}

impl Update {
    pub fn height(&self) -> u16 {
        self.layout().height()
    }

    pub fn id(&self) -> Id {
        match self {
            Update::AccessPoint { id, .. } => *id,
            Update::Switch { id, .. } => *id,
        }
    }

    fn layout(&self) -> Layout {
        match self {
            Update::AccessPoint { layout, .. } => *layout,
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
