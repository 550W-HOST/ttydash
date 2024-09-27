use std::collections::HashMap;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_deref::{Deref, DerefMut};
use ratatui::prelude::Rect;
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::{
    action::Action,
    components::{dash::Dash, fps::FpsCounter, Component},
    tui::{Event, Tui},
};

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct KeyBindings {
    bindings: HashMap<Vec<KeyEvent>, Action>,
}
/// A structure to manage key bindings for actions.
///
/// # Methods
///
/// * `new` - Creates a new instance of `KeyBindings`.
/// * `bind` - Binds a vector of `KeyEvent` to an `Action`.
/// * `bind_keys` - Binds a vector of tuples containing `KeyCode` and `KeyModifiers` to an `Action`.
/// * `get` - Retrieves the `Action` associated with a vector of `KeyEvent`, if it exists.
impl KeyBindings {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
    pub fn bind(&mut self, keys: Vec<KeyEvent>, action: Action) {
        self.bindings.insert(keys, action);
    }
    /// Binds multiple keys to a single action.
    ///
    /// # Arguments
    ///
    /// * `keys` - A vector of tuples where each tuple contains a `KeyCode` and `KeyModifiers`.
    /// * `action` - The action to be performed when any of the keys are pressed.
    ///
    /// # Example
    ///
    /// ```
    /// keybindings.bind_keys(
    ///     vec![
    ///         (KeyCode::Char('Q'), KeyModifiers::NONE),
    ///         (KeyCode::Char('q'), KeyModifiers::NONE),
    ///     ],
    ///     Action::Quit,
    /// );
    /// ```
    pub fn bind_keys(&mut self, keys: Vec<(KeyCode, KeyModifiers)>, action: Action) {
        for (key, modifier) in keys {
            self.bind(vec![KeyEvent::new(key, modifier)], action.clone());
        }
    }
    pub fn get(&self, keys: &Vec<KeyEvent>) -> Option<&Action> {
        self.bindings.get(keys)
    }
}

pub struct App {
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    keybindings: KeyBindings,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let mut keybindings = KeyBindings::new();
        keybindings.bind_keys(
            vec![
                (KeyCode::Char('Q'), KeyModifiers::NONE),
                (KeyCode::Char('q'), KeyModifiers::NONE),
            ],
            Action::Quit,
        );
        keybindings.bind_keys(
            vec![
                (KeyCode::Char('s'), KeyModifiers::CONTROL),
                (KeyCode::Char('S'), KeyModifiers::CONTROL),
            ],
            Action::Suspend,
        );

        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(Dash::new()), Box::new(FpsCounter::default())],
            should_quit: false,
            should_suspend: false,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            keybindings,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
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
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();
        info!("Got key event: {key:?}");
        match self.keybindings.get(&vec![key]) {
            Some(action) => {
                info!("Got action: {action:?}");
                action_tx.send(action.clone())?
            }
            _ => {
                self.last_tick_key_events.push(key);
                // Check for multi-key combinations
                if let Some(action) = self.keybindings.get(&self.last_tick_key_events) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                }
            }
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
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
}
