#[derive(Clone, Default)]
pub(crate) enum ViewState {
    Add,
    Edit {
        original: usize,
    },
    #[default]
    View,
}

impl ViewState {
    pub fn is_editing(&self) -> bool {
        matches!(self, ViewState::Add | ViewState::Edit { .. })
    }

    pub fn try_to_add(&self) -> Option<Self> {
        match self {
            Self::View => Some(Self::Add),
            _ => None,
        }
    }

    pub fn try_to_edit(&self, original: Option<usize>) -> Option<Self> {
        match (self, original) {
            (ViewState::View, None) => Some(Self::Add),
            (ViewState::View, Some(original)) => Some(Self::Edit { original }),
            _ => None,
        }
    }

    pub fn to_view(&self) -> Self {
        Self::View
    }
}