use std::{
    collections::HashMap,
    convert::Into,
    iter,
    sync::{Arc, Mutex},
};

use crossterm::event::KeyEvent;
use eyre::Result;
use itertools::Itertools;
use layout::Flex;
use ratatui::{
    prelude::*,
    widgets::{Cell, Clear, Paragraph, Row, Table},
    Frame,
};
use strum::EnumProperty;

use crate::ui::{app::Mode, widgets::Border, Action, Component, Config};

pub struct Help {
    config: Option<Config>,
    previous_mode: Arc<Mutex<Option<Mode>>>,
    render: bool,
}

impl Help {
    pub fn new(previous_mode: Arc<Mutex<Option<Mode>>>) -> Self {
        Self {
            config: Default::default(),
            previous_mode,
            render: false,
        }
    }

    fn active_keys(&self) -> Option<Vec<(String, String)>> {
        let config = &(self.config.clone()?);

        let mode = self.previous_mode();

        Some(key_help(config.keybindings.get(&mode)?))
    }

    fn previous_mode(&self) -> Mode {
        let guard = self.previous_mode.lock().unwrap();

        guard.clone().unwrap_or_default()
    }
}

fn key_help(map: &HashMap<Vec<KeyEvent>, Action>) -> Vec<(String, String)> {
    map.iter()
        .flat_map(|(keys, action)| keys.iter().map(|key| (key, action)).collect::<Vec<_>>())
        .sorted_by_cached_key(|(key, _)| key.code.to_string())
        .map(|(key, action)| {
            let help = action.get_str("Help").unwrap_or("unknown");

            (format!("<{}>", key_name(key)), help.to_string())
        })
        .collect::<Vec<_>>()
}

fn key_name(key: &KeyEvent) -> String {
    key.modifiers
        .iter_names()
        .map(|(name, _)| name.to_string())
        .chain(iter::once(key.code.to_string()))
        .reduce(|key, part| format!("{key}-{part}"))
        .unwrap_or("what".into())
}

impl Component for Help {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if !self.render {
            return Ok(());
        }

        let mode: &str = self.previous_mode().into();

        let border = Border::new().border_type("Help").title(mode).uniform(1);

        let Some(keys) = self.active_keys() else {
            return draw_no_help(mode, &border, area, frame);
        };

        let rows: Vec<_> = keys
            .iter()
            .sorted_by_cached_key(|(key, _)| key)
            .map(|(key, help)| {
                let width = key.len().max(help.len());

                let row = Row::new(vec![
                    Cell::new(Text::from(key.as_str()).right_aligned()),
                    Cell::new(Text::from(help.as_str()).left_aligned()),
                ]);

                (row, width)
            })
            .collect();

        let width = rows.iter().map(|(_, width)| width).max().unwrap_or(&40) + 2;
        let width = width.try_into().unwrap_or(u16::MAX);
        let height = rows.len().min(area.height as usize) + 4;

        let rows: Vec<_> = rows.into_iter().map(|(row, _)| row).collect();

        let table = Table::new(rows, Constraint::from_lengths([width, width]))
            .block((&border).into())
            .column_spacing(2)
            .flex(Flex::Center);

        let area = center_exactly(area, width * 2, height);

        frame.render_widget(Clear, area);
        frame.render_widget(table, area);

        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = Some(config);

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::HelpShow => {
                self.render = true;
            }
            Action::HelpHide => {
                self.render = false;
            }
            _ => (),
        }

        Ok(None)
    }
}

fn center_exactly(area: Rect, width: u16, height: usize) -> Rect {
    let [_, area, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width),
        Constraint::Fill(1),
    ])
    .areas(area);

    let [_, area, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height.try_into().unwrap_or(u16::MAX)),
        Constraint::Fill(1),
    ])
    .areas(area);
    area
}

fn draw_no_help(mode: &str, border: &Border<'_>, area: Rect, frame: &mut Frame<'_>) -> Result<()> {
    let mut line = Line::default();

    line.push_span(Span::raw("Missing key map for current mode "));
    line.push_span(Span::styled(mode, Style::default().italic()));

    let width = line.width() + 4;
    let width = width.try_into().unwrap_or(u16::MAX);

    let paragraph = Paragraph::new(line).block(border.into());
    let height = paragraph.line_count(width);
    let area = center_exactly(area, width, height);

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);

    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use rstest::rstest;

    use crate::ui::Action;

    #[test]
    fn key_help() {
        let mut map: HashMap<Vec<KeyEvent>, Action> = HashMap::default();
        map.insert(
            vec![KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)],
            Action::Quit,
        );
        map.insert(
            vec![KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)],
            Action::Quit,
        );
        map.insert(
            vec![KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE)],
            Action::FilterShow,
        );

        let help = super::key_help(&map);

        let expected = vec![
            ("<CONTROL-c>".to_string(), "Quit".to_string()),
            ("<e>".to_string(), "Show filters".to_string()),
            ("<q>".to_string(), "Quit".to_string()),
        ];

        assert_eq!(expected, help);
    }

    #[rstest]
    #[case(KeyCode::Char('f'), KeyModifiers::NONE, "f")]
    #[case(KeyCode::Char('f'), KeyModifiers::ALT, "ALT-f")]
    #[case(KeyCode::Esc, KeyModifiers::NONE, "Esc")]
    #[case(KeyCode::Esc, KeyModifiers::SHIFT, "SHIFT-Esc")]
    fn key_name(#[case] code: KeyCode, #[case] modifiers: KeyModifiers, #[case] expected: &str) {
        let key = KeyEvent::new(code, modifiers);

        assert_eq!(expected.to_string(), super::key_name(&key), "for {key:?}");
    }
}

