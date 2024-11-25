use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Clone, Debug, Deserialize, Display, strum::EnumProperty, Eq, PartialEq, Serialize)]
pub enum Action {
    ClearScreen,
    Error(String),
    #[strum(props(Help = "Show detail"))]
    EventLogDetailShow,
    #[strum(props(Help = "Create filter"))]
    EventLogFilterCreate,
    #[strum(props(Help = "Last event"))]
    EventLogLast,
    #[strum(props(Help = "Show list"))]
    EventLogListShow,
    #[strum(props(Help = "Next event"))]
    EventLogNext,
    #[strum(props(Help = "Previous event"))]
    EventLogPrevious,
    #[strum(props(Help = "Scroll left"))]
    EventLogScrollLeft,
    #[strum(props(Help = "Scroll left x10"))]
    EventLogScrollLeftBig,
    #[strum(props(Help = "Reset scroll"))]
    EventLogScrollReset,
    #[strum(props(Help = "Scroll right"))]
    EventLogScrollRight,
    #[strum(props(Help = "Scroll right x10"))]
    EventLogScrollRightBig,
    #[strum(props(Help = "Return to live"))]
    EventLogSelectClear,
    #[strum(props(Help = "First event"))]
    EventLogTop,
    #[strum(props(Help = "Toggle wrap"))]
    EventLogWrapToggle,
    #[strum(props(Help = "Add filter"))]
    FilterAdd,
    #[strum(props(Help = "Cancel filter add/edit", Back = "true"))]
    FilterCancel,
    #[strum(props(Help = "Cancel filter creation"))]
    FilterCreateCancel,
    #[strum(props(Help = "Next setting"))]
    FilterCreateNext,
    #[strum(props(Help = "Last setting"))]
    FilterCreateLast,
    #[strum(props(Help = "Bottom setting"))]
    FilterCreatePrevious,
    #[strum(props(Help = "Toggle setting"))]
    FilterCreateToggle,
    #[strum(props(Help = "Top setting"))]
    FilterCreateTop,
    #[strum(props(Help = "Delete filter"))]
    FilterDelete,
    #[strum(props(Help = "Edit filter"))]
    FilterEdit,
    #[strum(props(Help = "Edit filter", Back = "true"))]
    FilterHide,
    #[strum(props(Help = "Last filter"))]
    FilterLast,
    #[strum(props(Help = "Next filter"))]
    FilterNext,
    #[strum(props(Help = "Previous filter"))]
    FilterPrevious,
    #[strum(props(Help = "First filter"))]
    FilterTop,
    #[strum(props(Help = "Show filters"))]
    FilterShow,
    #[strum(props(Help = "Submit filter add/edit"))]
    FilterSubmit,
    #[strum(props(Help = "Hide format dialog", Back = "true"))]
    FormatHide,
    #[strum(props(Help = "Toggle format setting"))]
    FormatRowEdit,
    #[strum(props(Help = "Last format setting"))]
    FormatRowLast,
    #[strum(props(Help = "Next format setting"))]
    FormatRowNext,
    #[strum(props(Help = "First format setting"))]
    FormatRowTop,
    #[strum(props(Help = "Previous format setting"))]
    FormatRowPrevious,
    #[strum(props(Help = "Show format dialog"))]
    FormatShow,
    Input(KeyEvent),
    #[strum(props(Help = "Hide help", Back = "true"))]
    HelpHide,
    #[strum(props(Help = "Show help"))]
    HelpShow,
    #[strum(props(Help = "Quit"))]
    Quit,
    Render,
    Resize(u16, u16),
    Resume,
    #[strum(props(Help = "Suspend"))]
    Suspend,
    Tick,
}
