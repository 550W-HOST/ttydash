use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

use super::Component;
use crate::{
    action::Action,
    cli::{Cli, Unit},
};
use color_eyre::Result;

use ratatui::{prelude::*, widgets::*};

use symbols::bar;
use tokio::{io::AsyncBufReadExt, sync::mpsc::UnboundedSender, task};

#[derive(Debug, Clone)]
struct DashState {
    data: Vec<f64>,
    unit: String,
    length: usize,
    min_value: f64,
    max_value: f64,
    average: f64,
}

impl DashState {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0.0; size],
            unit: String::new(),
            length: 0,
            min_value: f64::INFINITY,
            max_value: f64::NEG_INFINITY,
            average: 0.0,
        }
    }

    fn calculate_stats(&mut self) {
        let data_slice = &self.data[self.data.len() - self.length..];
        let sum: f64 = data_slice.iter().sum();
        let len = data_slice.len() as f64;
        self.average = sum / len;
        self.min_value = data_slice.iter().copied().fold(f64::INFINITY, f64::min);
        self.max_value = data_slice.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    }

    fn update(&mut self, value: f64) {
        self.data.rotate_left(1);
        self.data[0] = value;
        self.calculate_stats();
        self.length = std::cmp::min(self.length + 1, self.data.len());
    }
}

impl Default for DashState {
    fn default() -> Self {
        Self::new(200)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Dash {
    bar_set: bar::Set,
    update_frequency: u64,

    state: Arc<RwLock<Vec<DashState>>>,
    titles: Option<Vec<String>>,
    units: Vec<Unit>,
    indices: Option<Vec<usize>>,

    command_tx: Option<UnboundedSender<Action>>,
    stop_signal: Arc<AtomicBool>,
}

impl Dash {
    pub fn new(args: Cli) -> Self {
        let bar_set = bar::Set {
            full: "⣿",
            seven_eighths: "⣾",
            three_quarters: "⣶",
            five_eighths: "⣴",
            half: "⣤",
            three_eighths: "⣠",
            one_quarter: "⣀",
            one_eighth: "⢀",
            empty: " ",
        };
        let stop_signal = Arc::new(AtomicBool::new(false));
        let units = args.units.unwrap_or_default();
        let instance = Self {
            titles: args.titles,
            state: Arc::new(RwLock::new(vec![DashState::default()])),
            units,
            indices: args.indices,
            command_tx: None,
            update_frequency: 1000,
            bar_set,
            stop_signal: stop_signal.clone(),
        };
        let cloned_instance = instance.clone();
        task::spawn(cloned_instance.update_chart(stop_signal));
        instance
    }

    async fn update_chart(self, stop_signal: Arc<AtomicBool>) {
        let stdin = tokio::io::stdin();
        let mut lines = tokio::io::BufReader::new(stdin).lines();
        while !stop_signal.load(Ordering::Relaxed) {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.update_frequency)).await;
            let line = lines.next_line().await.unwrap().unwrap();
            let mut state = self.state.write().unwrap();
            if !self.units.is_empty() {
                for (i, unit) in self.units.iter().enumerate() {
                    let unit_str = unit.to_string();
                    // parse the value with the unit
                    let re = regex::Regex::new(&format!(r"(?i)\b(\d+(\.\d+)?)\s*{}\b", unit_str))
                        .unwrap();
                    if let Some(captures) = re.captures(&line) {
                        let value = captures
                            .get(1)
                            .and_then(|v| v.as_str().parse::<f64>().ok())
                            .unwrap_or(0.0);
                        // state.update(value);
                        state[i].update(value);
                        state[i].unit = unit_str.to_string();
                    }
                }
            } else if let Some(_) = line.split_whitespace().next() {
                let values: Vec<f64> = line
                    .split_whitespace()
                    .filter_map(|value_str| value_str.parse::<f64>().ok())
                    .collect();
                if let Some(indices) = &self.indices {
                    // Update only the specified indices
                    if state.len() < values.len() {
                        state.resize(indices.len(), DashState::default());
                    }
                    indices
                        .iter()
                        .filter_map(|&index| values.get(index - 1).map(|&value| value)) // Safe access to values
                        .enumerate()
                        .for_each(|(i, value)| state[i].update(value));
                } else {
                    if state.len() < values.len() {
                        state.resize(values.len(), DashState::default());
                    }
                    state
                        .iter_mut()
                        .zip(values.iter())
                        .for_each(|(state_item, &value)| {
                            state_item.update(value);
                        });
                }
            }
        }
        // release the IO
        drop(lines);
    }
}

impl Drop for Dash {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
    }
}

impl Dash {
    fn draw_chart(&mut self, frame: &mut Frame, area: &Rect, i: usize) -> Result<()> {
        let title = self
            .titles
            .as_ref()
            .and_then(|titles| titles.get(i))
            .unwrap_or(&format!("Chart {}", i + 1))
            .to_string();
        let state = self.state.read().unwrap();
        let state = &state[i];
        let chart_state = &state.data;
        let width = area.width - 1;
        let start = chart_state.len().saturating_sub(width as usize);
        let bars = chart_state[start..]
            .iter()
            .map(|&value| Bar::default().value(value as u64).text_value("".to_owned()))
            .collect::<Vec<_>>();

        let time_labels = (1..)
            .map(|i| i * 30)
            .take_while(|&t| t <= width - 5)
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
                    .title(Line::from(title).right_aligned())
                    .title_bottom(Line::from(span_vec))
                    .title_alignment(Alignment::Right)
                    .borders(Borders::ALL),
            )
            .bar_width(1);
        frame.render_widget(chart, *area);

        let [top, _] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(*area);

        let message = format!(
            "Avg: {:.2} {} Min: {:.2} {} Max: {:.2} {}",
            state.average, state.unit, state.min_value, state.unit, state.max_value, state.unit
        );
        let span = Span::styled(message, Style::new().dim());
        let paragraph = Paragraph::new(span)
            .left_aligned()
            .block(Block::default().padding(Padding::horizontal(2)));
        frame.render_widget(paragraph, top);
        Ok(())
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

        let num_chart_states = state.len();
        // split the area
        let chunks =
            Layout::horizontal(vec![Constraint::Percentage(100); num_chart_states]).split(area);
        // release the lock
        drop(state);
        for (i, chunk) in chunks.iter().enumerate() {
            self.draw_chart(frame, chunk, i)?;
        }
        Ok(())
    }
}
