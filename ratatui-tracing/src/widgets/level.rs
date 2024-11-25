#[derive(Clone, Copy, Default, strum::IntoStaticStr, PartialEq)]
pub(crate) enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    #[default]
    Off,
}

impl Level {
    pub fn next(&self) -> Self {
        match self {
            Self::Trace => Self::Debug,
            Self::Debug => Self::Info,
            Self::Info => Self::Warn,
            Self::Warn => Self::Error,
            Self::Error => Self::Off,
            Self::Off => Self::Trace,
        }
    }
}
