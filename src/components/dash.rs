use std::sync::{Arc, RwLock};

use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use symbols::Marker;
use tokio::{io::AsyncBufReadExt, sync::mpsc::UnboundedSender, task};
use tracing::info;

use super::Component;
use crate::action::Action;
use crate::components::barchart::Bar;
use crate::components::barchart::BarChart;
use crate::components::barchart::BarGroup;
use lazy_static::lazy_static;
use regex::Regex;

fn extract_ping(input: &str) -> Option<&str> {
    // paser the ping value from the line
    // PING puqing.work (2606:4700:3037::6815:2eae) 56 data bytes
    // 64 bytes from 2606:4700:3037::6815:2eae: icmp_seq=1 ttl=52 time=198 ms
    // 64 bytes from 2606:4700:3037::6815:2eae: icmp_seq=2 ttl=52 time=221 ms
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(\d+) bytes from .*: icmp_seq=\d+ ttl=\d+ time=(?P<ping>\d+) ms").unwrap();
    }
    RE.captures(input)
        .and_then(|cap| cap.name("ping").map(|ping| ping.as_str()))
}

#[derive(Debug, Default)]
struct DashState {
    chart_state: Vec<f64>,
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
        let this = instance.clone();
        task::spawn(this.update_chart());
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
                state.chart_state.push(value);
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
        let state = self.state.read().unwrap();
        // every data will convert to u64 for now
        let bars = state
            .chart_state
            .iter()
            .map(|&value| Bar::default().value(value as u64))
            .collect::<Vec<_>>();
        let chart = BarChart::default()
            .data(BarGroup::default().bars(&bars))
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .border_style(style::Color::White)
                    .title("Ping Chart")
                    .borders(Borders::ALL),
            )
            .bar_width(1);
        frame.render_widget(chart, area);
        Ok(())
    }
}
