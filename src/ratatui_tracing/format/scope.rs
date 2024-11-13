use ratatui::text::{Text, ToText};

#[derive(Clone, Copy, Default, PartialEq)]
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

impl ToText for ScopeDisplay {
    fn to_text(&self) -> Text<'_> {
        let text = match self {
            ScopeDisplay::All => "All",
            ScopeDisplay::Last => "Last",
            ScopeDisplay::None => "None",
        };

        Text::from(text)
    }
}
