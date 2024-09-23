use std::{sync::Arc, time::Duration};

use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use symbols::Marker;
use tokio::{
    sync::{
        mpsc::{self, UnboundedSender},
        Mutex,
    },
    task,
};

use super::Component;
use crate::action::Action;

#[derive(Debug, Clone)]
struct Data {
    values: Vec<(f64, f64)>,
    index: f64,
}

impl Data {
    pub fn new() -> Self {
        Self {
            values: vec![],
            index: 0.0,
        }
    }

    pub fn push(&mut self, value: f64) {
        self.values.push((self.index, value));
        self.index += 1.0;
    }
}

#[derive()]
pub struct Dash {
    command_tx: Option<UnboundedSender<Action>>,
    datas: Data,
    data_rx: mpsc::Receiver<f64>,
}

impl Dash {
    pub fn new(data_rx: mpsc::Receiver<f64>) -> Arc<Mutex<Self>> {
        let dash = Arc::new(Mutex::new(Self {
            command_tx: None,
            data_rx,
            datas: Data::new(),
        }));

        let dash_clone = Arc::clone(&dash);
        task::spawn(async move {
            let mut dash = dash_clone.lock().await;
            while let Some(data) = dash.data_rx.recv().await {
                dash.receive_data(data).await;
            }
        });

        dash
    }

    async fn receive_data(&mut self, data: f64) {
        self.datas.push(data);
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
        let dataset = Dataset::default()
            .marker(Marker::Dot)
            .style(Style::default().fg(Color::Yellow))
            .data(&self.datas.values);

        let chart = Chart::new(vec![dataset])
            .block(
                Block::default()
                    .title("Data")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .x_axis(
                Axis::default()
                    .title("X Axis")
                    .style(Style::default().fg(Color::Gray)),
            )
            .y_axis(
                Axis::default()
                    .title("Y Axis")
                    .style(Style::default().fg(Color::Gray)),
            );
        frame.render_widget(chart, area);
        Ok(())
    }
}
