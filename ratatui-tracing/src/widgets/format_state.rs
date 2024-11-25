use ratatui::widgets::TableState;
use time::UtcOffset;

use crate::{
    widgets::{OnOff, ScopeDisplay, TimeFormat},
    EventReceiver,
};

/// State of a [`super::Format`] widget:
/// * The format of the event time
/// * Whether the event level is shown
/// * How many event scopes are shown
/// * Whether event scope fields are shown
/// * Whether the event target is shown
/// * Whether events are wrapped
#[derive(Clone)]
pub struct FormatState {
    pub(crate) local_offset: Option<UtcOffset>,
    display_level: ShowHide,
    pub(crate) display_scope: ScopeDisplay,
    display_scope_fields: ShowHide,
    display_target: ShowHide,
    pub(crate) table: TableState,
    pub(crate) time: TimeFormat,
    wrap: OnOff,
}

impl FormatState {
    /// Create a [`FormatState`] with a [`UtcOffset`]
    pub fn local_offset(local_offset: UtcOffset) -> Self {
        Self {
            local_offset: Some(local_offset),
            ..Default::default()
        }
    }

    pub(crate) fn as_rows(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Time", self.time.into()),
            ("Level", self.display_level.into()),
            ("Scope Display", self.display_scope.into()),
            ("Scope Fields", self.display_scope_fields.into()),
            ("Target", self.display_target.into()),
            ("Wrap", self.wrap.into()),
        ]
    }

    pub(crate) fn display_target(&self) -> bool {
        self.display_target == ShowHide::Show
    }

    pub(crate) fn display_scope_fields(&self) -> bool {
        self.display_scope_fields == ShowHide::Show
    }

    pub(crate) fn display_level(&self) -> bool {
        self.display_level == ShowHide::Show
    }

    /// Returns true if wrapping is enabled
    pub fn is_wrap(&self) -> bool {
        self.wrap == OnOff::On
    }

    /// Select the last row
    pub fn row_last(&mut self) {
        self.table.select_last()
    }

    /// Advance the row to its next state
    pub fn row_edit(&mut self) {
        let selected = self.table.selected();

        let Some(selected) = selected else {
            return;
        };

        match selected {
            0 => {
                self.time = self.time.next(self.local_offset);
            }
            1 => {
                self.display_level = self.display_level.next();
            }
            2 => {
                self.display_scope = self.display_scope.next();
            }
            3 => {
                self.display_scope_fields = self.display_scope_fields.next();
            }
            4 => {
                self.display_target = self.display_target.next();
            }
            5 => {
                self.wrap = self.wrap.next();
            }
            _ => (),
        }
    }

    /// Select the first row
    pub fn row_first(&mut self) {
        self.table.select_first()
    }

    /// Select the next row
    pub fn row_next(&mut self) {
        self.table.select_next()
    }

    /// Select the previous row
    pub fn row_previous(&mut self) {
        self.table.select_previous()
    }

    /// Toggle wrapping
    pub fn wrap_toggle(&mut self) {
        self.wrap = self.wrap.next();
    }
}

impl Default for FormatState {
    /// The default [`FormatState`] uses uptime time format and all other options visibile
    fn default() -> Self {
        let table = TableState::new().with_selected_cell((0, 1));

        Self {
            display_level: Default::default(),
            display_scope: Default::default(),
            display_scope_fields: Default::default(),
            display_target: Default::default(),
            local_offset: Default::default(),
            table,
            time: Default::default(),
            wrap: Default::default(),
        }
    }
}

impl From<&EventReceiver> for FormatState {
    fn from(event_receiver: &EventReceiver) -> Self {
        event_receiver
            .local_offset
            .map(Self::local_offset)
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy, Default, strum::IntoStaticStr, PartialEq)]
enum ShowHide {
    #[default]
    Show,
    Hide,
}

impl ShowHide {
    fn next(&self) -> ShowHide {
        match self {
            ShowHide::Show => ShowHide::Hide,
            ShowHide::Hide => ShowHide::Show,
        }
    }
}
