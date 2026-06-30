use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, Dialog};

pub fn draw(frame: &mut Frame, app: &App) {
    let Some(dialog) = &app.dialog else {
        return;
    };

    let area = centered_rect(50, 5, frame.area());

    frame.render_widget(Clear, area);

    let title = match dialog {
        Dialog::SaveRequest => " Save Request ",
        Dialog::NewCollection => " New Collection ",
    };

    let label = match dialog {
        Dialog::SaveRequest => "Name: ",
        Dialog::NewCollection => "Name: ",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let line = Line::from(vec![
        Span::styled(label, Style::default().fg(Color::DarkGray)),
        Span::styled(&app.dialog_input, Style::default().fg(Color::White)),
        Span::styled("▌", Style::default().fg(Color::Cyan)),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, inner);
}

fn centered_rect(width_percent: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}
