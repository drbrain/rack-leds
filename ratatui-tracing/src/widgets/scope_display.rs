#[derive(Clone, Copy, Default, strum::IntoStaticStr, PartialEq)]
pub enum ScopeDisplay {
    #[default]
    All,
    Last,
    None,
}

impl ScopeDisplay {
    pub fn next(&self) -> Self {
        match self {
            ScopeDisplay::All => ScopeDisplay::Last,
            ScopeDisplay::Last => ScopeDisplay::None,
            ScopeDisplay::None => ScopeDisplay::All,
        }
    }
}
