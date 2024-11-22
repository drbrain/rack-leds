use std::sync::{atomic::AtomicBool, Arc, Mutex};

use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use ratatui_tracing::{EventReceiver, Reloadable};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, instrument, trace, warn};

use crate::{
    collector::UpdateReceiver,
    png_builder::PngSender,
    ui::{
        action::Action,
        components::{fps::FpsCounter, home::Home, Component, Help},
        config::Config,
        tui::{Event, Tui},
    },
    Columns,
};

pub struct App {
    gui_active: Arc<AtomicBool>,
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    mode: Arc<Mutex<Mode>>,
    previous_mode: Arc<Mutex<Option<Mode>>>,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
}

#[derive(
    Debug, Default, Deserialize, Clone, Eq, Hash, strum::IntoStaticStr, PartialEq, Serialize,
)]
pub enum Mode {
    #[default]
    Home,
    #[strum(serialize = "Event log detail")]
    EventLogDetail,
    #[strum(serialize = "Filter")]
    Filter,
    #[strum(serialize = "Filter edit")]
    FilterEdit,
    #[strum(serialize = "Filter submit")]
    FilterSubmit,
    #[strum(serialize = "Format")]
    Format,
    #[strum(serialize = "Help")]
    Help,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gui_active: Arc<AtomicBool>,
        events: EventReceiver,
        reloadable: Reloadable,
        tick_rate: f64,
        frame_rate: f64,
        columns: Columns,
        updates: UpdateReceiver,
        png_sender: PngSender,
    ) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        render_on_update(events.resubscribe(), updates.clone(), action_tx.clone())?;

        let mode = Arc::new(Mutex::new(Mode::Home));
        let previous_mode = Arc::new(Mutex::new(None));

        Ok(Self {
            gui_active,
            tick_rate,
            frame_rate,
            components: vec![
                Box::new(Home::new(columns, updates, png_sender, events, reloadable)),
                Box::new(FpsCounter::default()),
                Box::new(Help::new(previous_mode.clone())),
            ],
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            mode,
            previous_mode,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
        })
    }

    pub fn action_tx(&self) -> mpsc::UnboundedSender<Action> {
        self.action_tx.clone()
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new(self.gui_active.clone())?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);

        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }

        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }

        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        let action_tx = self.action_tx.clone();

        loop {
            self.handle_events(&mut tui).await?;

            self.handle_actions(&mut tui)?;

            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }

        tui.exit()?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(ref event) = tui.next_event().await else {
            return Ok(());
        };

        let action_tx = self.action_tx.clone();

        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(*x, *y))?,
            Event::Key(key) => self.handle_key_event(*key)?,
            event => {
                debug!(?event, "unhandled");
            }
        }

        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }

        Ok(())
    }

    #[instrument(skip_all, fields(?key))]
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let Some(keymap) = self.config.keybindings.get(&self.mode()) else {
            return Ok(());
        };

        let action_tx = self.action_tx.clone();

        match keymap.get(&vec![key]) {
            Some(action) => {
                debug!(?action, "got action");
                action_tx.send(action.clone())?;
            }
            _ => {
                if self.mode() == Mode::FilterEdit {
                    action_tx.send(Action::Input(key))?;

                    return Ok(());
                }

                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    debug!(?action, "got action");
                    action_tx.send(action.clone())?;
                }
            }
        }
        Ok(())
    }

    #[instrument(skip_all)]
    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                trace!(?action);
            }

            match action {
                Action::ClearScreen => tui.terminal.clear()?,
                Action::EventLogDetailShow => {
                    self.set_mode(Mode::EventLogDetail);
                }
                Action::EventLogListShow | Action::FilterHide | Action::FormatHide => {
                    self.set_mode(Mode::Home);
                }
                Action::FilterAdd | Action::FilterEdit => {
                    self.set_mode(Mode::FilterEdit);
                }
                Action::FilterCancel | Action::FilterShow | Action::FilterSubmit => {
                    self.set_mode(Mode::Filter);
                }
                Action::FormatShow => {
                    self.set_mode(Mode::Format);
                }
                Action::HelpShow => {
                    {
                        let mut guard = self.previous_mode.lock().unwrap();
                        *guard = Some(self.mode());
                    }

                    self.set_mode(Mode::Help);
                }
                Action::HelpHide => {
                    let previous = {
                        let mut guard = self.previous_mode.lock().unwrap();
                        let previous = guard.clone().unwrap_or_default();
                        *guard = None;
                        previous
                    };

                    self.set_mode(previous);
                }
                Action::Quit => self.should_quit = true,
                Action::Render => self.render(tui)?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Resume => self.should_suspend = false,
                Action::Suspend => self.should_suspend = true,
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                _ => {}
            }

            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }

        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;

        self.render(tui)?;

        Ok(())
    }

    pub fn mode(&self) -> Mode {
        let guard = self.mode.lock().unwrap();

        guard.clone()
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, frame.area()) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {:?}", err)));
                }
            }
        })?;

        Ok(())
    }

    pub fn set_mode(&self, mode: Mode) {
        let mut guard = self.mode.lock().unwrap();

        if mode == *guard {
            return;
        }

        *guard = mode.clone();

        debug!(?mode, "mode switched");
    }
}

fn render_on_update(
    mut events: EventReceiver,
    mut updates: UpdateReceiver,
    action_tx: mpsc::UnboundedSender<Action>,
) -> Result<()> {
    tokio::task::Builder::new()
        .name("render_on_update")
        .spawn(async move {
            loop {
                tokio::select! {
                    result = updates.changed() => {
                        if let Err(error) = result {
                            error!(?error, "updates sender dropped");
                            break;
                        }
                    }
                    result = events.recv() => {
                        if let Err(error) = result {
                            error!(?error, "tracing event sender dropped");
                            break;
                        }
                    }
                }

                // drop all other events in the queue because we only need to re-render once
                events = events.resubscribe();

                let _ = action_tx.send(Action::Render).or_else(|e| -> Result<()> {
                    error!(?e, "failed to trigger rendering");
                    Ok(())
                });
            }
        })?;

    Ok(())
}
