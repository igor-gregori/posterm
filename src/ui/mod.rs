pub mod request;
pub mod response;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Panel};

pub fn draw(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(main_layout[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Sidebar
    let sidebar = Block::default()
        .title(" Collections ")
        .borders(Borders::ALL)
        .border_style(border_style(app, Panel::Sidebar));
    frame.render_widget(sidebar, chunks[0]);

    // Request panel
    request::draw(frame, app, right[0]);

    // Response panel
    response::draw(frame, app, right[1]);

    // Footer
    draw_footer(frame, app, main_layout[1]);
}

fn draw_footer(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let hints = if app.editing.is_some() {
        vec![
            ("Esc", "Cancel"),
            ("Enter", "Confirm"),
            ("Tab", "Next field"),
        ]
    } else {
        vec![
            ("Ctrl+R", "Send"),
            ("Ctrl+T", "Method"),
            ("Ctrl+U", "URL"),
            ("Ctrl+H", "Headers"),
            ("Ctrl+B", "Body"),
            ("Ctrl+P", "Params"),
            ("Tab", "Panel"),
            ("q", "Quit"),
        ]
    };

    let spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut s = vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default().fg(Color::Black).bg(Color::DarkGray).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(Color::DarkGray)),
            ];
            if i < hints.len() - 1 {
                s.push(Span::raw(" "));
            }
            s
        })
        .collect();

    let footer = Paragraph::new(Line::from(spans));
    frame.render_widget(footer, area);
}

fn border_style(app: &App, panel: Panel) -> Style {
    if app.active_panel == panel {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}
