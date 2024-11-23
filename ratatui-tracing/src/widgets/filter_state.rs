use crossterm::event::KeyEvent;
use ratatui::widgets::ListState;

use crate::{
    widgets::{FilterEditState, ViewState},
    Reloadable,
};

/// State of a [`super::Filter`] widget
///
/// Whether filter directives are being viewed, edited, or added and allows deleting directives.
pub struct FilterState<'a> {
    pub(crate) reloadable: Reloadable,
    pub(crate) filter_edit_state: FilterEditState<'a>,
    pub(crate) list_state: ListState,
    view_state: ViewState,
}

impl<'a> FilterState<'a> {
    /// Create a [`FilterState`] that will modify a [`Reloadable`]
    pub fn new(reloadable: Reloadable) -> Self {
        let list_state = ListState::default().with_offset(0).with_selected(Some(0));

        Self {
            filter_edit_state: Default::default(),
            reloadable,
            list_state,
            view_state: Default::default(),
        }
    }

    /// Start adding a new directive displaying an input box
    pub fn add_start(&mut self) {
        if let Some(state) = self.view_state.try_to_add() {
            self.view_state = state;

            self.filter_edit_state.clear();
        }
    }

    /// Cancel adding or editing a directive and return to list view
    pub fn cancel(&mut self) {
        self.view_state = self.view_state.to_view();

        self.filter_edit_state.clear();
    }

    /// Delete the selected directive in list view
    pub fn delete_selected(&mut self) {
        let selected = self.list_state.selected();

        if let Some(selected) = selected {
            self.reloadable.delete(selected);
        }
    }

    /// Start editing the selected directive
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

    /// True when the filter is in add mode
    pub fn is_adding(&self) -> bool {
        self.view_state.is_add()
    }

    /// True when the filter is in edit mode
    pub fn is_editing(&self) -> bool {
        self.view_state.is_edit()
    }

    /// True when the filter is in view mode
    pub fn is_viewing(&self) -> bool {
        self.view_state.is_view()
    }

    /// Forward a key event to the textarea when in editing mode
    pub fn key(&mut self, key: KeyEvent) {
        self.filter_edit_state.key(key);
    }

    /// Select the last row
    pub fn row_last(&mut self) {
        self.list_state.select_last()
    }

    /// Select the first row
    pub fn row_first(&mut self) {
        self.list_state.select_first();
    }

    /// Select the next row
    pub fn row_next(&mut self) {
        self.list_state.select_next();
    }

    /// Select the previous row
    pub fn row_previous(&mut self) {
        self.list_state.select_previous();
    }

    /// When in add mode, add the directive to the filter
    ///
    /// When in edit mode, replace the selected directive
    ///
    /// If the directive does not parse an error is displayed and the filter remains in editing
    /// mode
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
}
