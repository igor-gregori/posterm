pub mod request;
pub mod response;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};

use crate::app::{App, Panel};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(frame.area());

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let sidebar = Block::default()
        .title(" Collections ")
        .borders(Borders::ALL)
        .border_style(border_style(app, Panel::Sidebar));

    let request = Block::default()
        .title(" Request ")
        .borders(Borders::ALL)
        .border_style(border_style(app, Panel::Request));

    let response = Block::default()
        .title(" Response ")
        .borders(Borders::ALL)
        .border_style(border_style(app, Panel::Response));

    frame.render_widget(sidebar, chunks[0]);
    frame.render_widget(request, right[0]);
    frame.render_widget(response, right[1]);
}

fn border_style(app: &App, panel: Panel) -> Style {
    if app.active_panel == panel {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}
