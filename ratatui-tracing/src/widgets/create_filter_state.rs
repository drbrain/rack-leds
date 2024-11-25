use std::{fmt::Display, str::FromStr, sync::Arc};

use tracing_subscriber::filter::Directive;

use crate::{
    widgets::{EventLogState, Level},
    Event, Reloadable,
};

pub struct CreateFilterState {
    pub(crate) event: Arc<Event>,
    reloadable: Reloadable,
    selection: Selection,
    pub(crate) level: Level,
    pub(crate) target: Target,
}

impl CreateFilterState {
    pub fn new(event: Arc<Event>, reloadable: Reloadable) -> Self {
        let target = Target::new(event.target.clone());

        Self {
            event,
            reloadable,
            selection: Default::default(),
            level: Default::default(),
            target,
        }
    }

    pub fn directive(&self) -> Option<Directive> {
        let target: Option<String> = (&self.target).try_into().ok();

        let level = self.level;

        let directive = match (target, level) {
            (None, level) => {
                let level: &'static str = level.into();
                level.to_string()
            }
            (Some(target), level) => {
                let level: &'static str = level.into();

                format!("{target}={level}")
            }
        };

        Directive::from_str(&directive).ok()
    }

    pub fn level_selected(&self) -> bool {
        matches!(self.selection, Selection::Level)
    }

    pub fn select_next(&mut self) {
        self.selection = self.selection.next();
    }

    pub fn target_selected(&self) -> bool {
        matches!(self.selection, Selection::Target)
    }

    pub fn toggle(&mut self) {
        match self.selection {
            Selection::Target => self.target.next(),
            Selection::Level => self.level = self.level.next(),
        }
    }
}

impl<'a> From<&EventLogState<'a>> for CreateFilterState {
    fn from(state: &EventLogState<'a>) -> Self {
        CreateFilterState::new(state.selected_event(), state.filter.reloadable.clone())
    }
}

#[derive(Default)]
enum Selection {
    #[default]
    Target,
    Level,
}

impl Selection {
    fn next(&self) -> Self {
        match self {
            Self::Target => Self::Level,
            Self::Level => Self::Target,
        }
    }
}

pub(crate) struct Target {
    target: Vec<String>,
    selected: Vec<String>,
}

impl Target {
    fn new(target: String) -> Self {
        let target: Vec<_> = target.split("::").map(|part| part.to_string()).collect();
        let selected = target.clone();

        Self { target, selected }
    }

    fn next(&mut self) {
        if self.selected.pop().is_none() {
            self.selected = self.target.clone();
        }
    }
}

impl TryInto<String> for Target {
    type Error = ();

    fn try_into(self) -> Result<String, Self::Error> {
        (&self).try_into()
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.selected.is_empty() {
            f.write_str("[Off]")
        } else {
            f.write_str(&self.selected.join("::"))
        }
    }
}

impl TryInto<String> for &Target {
    // We only care about Some/None so we'll `.ok()` this and ignore the error
    type Error = ();

    fn try_into(self) -> Result<String, Self::Error> {
        if self.selected.is_empty() {
            Err(())
        } else {
            Ok(self.selected.join("::"))
        }
    }
}
