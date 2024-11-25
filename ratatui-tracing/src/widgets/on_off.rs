#[derive(Clone, Copy, Default, strum::IntoStaticStr, PartialEq)]
pub(crate) enum OnOff {
    #[default]
    On,
    Off,
}

impl OnOff {
    pub fn next(&self) -> Self {
        match self {
            Self::On => Self::Off,
            Self::Off => Self::On,
        }
    }
}
