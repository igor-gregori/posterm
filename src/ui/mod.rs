pub mod dialog;
pub mod request;
pub mod response;
pub mod sidebar;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    // 3 vertical columns filling 100% width
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30),  // Sidebar column
            Constraint::Min(1),     // Request (fills remaining)
            Constraint::Min(1),     // Response (fills remaining)
        ])
        .split(frame.area());

    // Sidebar column: Collections (top) + Shortcuts panel (bottom)
    let sidebar_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(5)])
        .split(columns[0]);

    // Right side split equally between Request and Response
    let right_area = Rect {
        x: columns[1].x,
        y: columns[1].y,
        width: frame.area().width - columns[0].width,
        height: frame.area().height,
    };
    let right_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(right_area);

    // Render panels
    sidebar::draw(frame, app, sidebar_split[0]);
    draw_shortcuts_panel(frame, app, sidebar_split[1]);
    request::draw(frame, app, right_panels[0]);
    response::draw(frame, app, right_panels[1]);

    // Dialog overlay (on top of everything)
    dialog::draw(frame, app);
}

fn draw_shortcuts_panel(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Info ")
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let env_badge = match app.environments.active_env() {
        Some(env) => {
            let (r, g, b) = crate::storage::env::color_to_rgb(&env.color);
            let env_color = Color::Rgb(r, g, b);
            Line::from(Span::styled(
                format!(" {} ({} vars) ", env.name, env.variables.len()),
                Style::default().fg(Color::Black).bg(env_color).add_modifier(Modifier::BOLD),
            ))
        }
        None => Line::from(Span::styled(
            " no env ",
            Style::default().fg(Color::DarkGray),
        )),
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(" F1", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Help", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" F2", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Configs", Style::default().fg(Color::DarkGray)),
        ]),
        env_badge,
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
