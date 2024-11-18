use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    ClearScreen,
    Error(String),
    EventLogDetailShow,
    EventLogLast,
    EventLogListShow,
    EventLogNext,
    EventLogPrevious,
    EventLogSelectClear,
    EventLogTop,
    FilterAdd,
    FilterCancel,
    FilterDelete,
    FilterEdit,
    FilterHide,
    FilterLast,
    FilterNext,
    FilterPrevious,
    FilterTop,
    FilterShow,
    FilterSubmit,
    FormatHide,
    FormatRowEdit,
    FormatRowLast,
    FormatRowNext,
    FormatRowTop,
    FormatRowPrevious,
    FormatShow,
    Input(KeyEvent),
    Help,
    Quit,
    Render,
    Resize(u16, u16),
    Resume,
    Suspend,
    Tick,
}
