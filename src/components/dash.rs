use std::sync::{Arc, RwLock};

use super::Component;
use crate::action::Action;
use crate::components::barchart::Bar;
use crate::components::barchart::BarChart;
use crate::components::barchart::BarGroup;
use color_eyre::Result;
use lazy_static::lazy_static;
use ratatui::{prelude::*, widgets::*};
use regex::Regex;
use tokio::{io::AsyncBufReadExt, sync::mpsc::UnboundedSender, task};
use tracing::{debug, info};

fn extract_ping(input: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(\d+) bytes from .*: icmp_seq=\d+ ttl=\d+ time=(?P<ping>\d+) ms").unwrap();
    }
    RE.captures(input)
        .and_then(|cap| cap.name("ping").map(|ping| ping.as_str()))
}

#[derive(Debug)]
struct DashState {
    chart_state: [f64; 96],
}

impl Default for DashState {
    fn default() -> Self {
        Self {
            chart_state: [0.0; 96],
        }
    }
}

impl DashState {
    fn update(&mut self, value: f64) {
        self.chart_state.rotate_left(1);
        self.chart_state[0] = value;
    }
}

#[derive(Debug, Default, Clone)]
pub struct Dash {
    command_tx: Option<UnboundedSender<Action>>,
    state: Arc<RwLock<DashState>>,
}

impl Dash {
    pub fn new() -> Self {
        let instance = Self {
            command_tx: None,
            state: Arc::new(RwLock::new(DashState::default())),
        };
        let cloned_instance = instance.clone();
        task::spawn(cloned_instance.update_chart());
        instance
    }

    async fn update_chart(self) {
        let stdin = tokio::io::stdin();
        let mut lines = tokio::io::BufReader::new(stdin).lines();
        loop {
            let line = lines.next_line().await.unwrap().unwrap();
            let match_loop = || -> Option<()> {
                let ping = extract_ping(&line)?;
                let value: f64 = ping.parse().ok()?;
                let mut state = self.state.write().unwrap();
                state.update(value);
                Some(())
            };
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            if let Some(_) = match_loop() {
                continue;
            }
        }
    }
}

impl Component for Dash {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let width = (area.width - 1) / 2;

        let state = self.state.read().unwrap();
        let chart_state = &state.chart_state;

        let len = chart_state.len();
        let start = len.saturating_sub(width as usize);
        let bars = chart_state[start..]
            .iter()
            .map(|&value| Bar::default().value(value as u64))
            .collect::<Vec<_>>();

        let chart = BarChart::default()
            .data(BarGroup::default().bars(&bars))
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .title("Ping Chart")
                    .borders(Borders::ALL),
            )
            .bar_width(1);

        frame.render_widget(chart, area);
        Ok(())
    }
}
