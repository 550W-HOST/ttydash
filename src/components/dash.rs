use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

use super::Component;
use crate::{
    action::Action,
    cli::{self, Cli},
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
    group: bool,
    layout: cli::Layout,

    state: Arc<RwLock<Vec<DashState>>>,
    titles: Option<Vec<String>>,
    units: Vec<String>,
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
            group: args.group.unwrap_or(false),
            indices: args.indices,
            command_tx: None,
            update_frequency: args.update_frequency,
            bar_set,
            layout: args.layout.unwrap_or_default(),
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
            } else if line.split_whitespace().next().is_some() {
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
                        .filter_map(|&index| values.get(index - 1).copied()) // Safe access to values
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
    fn draw_grouped_chart(&mut self, frame: &mut Frame, area: &Rect) -> Result<()> {
        let state = self.state.read().unwrap();
        let window_size = (area.width - 1) / state.len() as u16;

        // Create time labels at intervals of 30, up to window_size - 5
        let time_labels = (1..)
            .map(|i| i * 30)
            .take_while(|&t| t <= window_size - 5)
            .collect::<Vec<_>>();

        let mut span_vec = vec![];
        let mut last_label_len = 0;

        // Generate time markers for the chart axis
        for &time in &time_labels {
            let pos = window_size - time - 1;
            if pos < window_size {
                // Add spacing and time marker (e.g., "30s", "60s") with a line separator
                span_vec.push(Span::raw("─".repeat(30 * state.len() - last_label_len)));
                span_vec.push(Span::raw("├"));
                span_vec.push(Span::styled(format!("{}s", time), Style::default().gray()));
                last_label_len = format!("{}s", time).len() + 1;
            }
        }

        span_vec.reverse(); // Reverse the order to display correctly on the chart

        // Initialize the bar chart with styling and layout
        let mut chart = BarChart::default()
            .bar_set(self.bar_set.clone())
            .bar_gap(0)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .title(Line::from("Group Chart").right_aligned()) // Add chart title
                    .title_bottom(Line::from(span_vec)) // Add time markers
                    .title_alignment(Alignment::Right)
                    .borders(Borders::ALL),
            )
            .bar_width(1)
            .group_gap(0);

        // Define a color map to style the bars
        let color_map = [
            Color::Green,
            Color::Red,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::White,
        ];

        // Map the grouped bars and create bar groups
        let grouped_bars = 0..window_size;

        // Generate the bar groups with their corresponding data and styles
        let mut bars = grouped_bars
            .map(|i| {
                BarGroup::default().bars(
                    &(0..state.len())
                        .map(|n| {
                            let state_n = &state[n];
                            // Fetch the value from state and create the Bar
                            let value =
                                state_n.data[state_n.data.len().saturating_sub((i + 1).into())];
                            Bar::default()
                                .value(value as u64)
                                .text_value("".to_owned()) // No text value displayed for the bars
                                .style(Style::default().fg(color_map[n % color_map.len()]))
                            // Style based on color map
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        bars.reverse(); // Reverse the bar order to match display order

        // Add each bar group to the chart
        bars.iter().for_each(|bar_group| {
            chart = chart.clone().data(bar_group.clone());
        });

        // Render the chart on the frame
        frame.render_widget(chart, *area);
        Ok(())
    }

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

fn is_prime(n: usize) -> bool {
    if n < 2 {
        return false;
    }
    for i in 2..=((n as f64).sqrt() as usize) {
        if n % i == 0 {
            return false;
        }
    }
    true
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
        if !self.group {
            let state = self.state.read().unwrap();
            let num_chart_states = state.len();
            // split the area
            let chunks = match self.layout {
                cli::Layout::Vertical => {
                    Layout::vertical(vec![Constraint::Percentage(100); num_chart_states])
                        .split(area)
                        .iter()
                        .copied()
                        .collect::<Vec<_>>()
                }
                cli::Layout::Horizontal => {
                    Layout::horizontal(vec![Constraint::Percentage(100); num_chart_states])
                        .split(area)
                        .iter()
                        .copied()
                        .collect::<Vec<_>>()
                }
                cli::Layout::Auto => {
                    if is_prime(num_chart_states) {
                        // grid + 1
                        let (rows, cols) = match num_chart_states - 1 {
                            1 => (1, 1),
                            2 => (1, 2),
                            _ => {
                                let rows = (2..=num_chart_states - 1)
                                    .rev()
                                    .find(|&i| num_chart_states % i == 0)
                                    .unwrap_or(1);
                                let cols = num_chart_states / rows;
                                (rows, cols)
                            }
                        };
                        let row_constraints =
                            vec![Constraint::Percentage((100 / rows + 1) as u16); rows + 1];
                        let row_chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints(row_constraints)
                            .split(area);
                        let mut chunks = vec![];
                        for row_chunk in row_chunks[1..].iter() {
                            let col_constraints =
                                vec![Constraint::Percentage((100 / cols) as u16); cols];
                            let col_chunks = Layout::default()
                                .direction(Direction::Horizontal)
                                .constraints(col_constraints)
                                .split(*row_chunk);
                            let col_chunks_vec = col_chunks.iter().copied().collect::<Vec<_>>();
                            chunks.extend(col_chunks_vec);
                        }
                        chunks.insert(0, row_chunks[0]);
                        chunks
                    } else {
                        let (rows, cols) = match num_chart_states {
                            1 => (1, 1),
                            2 => (1, 2),
                            _ => {
                                let rows = (2..=num_chart_states - 1)
                                    .rev()
                                    .find(|&i| num_chart_states % i == 0)
                                    .unwrap_or(1);
                                let cols = num_chart_states / rows;
                                (rows, cols)
                            }
                        };
                        let row_constraints =
                            vec![Constraint::Percentage((100 / rows) as u16); rows];
                        let row_chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints(row_constraints)
                            .split(area);
                        let mut chunks = vec![];
                        for row_chunk in row_chunks.iter() {
                            let col_constraints =
                                vec![Constraint::Percentage((100 / cols) as u16); cols];
                            let col_chunks = Layout::default()
                                .direction(Direction::Horizontal)
                                .constraints(col_constraints)
                                .split(*row_chunk);
                            let col_chunks_vec = col_chunks.iter().copied().collect::<Vec<_>>();
                            chunks.extend(col_chunks_vec);
                        }
                        chunks
                    }
                }
            };
            // release the lock
            drop(state);
            for (i, chunk) in chunks.iter().enumerate() {
                self.draw_chart(frame, chunk, i)?;
            }
        } else {
            self.draw_grouped_chart(frame, &area)?;
        }
        Ok(())
    }
}
