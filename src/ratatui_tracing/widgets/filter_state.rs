use crossterm::event::KeyEvent;
use ratatui::widgets::ListState;

use crate::ratatui_tracing::widgets::FilterEditState;
use crate::ratatui_tracing::Reloadable;

#[derive(Clone, Default)]
pub enum ViewState {
    Add,
    Edit {
        original: usize,
    },
    #[default]
    View,
}

impl ViewState {
    fn try_to_add(&self) -> Option<Self> {
        match self {
            Self::View => Some(Self::Add),
            _ => None,
        }
    }

    fn try_to_edit(&self, original: Option<usize>) -> Option<Self> {
        match (self, original) {
            (ViewState::View, None) => Some(Self::Add),
            (ViewState::View, Some(original)) => Some(Self::Edit { original }),
            _ => None,
        }
    }

    fn to_view(&self) -> Self {
        Self::View
    }
}

pub struct FilterState<'a> {
    pub(crate) reloadable: Reloadable,
    pub(crate) filter_edit_state: FilterEditState<'a>,
    pub(crate) list_state: ListState,
    view_state: ViewState,
}

impl<'a> FilterState<'a> {
    pub fn new(reloadable: Reloadable) -> Self {
        let list_state = ListState::default().with_offset(0).with_selected(Some(0));

        Self {
            filter_edit_state: Default::default(),
            reloadable,
            list_state,
            view_state: Default::default(),
        }
    }

    pub fn add_start(&mut self) {
        if let Some(state) = self.view_state.try_to_add() {
            self.view_state = state;

            self.filter_edit_state.clear();
        }
    }

    pub fn cancel(&mut self) {
        self.view_state = self.view_state.to_view();

        self.filter_edit_state.clear();
    }

    pub fn delete_selected(&mut self) {
        let selected = self.list_state.selected();

        if let Some(selected) = selected {
            self.reloadable.delete(selected);
        }
    }

    pub fn edit_start(&mut self) {
        let (original, directive) = {
            let original = self.list_state.selected();

            let directive = original
                .and_then(|index| {
                    self.reloadable
                        .directives()
                        .get(index)
                        .map(|directive| directive.to_string())
                })
                .unwrap_or("".to_string());

            (original, directive)
        };

        let Some(state) = self.view_state.try_to_edit(original) else {
            return;
        };

        self.view_state = state;

        self.filter_edit_state.clear();
        self.filter_edit_state.insert(directive);
    }

    pub fn key(&mut self, key: KeyEvent) {
        self.filter_edit_state.key(key);
    }

    pub fn row_last(&mut self) {
        self.list_state.select_last()
    }

    pub fn row_first(&mut self) {
        self.list_state.select_first();
    }

    pub fn row_next(&mut self) {
        self.list_state.select_next();
    }

    pub fn row_previous(&mut self) {
        self.list_state.select_previous();
    }

    pub fn submit(&mut self) {
        let directive = self.filter_edit_state.directive();

        if let Some(directive) = directive {
            match self.view_state {
                ViewState::Add => (),
                ViewState::Edit { original } => {
                    self.reloadable.delete(original);
                }
                ViewState::View => return,
            }

            self.reloadable.add(directive);

            self.view_state = self.view_state.to_view();
        }
    }

    pub fn view_state(&self) -> ViewState {
        self.view_state.clone()
    }
}
