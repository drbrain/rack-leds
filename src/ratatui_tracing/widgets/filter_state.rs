use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

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

#[derive(Clone)]
pub struct FilterState<'a> {
    pub(crate) reloadable: Reloadable,
    pub(crate) filter_edit_state: Arc<Mutex<FilterEditState<'a>>>,
    pub(crate) list_state: Arc<Mutex<ListState>>,
    view_state: Arc<Mutex<ViewState>>,
}

impl<'a> FilterState<'a> {
    pub fn new(reloadable: Reloadable) -> Self {
        let state = ListState::default().with_offset(0).with_selected(Some(0));

        Self {
            filter_edit_state: Default::default(),
            reloadable,
            list_state: Arc::new(Mutex::new(state)),
            view_state: Default::default(),
        }
    }

    pub fn add_start(&self) {
        let mut view_state = self.view_state.lock().unwrap();

        if let Some(state) = view_state.try_to_add() {
            *view_state = state;

            let mut guard = self.filter_edit_state.lock().unwrap();

            guard.clear();
        }
    }

    pub fn cancel(&self) {
        {
            let mut view_state = self.view_state.lock().unwrap();

            *view_state = view_state.to_view();
        }

        {
            let mut guard = self.filter_edit_state.lock().unwrap();

            guard.clear();
        }
    }

    pub fn delete_selected(&self) {
        let selected = {
            let guard = self.list_state.lock().unwrap();

            guard.selected()
        };

        if let Some(selected) = selected {
            self.reloadable.delete(selected);
        }
    }

    pub fn edit_start(&self) {
        let (original, directive) = {
            let state = self.list_state.lock().unwrap();

            let original = state.selected();

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

        let mut view_state = self.view_state.lock().unwrap();

        let Some(state) = view_state.try_to_edit(original) else {
            return;
        };

        *view_state = state;

        drop(view_state);

        let mut guard = self.filter_edit_state.lock().unwrap();

        guard.clear();

        guard.insert(directive);
    }

    pub fn key(&self, key: KeyEvent) {
        let mut guard = self.filter_edit_state.lock().unwrap();
        guard.key(key);
    }

    pub fn row_last(&self) {
        let mut guard = self.list_state.lock().unwrap();

        guard.select_last()
    }

    pub fn row_first(&self) {
        let mut guard = self.list_state.lock().unwrap();

        guard.select_first()
    }

    pub fn row_next(&self) {
        let mut guard = self.list_state.lock().unwrap();

        guard.select_next()
    }

    pub fn row_previous(&self) {
        let mut guard = self.list_state.lock().unwrap();

        guard.select_previous()
    }

    pub fn submit(&self) {
        let directive = {
            let mut guard = self.filter_edit_state.lock().unwrap();

            guard.directive()
        };

        let mut view_state = self.view_state.lock().unwrap();

        if let Some(directive) = directive {
            match view_state.deref() {
                ViewState::Add => (),
                ViewState::Edit { original } => {
                    self.reloadable.delete(*original);
                }
                ViewState::View => return,
            }

            self.reloadable.add(directive);

            *view_state = view_state.to_view();
        }
    }

    pub fn view_state(&self) -> ViewState {
        let view_state = self.view_state.lock().unwrap();

        view_state.clone()
    }
}
