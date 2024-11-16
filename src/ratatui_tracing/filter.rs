use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use crossterm::event::KeyEvent;
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{
        Block, BorderType, Clear, HighlightSpacing, List, ListItem, ListState, Padding,
        StatefulWidget, Widget,
    },
};

use crate::ratatui_tracing::FilterEdit;
use crate::ratatui_tracing::Reloadable;

#[derive(Clone, Default)]
enum ViewState {
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
pub struct Filter<'a> {
    reloadable: Reloadable,
    filter_edit: FilterEdit<'a>,
    state: Arc<Mutex<ListState>>,
    view_state: Arc<Mutex<ViewState>>,
}

impl<'a> Filter<'a> {
    pub fn new(reloadable: Reloadable) -> Self {
        let state = ListState::default().with_offset(0).with_selected(Some(0));

        Self {
            filter_edit: Default::default(),
            reloadable,
            state: Arc::new(Mutex::new(state)),
            view_state: Default::default(),
        }
    }

    pub fn add_start(&self) {
        let mut view_state = self.view_state.lock().unwrap();

        if let Some(state) = view_state.try_to_add() {
            *view_state = state;

            self.filter_edit.clear();
        }
    }

    pub fn cancel(&self) {
        let mut view_state = self.view_state.lock().unwrap();

        *view_state = view_state.to_view();

        self.filter_edit.clear();
    }

    pub fn delete_selected(&self) {
        let selected = {
            let guard = self.state.lock().unwrap();

            guard.selected()
        };

        if let Some(selected) = selected {
            self.reloadable.delete(selected);
        }
    }

    pub fn edit_start(&self) {
        let (original, directive) = {
            let state = self.state.lock().unwrap();

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

        self.filter_edit.clear();

        self.filter_edit.insert(directive);
    }

    pub fn key(&self, key: KeyEvent) {
        self.filter_edit.key(key);
    }

    pub fn row_last(&self) {
        let mut guard = self.state.lock().unwrap();

        guard.select_last()
    }

    pub fn row_first(&self) {
        let mut guard = self.state.lock().unwrap();

        guard.select_first()
    }

    pub fn row_next(&self) {
        let mut guard = self.state.lock().unwrap();

        guard.select_next()
    }

    pub fn row_previous(&self) {
        let mut guard = self.state.lock().unwrap();

        guard.select_previous()
    }

    fn render_list(&self, area: Rect, buf: &mut Buffer) {
        let dialog_border = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from("Filters").bold())
            .title_bottom(Line::from("Esc to dismiss").right_aligned().italic())
            .padding(Padding::symmetric(1, 0));

        let items: Vec<_> = self
            .reloadable
            .directives()
            .iter()
            .map(|directive| ListItem::new(directive.to_string()))
            .collect();

        let list = List::new(items)
            .block(dialog_border)
            .highlight_symbol("❯")
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_style(Style::default().bold().fg(Color::Black).bg(Color::Gray))
            .direction(ratatui::widgets::ListDirection::TopToBottom);

        let mut state = self.state.lock().unwrap();

        StatefulWidget::render(list, area, buf, &mut state);
    }

    pub fn submit(&self) {
        let mut view_state = self.view_state.lock().unwrap();

        if let Some(directive) = self.filter_edit.directive() {
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

    fn view_state(&self) -> ViewState {
        let view_state = self.view_state.lock().unwrap();

        view_state.clone()
    }
}

impl Widget for &Filter<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Clear.render(area, buf);

        match self.view_state() {
            ViewState::Add | ViewState::Edit { .. } => {
                self.filter_edit.render(area, buf);
            }
            ViewState::View => {
                self.render_list(area, buf);
            }
        }
    }
}
