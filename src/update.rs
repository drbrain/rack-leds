mod switch;

pub use switch::Switch;

#[derive(Clone)]
pub enum Update {
    Switch(Switch),
}

impl Update {
    pub fn height(&self) -> u16 {
        match self {
            Update::Switch(switch) => switch.height(),
        }
    }

    pub fn width(&self) -> u16 {
        match self {
            Update::Switch(switch) => switch.width(),
        }
    }
}
