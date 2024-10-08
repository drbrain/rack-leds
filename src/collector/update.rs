mod switch;

pub use switch::Switch;

#[derive(Clone)]
pub enum Update {
    Switch(Switch),
}
