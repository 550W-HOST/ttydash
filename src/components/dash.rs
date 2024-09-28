use std::sync::{Arc, RwLock};

use super::Component;
use crate::action::Action;
use color_eyre::Result;

use lazy_static::lazy_static;
use ratatui::{prelude::*, widgets::*};
use regex::Regex;

use symbols::bar;
use tokio::{io::AsyncBufReadExt, sync::mpsc::UnboundedSender, task};

#[derive(Debug)]
struct DashState {
    data: Vec<f64>,
    unit: String,
}

impl DashState {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0.0; size],
            unit: String::new(),
        }
    }

    fn update(&mut self, value: f64) {
        self.data.rotate_left(1);
        self.data[0] = value;
    }
}

impl Default for DashState {
    fn default() -> Self {
        Self::new(200)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Dash {
    command_tx: Option<UnboundedSender<Action>>,
    state: Arc<RwLock<DashState>>,
    bar_set: bar::Set,
}

impl Dash {
    pub fn new() -> Self {
        let bar_set = bar::Set {
            full: "⣿",
            seven_eighths: "⣾",
            three_quarters: "⣶",
            five_eighths: "⣴",
            half: "⣤",
            three_eighths: "⣠",
            one_quarter: "⣀",
            one_eighth: "⢀",
            empty: "⠀",
        };
        let instance = Self {
            command_tx: None,
            state: Arc::new(RwLock::new(DashState::default())),
            bar_set,
        };
        let cloned_instance = instance.clone();
        task::spawn(cloned_instance.update_chart());
        instance
    }

    async fn update_chart(self) {
        let stdin = tokio::io::stdin();
        let mut lines = tokio::io::BufReader::new(stdin).lines();
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let line = lines.next_line().await.unwrap().unwrap();
            let mut state = self.state.write().unwrap();
            let value = line
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            state.update(value);
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
        let width = area.width - 1;
        let state = self.state.read().unwrap();
        let chart_state = &state.data;

        let len = chart_state.len();
        let start = len.saturating_sub(width as usize);
        let bars = chart_state[start..]
            .iter()
            .map(|&value| Bar::default().value(value as u64))
            .collect::<Vec<_>>();

        let max_time = width;
        let time_labels = (1..)
            .map(|i| i * 30)
            .take_while(|&t| t <= max_time - 5)
            .collect::<Vec<_>>();
        let mut span_vec = vec![];
        let mut last_label_len = 0;
        for &time in &time_labels {
            let pos = width - time - 1;
            if pos < (width) {
                span_vec.push(Span::raw("─".repeat(30 - last_label_len)));
                span_vec.push(Span::raw("├"));
                span_vec.push(Span::styled(format!("{}s", time), Style::default().gray()));
                last_label_len = format!("{}s", time).len() + 1;
            }
        }
        span_vec.reverse();
        let chart = BarChart::default()
            .data(BarGroup::default().bars(&bars))
            .bar_set(self.bar_set.clone())
            .bar_gap(0)
            .bar_style(Style::default().fg(Color::Green))
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .title(Line::from(state.unit.clone()).centered())
                    .title_bottom(Line::from(span_vec))
                    .title_alignment(Alignment::Right)
                    .borders(Borders::ALL),
            )
            .bar_width(1);
        frame.render_widget(chart, area);
        Ok(())
    }
}
