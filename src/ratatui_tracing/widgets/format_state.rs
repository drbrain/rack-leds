use ratatui::widgets::TableState;
use time::UtcOffset;

use crate::ratatui_tracing::widgets::ScopeDisplay;
use crate::ratatui_tracing::widgets::TimeFormat;
use crate::LOCAL_OFFSET;

#[derive(Clone)]
pub struct FormatState {
    pub(crate) local_offset: UtcOffset,
    pub(crate) display_level: bool,
    pub(crate) display_scope: ScopeDisplay,
    pub(crate) display_scope_fields: bool,
    pub(crate) display_target: bool,
    pub(crate) table: TableState,
    pub(crate) time: TimeFormat,
    wrap: OnOff,
}

impl FormatState {
    pub fn as_rows(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Time", self.time.into()),
            ("Level", visibility(self.display_level)),
            ("Scope Display", self.display_scope.into()),
            ("Scope Fields", visibility(self.display_scope_fields)),
            ("Target", visibility(self.display_target)),
            ("Wrap", self.wrap.into()),
        ]
    }

    pub fn row_last(&mut self) {
        self.table.select_last()
    }

    pub fn row_edit(&mut self) {
        let selected = self.table.selected();

        let Some(selected) = selected else {
            return;
        };

        match selected {
            0 => {
                self.time = self.time.next();
            }
            1 => {
                self.display_level = !self.display_level;
            }
            2 => {
                self.display_scope = self.display_scope.next();
            }
            3 => {
                self.display_scope_fields = !self.display_scope_fields;
            }
            4 => {
                self.display_target = !self.display_target;
            }
            5 => {
                self.wrap = self.wrap.next();
            }
            _ => (),
        }
    }

    pub fn row_first(&mut self) {
        self.table.select_first()
    }

    pub fn row_next(&mut self) {
        self.table.select_next()
    }

    pub fn row_previous(&mut self) {
        self.table.select_previous()
    }

    pub fn wrap(&self) -> bool {
        self.wrap == OnOff::On
    }
}

impl Default for FormatState {
    fn default() -> Self {
        let local_offset = *LOCAL_OFFSET.get().expect("init::local_offset() not called");
        let table = TableState::new().with_selected_cell((0, 1));

        Self {
            display_level: true,
            display_scope: Default::default(),
            display_scope_fields: true,
            display_target: true,
            local_offset,
            table,
            time: Default::default(),
            wrap: OnOff::On,
        }
    }
}

#[derive(Clone, Copy, strum::IntoStaticStr, PartialEq)]
enum OnOff {
    On,
    Off,
}

impl OnOff {
    fn next(&self) -> OnOff {
        match self {
            OnOff::On => OnOff::Off,
            OnOff::Off => OnOff::On,
        }
    }
}

fn visibility(visible: bool) -> &'static str {
    if visible {
        "Show"
    } else {
        "Hide"
    }
}
